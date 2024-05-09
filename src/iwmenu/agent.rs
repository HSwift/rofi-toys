use dbus::blocking::Connection;
use dbus::Path;
use dbus_crossroads::{Context, Crossroads};
use std::{process, thread};

use crate::service::DBUS_TIMEOUT;

const IWD_AGENT_INTERFACE: &str = "net.connman.iwd.Agent";

pub struct IWDAgent {
    cr: Crossroads,
    conn: Connection,
    user_data: IWDAgentUserData,
    method_path: String,
}

#[derive(Clone)]
pub struct IWDAgentUserData {
    pub passphrase: String,
}

impl IWDAgent {
    pub fn new(user_data: IWDAgentUserData) -> IWDAgent {
        IWDAgent {
            cr: Crossroads::new(),
            conn: Connection::new_system().unwrap(),
            user_data: user_data,
            method_path: format!("/agent/{}", process::id()),
        }
    }
    pub fn agent_interface(iface_builder: &mut dbus_crossroads::IfaceBuilder<IWDAgentUserData>) {
        iface_builder.method(
            "RequestPassphrase",
            ("network",),
            ("passphrase",),
            IWDAgent::agent_request_passphrase_method,
        );
    }
    pub fn agent_request_passphrase_method(
        _: &mut Context,
        user_data: &mut IWDAgentUserData,
        (_,): (Path,),
    ) -> Result<(std::string::String,), dbus::MethodErr> {
        Ok((user_data.passphrase.clone(),))
    }
    pub fn register(&mut self) -> anyhow::Result<()> {
        let iface = self
            .cr
            .register(IWD_AGENT_INTERFACE, IWDAgent::agent_interface);

        let proxy: dbus::blocking::Proxy<'_, &Connection> =
            self.conn
                .with_proxy("net.connman.iwd", "/net/connman/iwd", DBUS_TIMEOUT);

        proxy.method_call(
            "net.connman.iwd.AgentManager",
            "RegisterAgent",
            (Path::from(&self.method_path),),
        )?;
        self.cr
            .insert(self.method_path.clone(), &[iface], self.user_data.clone());

        Ok(())
    }
    pub fn serve(self) {
        thread::spawn(move || {
            self.cr.serve(&self.conn).unwrap();
        });
    }
}
