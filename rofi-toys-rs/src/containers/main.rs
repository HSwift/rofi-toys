use docker_api::docker;
use docker_api::opts::ContainerListOpts;
use once_cell::sync::Lazy;
use rofi_toys::clipboard;
use rofi_toys::rofi::{RofiPlugin, RofiPluginError};

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
        let mut result = col_text.chars().take(max_length - 3).collect::<String>();
        result.push_str("...");
        result
    } else {
        let mut result = col_text.clone();
        result.push_str(&" ".repeat(max_length - col_text.chars().count()));
        result
    }
}

pub fn list_containers(rofi: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let docker = get_docker();
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

        rofi.add_menu_entry_with_params(
            &format!(
                "id: {} image: {} name: {} status: {} command: {}",
                sid,
                make_table_column(image, 26),
                make_table_column(name, 20),
                make_table_column(status, 26),
                command,
            ),
            container_menu,
            vec![id],
        );
    });

    Ok(())
}

const NULL: &str = "[null]";

pub fn container_menu(rofi: &RofiPlugin, params: Vec<String>) -> anyhow::Result<()> {
    let id = &params[0];
    let docker = get_docker();
    let container_inspect = TOKIO.block_on(docker.containers().get(id).inspect())?;

    let sid = id.chars().take(12).collect::<String>();

    let mut name = container_inspect
        .name
        .to_owned()
        .unwrap_or_else(|| "[no name found]".to_string());
    name.drain(0..1);

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

    let (entrypoint, command, hostname) = if let Some(config) = container_inspect.config {
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

    rofi.add_menu_entry_with_params(
        "[exec]",
        copy_to_clipboard,
        vec![format!("sudo docker exec -it {} bash", &sid)],
    );

    rofi.add_menu_entry_with_params(
        "[restart]",
        copy_to_clipboard,
        vec![format!("sudo docker restart -it {} bash", &sid)],
    );

    rofi.add_menu_entry_with_params(
        "[stop]",
        copy_to_clipboard,
        vec![format!("sudo docker stop -it {} bash", &sid)],
    );

    rofi.add_menu_entry_with_params(
        "[logs]",
        copy_to_clipboard,
        vec![format!("sudo docker logs -it {} bash", &sid)],
    );

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
