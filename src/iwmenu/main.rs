mod agent;
mod service;

use anyhow::{self, Ok};

use rofi_toys::rofi::RofiPlugin;
use rofi_toys::utils::make_table_column;

fn display_network_strength(dbms: i16) -> String {
    if dbms >= -6000 {
        "****".to_string()
    } else if dbms >= -6700 {
        "***".to_string()
    } else if dbms >= -7500 {
        "**".to_string()
    } else {
        "*".to_string()
    }
}

fn list_available_networks(rofi: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let mut iwd_info = service::get_iwd_info()?;
    service::scan_available_networks(&iwd_info)?;
    service::get_signal_strength(&mut iwd_info)?;

    if iwd_info.connected {
        rofi.set_message(&format!(
            "<b>current network: {}</b>",
            iwd_info.current_network
        ));
        rofi.add_menu_entry_with_params("[disconnect]", disconnect, vec![]);
    }
    rofi.add_menu_entry_with_params("[refresh]", list_available_networks, vec![]);

    for (path, network_info) in iwd_info.available_networks {
        if network_info.connected {
            continue;
        }

        let network_desc = format!(
            "{}{}{}",
            make_table_column(network_info.name, 40),
            make_table_column(network_info.security_type, 6),
            display_network_strength(network_info.strength),
        );
        let mut params = vec![path.to_string()];
        if network_info.known {
            params.push(String::new());
        }
        rofi.add_menu_entry_with_params(&network_desc, connect_to_network, params);
    }

    Ok(())
}

fn connect_to_network(_: &RofiPlugin, params: Vec<String>) -> anyhow::Result<()> {
    let iwd_info = service::get_iwd_info()?;
    let network_path = &params[0];
    let passphrase = &params[1];

    let target_network = iwd_info
        .available_networks
        .get(&dbus::Path::from(network_path));
    match target_network {
        Some(x) => {
            if x.known {
                service::connect_to_known_network(&iwd_info, network_path.as_str())?;
            } else {
                service::connect_to_network_with_passphrase(
                    &iwd_info,
                    network_path.as_str(),
                    passphrase.as_str(),
                )?;
            }
        }
        None => {}
    }

    Ok(())
}

fn disconnect(_: &RofiPlugin, _: Vec<String>) -> anyhow::Result<()> {
    let iwd_info = service::get_iwd_info()?;
    service::disconnect_from_current_network(&iwd_info)?;

    Ok(())
}

fn main() {
    let mut rofi = RofiPlugin::new();

    rofi.register_entrypoint(list_available_networks);

    rofi.register_callback_with_params(
        connect_to_network,
        vec![String::from("path"), String::from("passphrase")],
    );
    rofi.register_callback(disconnect);

    rofi.run();
}
