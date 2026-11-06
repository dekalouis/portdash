#[cfg(target_os = "linux")]
use crate::parse::{filter_and_sort_listeners, text_to_entries};
#[cfg(target_os = "macos")]
use crate::parse::{group_lsof_entry, parse_lsof};
#[cfg(target_os = "linux")]
use crate::sockets::read_entries;
#[cfg(target_os = "macos")]
use crate::sockets::run_lsof;
use std::net::SocketAddr;

mod parse;
mod process;
mod sockets;

fn main() {
    #[cfg(target_os = "macos")]
    match run_lsof() {
        Ok(lsof_text) => {
            let socket_lines = lsof_text
                .lines()
                .filter(|line| line.starts_with('n'))
                .count();

            match parse_lsof(&lsof_text) {
                Ok(entries) => {
                    let mut listeners = group_lsof_entry(&entries);

                    listeners.sort_by_key(|listener| listener.port);
                    for listener in &listeners {
                        let socket_address = SocketAddr::new(listener.address, listener.port);
                        println!("{}: {:?}", socket_address, listener.pids);
                    }
                    println!("count {}, actual {}", socket_lines, entries.len());
                }
                Err(err) => {
                    eprintln!("{:?}", err);
                }
            }
        }
        Err(err) => {
            eprintln!("could not read sockets: {err}");
            std::process::exit(1);
        }
    }

    #[cfg(target_os = "linux")]
    {
        let texts = match read_entries() {
            Ok(texts) => texts,
            Err(err) => {
                eprintln!("could not read sockets: {err}");
                std::process::exit(1);
            }
        };
        let mut entries = Vec::new();
        for text in &texts {
            let to_entry = text_to_entries(text);
            entries.extend(to_entry);
        }
        let filtered_and_sorted = filter_and_sort_listeners(entries);

        let owners = match process::socket_owners() {
            Ok(map) => map,
            Err(err) => {
                eprintln!("could not read socket owners: {err}");
                std::process::exit(1);
            }
        };

        for entry in &filtered_and_sorted {
            let socket_address = SocketAddr::new(entry.local_address, entry.port);
            match owners.get(&entry.inode) {
                Some(pids) => println!("{}: {:?}", socket_address, pids),
                None => println!("{}: -", socket_address),
            }
        }
        println!("count {}", filtered_and_sorted.len());
    }
}
