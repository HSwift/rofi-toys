use std::collections::HashMap;

use docker_api::docker;
use docker_api::opts::ContainerListOpts;
use once_cell::sync::Lazy;
use rofi_toys::rofi::{RofiPlugin, RofiPluginError};
use rofi_toys::{clipboard, file};

#[derive(serde::Serialize, serde::Deserialize)]
struct ContainerConfig {
    field_order: Vec<String>,
    hide_stopped_container: bool,
    command_with_sudo: bool,
}

fn generate_default_config() -> ContainerConfig {
    let container_config = ContainerConfig {
        field_order: vec![
            "sid".to_string(),
            "image".to_string(),
            "name".to_string(),
            "status".to_string(),
            "command".to_string(),
        ],
        hide_stopped_container: true,
        command_with_sudo: true,
    };
    file::config_save_to_file(&container_config, "container").unwrap();
    return container_config;
}

fn read_config() -> ContainerConfig {
    match file::config_restore_from_file("container") {
        Ok(x) => x,
        Err(_) => generate_default_config(),
    }
}

static TOKIO: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
});

pub fn get_docker() -> docker::Docker {
    docker::Docker::unix("/var/run/docker.sock")
}

pub fn make_table_column(col_text: String, max_length: usize) -> String {
    if col_text.len() > max_length {
        let mut result = col_text.chars().take(max_length - 1).collect::<String>();
        result.push_str("…");
        result
    } else {
        let mut result = col_text.clone();
        result.push_str(&" ".repeat(max_length - col_text.chars().count()));
        result
    }
}

pub fn list_containers(rofi: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let docker = get_docker();
    let container_config = read_config();
    let mut containers = TOKIO.block_on(
        docker
            .containers()
            .list(&ContainerListOpts::builder().all(true).build()),
    )?;

    if containers.is_empty() {
        return Err(RofiPluginError::new("no contains exists").into());
    }

    containers.sort_by(|left, right| {
        if let (Some(left), Some(right)) = (left.status.clone(), right.status.clone()) {
            let left_up = left.to_ascii_lowercase().starts_with("up");
            let right_up = right.to_ascii_lowercase().starts_with("up");

            if left_up && right_up {
                return std::cmp::Ordering::Equal;
            } else if left_up && !right_up {
                return std::cmp::Ordering::Less;
            } else {
                return std::cmp::Ordering::Greater;
            }
        }

        return std::cmp::Ordering::Equal;
    });

    containers.iter().for_each(|c| {
        let id = if let Some(id) = c.id.to_owned() {
            id
        } else {
            // 必须要有 id, 没有的话跳过
            return;
        };
        if container_config.hide_stopped_container
            && c.status
                .to_owned()
                .unwrap_or_else(|| String::new())
                .starts_with("Exited")
        {
            return;
        }

        let sid = id.chars().take(12).collect::<String>();

        let mut name = c
            .names
            .to_owned()
            .unwrap_or_else(|| vec!["[no name found]".to_string()])
            .remove(0);
        name.drain(0..1);

        let image = c
            .image
            .to_owned()
            .unwrap_or_else(|| "[no image found]".to_string());

        let status = c
            .status
            .to_owned()
            .unwrap_or_else(|| "[status acuiqre failed]".to_string());

        let command = c
            .command
            .to_owned()
            .unwrap_or_else(|| "[command acuiqre failed]".to_string());

        let mut table: HashMap<String, String> = HashMap::new();
        table.insert("sid".to_string(), sid);
        table.insert("image".to_string(), make_table_column(image, 32));
        table.insert("name".to_string(), make_table_column(name, 32));
        table.insert("status".to_string(), make_table_column(status, 30));
        table.insert("command".to_string(), make_table_column(command, 40));

        let mut row_str = String::new();
        for field in &container_config.field_order {
            row_str.push_str(table.get(field).unwrap_or(&String::new()));
            row_str.push(' ')
        }

        rofi.add_menu_entry_with_params(row_str.as_str(), container_menu, vec![id]);
    });

    Ok(())
}

const NULL: &str = "[null]";

pub fn container_menu(rofi: &RofiPlugin, params: Vec<String>) -> anyhow::Result<()> {
    let id = &params[0];
    let docker = get_docker();
    let container_inspect = TOKIO.block_on(docker.containers().get(id).inspect())?;
    let container_config = read_config();

    let sid = id.chars().take(12).collect::<String>();

    let mut name = container_inspect
        .name
        .to_owned()
        .unwrap_or_else(|| "[no name found]".to_string());
    name.drain(0..1);

    let status = if let Some(state) = container_inspect.state {
        state.status.unwrap_or_else(|| NULL.to_string())
    } else {
        NULL.to_string()
    };

    let image = container_inspect
        .image
        .to_owned()
        .unwrap_or_else(|| "[no image found]".to_string());

    let image_inspect = TOKIO.block_on(docker.images().get(&image).inspect())?;
    let image_tag = image_inspect.repo_tags.to_owned();

    let image_tag = if let Some(mut image_tag) = image_tag {
        if image_tag.len() > 0 {
            image_tag.remove(0)
        } else {
            image
        }
    } else {
        image
    };

    let (entrypoint, command, hostname) = if let Some(config) = &container_inspect.config {
        let command = config
            .cmd
            .to_owned()
            .map(|x| x.join(" "))
            .unwrap_or_else(|| NULL.to_string());

        let entrypoint = config
            .entrypoint
            .to_owned()
            .map(|x| x.join(" "))
            .unwrap_or_else(|| NULL.to_string());

        let hostname = config
            .hostname
            .to_owned()
            .unwrap_or_else(|| NULL.to_string());

        (entrypoint, command, hostname)
    } else {
        (
            "[config acuiqre failed]".to_string(),
            "[config acuiqre failed]".to_string(),
            "[config acuiqre failed]".to_string(),
        )
    };

    let ports: Vec<String>;
    if let Some(config) = &container_inspect.config {
        let port_map = config
            .exposed_ports
            .to_owned()
            .unwrap_or_else(|| HashMap::new());
        ports = port_map
            .into_keys()
            .map(|x| x.split_once('/').unwrap().0.to_string())
            .collect()
    } else {
        ports = vec![]
    }

    let ip_addresses = if let Some(network) = container_inspect.network_settings {
        let mut ip_addresses = Vec::new();
        if let Some(ip_address) = network.networks {
            for (_, v) in ip_address.iter() {
                if let Some(ipv4) = v.ip_address.to_owned() {
                    if !ipv4.is_empty() {
                        ip_addresses.push(ipv4);
                    }
                }
                if let Some(ipv6) = v.global_i_pv_6_address.to_owned() {
                    if !ipv6.is_empty() {
                        ip_addresses.push(ipv6);
                    }
                }
            }
        }
        ip_addresses
    } else {
        Vec::new()
    };

    rofi.add_menu_entry_with_params(
        &format!("id:\t\t {}", &sid),
        copy_to_clipboard,
        vec![sid.clone()],
    );
    rofi.add_menu_entry_with_params(
        &format!("name:\t\t {}", &name),
        copy_to_clipboard,
        vec![name],
    );
    rofi.add_menu_entry_with_params(
        &format!("status:\t\t {}", &status),
        copy_to_clipboard,
        vec![status],
    );
    rofi.add_menu_entry_with_params(
        &format!("image:\t\t {}", &image_tag),
        copy_to_clipboard,
        vec![image_tag],
    );
    rofi.add_menu_entry_with_params(
        &format!("command:\t {}", &command),
        copy_to_clipboard,
        vec![command],
    );
    rofi.add_menu_entry_with_params(
        &format!("entrypoint:\t {}", &entrypoint),
        copy_to_clipboard,
        vec![entrypoint],
    );
    rofi.add_menu_entry_with_params(
        &format!("hostname:\t {}", &hostname),
        copy_to_clipboard,
        vec![hostname],
    );

    for port in ports {
        let ip_port = if ip_addresses.len() == 1 {
            format!("{}:{}", ip_addresses[0], &port)
        } else {
            port
        };
        rofi.add_menu_entry_with_params(
            &format!("port:\t\t {}", &ip_port),
            copy_to_clipboard,
            vec![ip_port],
        );
    }

    if ip_addresses.is_empty() {
        rofi.add_menu_entry_with_params(
            &format!("ip_addr:\t {}", NULL),
            copy_to_clipboard,
            vec![NULL.to_string()],
        );
    } else {
        for ip_address in ip_addresses {
            rofi.add_menu_entry_with_params(
                &format!("ip_addr:\t {}", &ip_address),
                copy_to_clipboard,
                vec![ip_address],
            );
        }
    }

    rofi.add_menu_line(" ");
    let sudo_prefix;
    if container_config.command_with_sudo {
        sudo_prefix = "sudo ";
    } else {
        sudo_prefix = "";
    }

    rofi.add_menu_entry_with_params(
        "[exec]",
        copy_to_clipboard,
        vec![format!("{}docker exec -it {} bash", &sudo_prefix, &sid)],
    );

    rofi.add_menu_entry_with_params(
        "[restart]",
        copy_to_clipboard,
        vec![format!("{}docker restart -t0 {}", &sudo_prefix, &sid)],
    );

    rofi.add_menu_entry_with_params(
        "[stop]",
        copy_to_clipboard,
        vec![format!("{}docker stop -t0 {}", &sudo_prefix, &sid)],
    );

    rofi.add_menu_entry_with_params(
        "[logs]",
        copy_to_clipboard,
        vec![format!("{}docker logs {}", &sudo_prefix, &sid)],
    );

    rofi.add_menu_entry_with_params(
        "[inspect]",
        copy_to_clipboard,
        vec![format!("{}docker inspect {}", &sudo_prefix, &sid)],
    );

    rofi.add_menu_entry("[back]", list_containers);

    Ok(())
}

pub fn copy_to_clipboard(_: &RofiPlugin, params: Vec<String>) -> anyhow::Result<()> {
    clipboard::clipboard_set_text(&params[0]);

    Ok(())
}

fn main() {
    let mut rofi = RofiPlugin::new();

    rofi.register_entrypoint(list_containers);

    rofi.register_callback_with_params(container_menu, vec![String::from("id")]);
    rofi.register_callback_with_params(copy_to_clipboard, vec![String::from("content")]);

    rofi.run();
}
