use core::fmt;
use dbus::blocking::Connection;
use dbus::Path;
use rofi_toys::rofi::RofiPluginError;
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use crate::agent::{IWDAgent, IWDAgentUserData};

pub const DBUS_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub struct NetworkInfo {
    pub name: String,
    pub connected: bool,
    pub known: bool,
    pub security_type: String,
    pub strength: i16,
}

impl<'a> fmt::Display for NetworkInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}:{}, security={}, connected:{}",
            self.name, self.strength, self.security_type, self.connected
        )
    }
}

#[derive(Debug)]
pub struct IWDInfo<'a> {
    pub station: Path<'a>,
    pub device: String,
    pub scanning: bool,
    pub available_networks: BTreeMap<Path<'a>, NetworkInfo>,
    pub current_network: String,
    pub connected: bool,
}

impl<'a> fmt::Display for IWDInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "device:{}, scanning:{}\n", self.device, self.scanning)?;
        for (path, network) in &self.available_networks {
            write!(f, "{} -> {}", path, network)?;
        }
        write!(f, "\n")
    }
}

pub fn get_iwd_info<'a>() -> anyhow::Result<IWDInfo<'a>> {
    let conn = Connection::new_system().unwrap();
    let proxy: dbus::blocking::Proxy<'_, &Connection> =
        conn.with_proxy("net.connman.iwd", "/", DBUS_TIMEOUT);
    let mut iwd_info = IWDInfo {
        station: Path::default(),
        device: "unknown".to_string(),
        scanning: false,
        available_networks: BTreeMap::new(),
        current_network: "".to_string(),
        connected: false,
    };

    use dbus::blocking::stdintf::org_freedesktop_dbus::ObjectManager;

    let objects = proxy.get_managed_objects()?;
    for (path, elem) in objects {
        if elem.contains_key("net.connman.iwd.Device") {
            iwd_info.station = path.clone();
            let device = elem.get("net.connman.iwd.Device").unwrap();
            let device_name = device.get("Name");
            if let Some(x) = device_name {
                if let Some(y) = x.0.as_str() {
                    iwd_info.device = y.to_string()
                }
            }
        }
        if elem.contains_key("net.connman.iwd.Station") {
            let station = elem.get("net.connman.iwd.Station").unwrap();
            if let Some(x) = station.get("Scanning") {
                if let Some(y) = x.0.as_u64() {
                    iwd_info.scanning = y == 1;
                }
            }
            if let Some(x) = station.get("State") {
                if let Some(y) = x.0.as_str() {
                    iwd_info.connected = y == "connected";
                }
            }
        }
        if elem.contains_key("net.connman.iwd.Network") {
            let network = elem.get("net.connman.iwd.Network").unwrap();
            let mut network_info = NetworkInfo {
                name: String::new(),
                connected: false,
                security_type: String::new(),
                known: false,
                strength: 0,
            };
            if let Some(x) = network.get("Connected") {
                if let Some(y) = x.0.as_u64() {
                    network_info.connected = y == 1;
                }
            }
            if let Some(x) = network.get("Name") {
                if let Some(y) = x.0.as_str() {
                    network_info.name = y.to_string();
                }
            }
            if let Some(x) = network.get("Type") {
                if let Some(y) = x.0.as_str() {
                    network_info.security_type = y.to_string();
                }
            }
            if network.contains_key("KnownNetwork") {
                network_info.known = true;
            }
            if network_info.connected == true {
                iwd_info.current_network = network_info.name.clone();
            }
            iwd_info.available_networks.insert(path, network_info);
        }
    }
    if iwd_info.device == "unknown" || iwd_info.station == Path::default() {
        return Err(RofiPluginError::new("can not found iwd device").into());
    }
    Ok(iwd_info)
}

pub fn scan_available_networks<'a>(iwd_info: &IWDInfo<'a>) -> anyhow::Result<()> {
    let conn = Connection::new_system().unwrap();
    let proxy: dbus::blocking::Proxy<'_, &Connection> =
        conn.with_proxy("net.connman.iwd", &iwd_info.station, DBUS_TIMEOUT);

    if iwd_info.scanning == false {
        proxy.method_call("net.connman.iwd.Station", "Scan", ())?;
    }

    Ok(())
}

pub fn get_signal_strength<'a>(iwd_info: &mut IWDInfo<'a>) -> anyhow::Result<()> {
    let conn = Connection::new_system().unwrap();
    let proxy: dbus::blocking::Proxy<'_, &Connection> =
        conn.with_proxy("net.connman.iwd", &iwd_info.station, DBUS_TIMEOUT);
    let ordered_networks: Vec<(dbus::Path<'static>, i16)>;

    (ordered_networks,) = proxy.method_call("net.connman.iwd.Station", "GetOrderedNetworks", ())?;
    for (network_path, network_strength) in ordered_networks {
        if let Some(network_info) = iwd_info.available_networks.get_mut(&network_path) {
            network_info.strength = network_strength;
        }
    }

    Ok(())
}

pub fn disconnect_from_current_network<'a>(iwd_info: &IWDInfo<'a>) -> anyhow::Result<()> {
    let conn = Connection::new_system().unwrap();
    let proxy: dbus::blocking::Proxy<'_, &Connection> =
        conn.with_proxy("net.connman.iwd", &iwd_info.station, DBUS_TIMEOUT);

    proxy.method_call("net.connman.iwd.Station", "Disconnect", ())?;

    Ok(())
}

pub fn connect_to_known_network<'a>(_iwd_info: &IWDInfo<'a>, path: &str) -> anyhow::Result<()> {
    let conn = Connection::new_system().unwrap();
    let path = dbus::Path::from(path);
    let proxy: dbus::blocking::Proxy<'_, &Connection> =
        conn.with_proxy("net.connman.iwd", &path, DBUS_TIMEOUT);

    proxy.method_call("net.connman.iwd.Network", "Connect", ())?;

    Ok(())
}

pub fn connect_to_network_with_passphrase<'a>(
    _iwd_info: &IWDInfo<'a>,
    path: &str,
    passphrase: &str,
) -> anyhow::Result<()> {
    let conn = Connection::new_system().unwrap();
    let path = dbus::Path::from(path);
    let agent_user_data = IWDAgentUserData {
        passphrase: passphrase.to_string(),
    };
    let mut iwd_agent = IWDAgent::new(agent_user_data);
    let proxy: dbus::blocking::Proxy<'_, &Connection> =
        conn.with_proxy("net.connman.iwd", &path, DBUS_TIMEOUT);

    iwd_agent.register()?;
    iwd_agent.serve();

    proxy.method_call("net.connman.iwd.Network", "Connect", ())?;

    Ok(())
}
