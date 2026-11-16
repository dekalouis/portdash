#![cfg(target_os = "linux")]

use std::{io, net::IpAddr, path::PathBuf};

use crate::{
    parse::{filter_listeners, text_to_entries},
    process::{attach_owners, read_details},
    project::find_project,
    sockets::read_socket_tables,
};

pub struct Owner {
    pub pid: u32,
    pub name: String,
    pub command: String,
    pub cwd: Option<PathBuf>,
}

pub struct Listener {
    pub address: IpAddr,
    pub port: u16,
    pub owners: Vec<Owner>,
    pub project: Option<String>,
}

pub fn scan() -> io::Result<Vec<Listener>> {
    let tables = read_socket_tables()?;
    let entries = tables
        .iter()
        .flat_map(|text| text_to_entries(text))
        .collect();

    let mut sockets = attach_owners(&filter_listeners(entries))?;
    sockets.sort_by_key(|socket| socket.port);

    let home = std::env::var("HOME").ok().map(PathBuf::from);
    let mut listeners: Vec<Listener> = Vec::new();

    for socket in &sockets {
        let mut owners = Vec::new();
        let mut project = None;

        for &pid in &socket.pids {
            let Some(details) = read_details(pid) else {
                continue;
            };

            let command_text = details.cmdline.join(" ");
            let command = if command_text.is_empty() {
                format!("[{}]", details.stat.name)
            } else {
                command_text
            };

            if project.is_none() {
                project = match &details.cwd {
                    Some(path) => find_project(path, home.as_deref()).map(|(_, name)| name),
                    None => None,
                };
            }

            owners.push(Owner {
                pid,
                name: details.stat.name,
                command,
                cwd: details.cwd,
            });
        }
        listeners.push(Listener {
            address: socket.address,
            port: socket.port,
            owners,
            project,
        });
    }
    Ok(listeners)
}
