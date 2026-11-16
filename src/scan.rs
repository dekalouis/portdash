#![cfg(target_os = "linux")]

use serde::Serialize;
use std::{io, net::IpAddr, path::PathBuf};

use crate::{
    parse::{filter_listeners, text_to_entries},
    process::{attach_owners, read_details},
    project::find_project,
    sockets::read_socket_tables,
};

/// One process holding a listening socket.
#[derive(Serialize)]
pub struct Owner {
    pub pid: u32,
    /// Process name from `/proc/<pid>/stat`, cut at 15 characters by the kernel.
    pub name: String,
    /// Full command line, or the name in brackets when it is empty (kernel threads).
    pub command: String,
    /// Working directory, or `None` when it cannot be read.
    pub cwd: Option<PathBuf>,
}

/// One listening socket with everything portdash knows about it.
#[derive(Serialize)]
pub struct Listener {
    pub address: IpAddr,
    pub port: u16,
    /// Processes holding the socket. Empty when the owner cannot be read (another user's process).
    pub owners: Vec<Owner>,
    /// Project of the first owner that has one.
    pub project: Option<String>,
}

/// Reads every listening TCP socket with its owners and project, sorted by port.
///
/// An owner that exits during the scan is left out. Returns an error only if the
/// socket tables or `/proc` itself cannot be read.
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
