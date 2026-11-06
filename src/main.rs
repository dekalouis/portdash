#[cfg(target_os = "macos")]
use crate::parse::parse_lsof;
#[cfg(target_os = "linux")]
use crate::parse::{filter_listeners, text_to_entries};
#[cfg(target_os = "macos")]
use crate::process::group_into_listeners;
#[cfg(target_os = "linux")]
use crate::process::socket_owners;
#[cfg(target_os = "linux")]
use crate::sockets::read_socket_tables;
#[cfg(target_os = "macos")]
use crate::sockets::run_lsof;
use std::net::SocketAddr;

mod parse;
mod process;
mod sockets;

fn main() {
    #[cfg(target_os = "macos")]
    {
        let lsof_text = match run_lsof() {
            Ok(text) => text,
            Err(err) => {
                eprintln!("could not read sockets: {err}");
                std::process::exit(1);
            }
        };
        let socket_lines = lsof_text
            .lines()
            .filter(|line| line.starts_with('n'))
            .count();

        let entries = match parse_lsof(&lsof_text) {
            Ok(entries) => entries,
            Err(err) => {
                // TODO: {:?} is development output. Replace it (Stage 8a).
                eprintln!("could not parse sockets: {err:?}");
                std::process::exit(1);
            }
        };

        let mut listeners = group_into_listeners(&entries);
        listeners.sort_by_key(|listener| listener.port);

        for listener in &listeners {
            let socket_address = SocketAddr::new(listener.address, listener.port);
            println!("{}: {:?}", socket_address, listener.pids);
        }
        println!("count {}, actual {}", socket_lines, entries.len());
    }

    #[cfg(target_os = "linux")]
    {
        let tables = match read_socket_tables() {
            Ok(tables) => tables,
            Err(err) => {
                eprintln!("could not read sockets: {err}");
                std::process::exit(1);
            }
        };
        let entries = tables
            .iter()
            .flat_map(|text| text_to_entries(text))
            .collect();

        let mut listeners = filter_listeners(entries);
        listeners.sort_by_key(|entry| entry.port);

        let owners = match socket_owners() {
            Ok(map) => map,
            Err(err) => {
                eprintln!("could not read socket owners: {err}");
                std::process::exit(1);
            }
        };

        for entry in &listeners {
            let socket_address = SocketAddr::new(entry.address, entry.port);
            match owners.get(&entry.inode) {
                Some(pids) => println!("{}: {:?}", socket_address, pids),
                None => println!("{}: -", socket_address),
            }
        }
        println!("count {}", listeners.len());
    }
}
