#[cfg(target_os = "macos")]
use crate::parse::parse_lsof;
#[cfg(target_os = "macos")]
use crate::process::group_into_listeners;
#[cfg(target_os = "linux")]
use crate::scan::scan;
#[cfg(target_os = "macos")]
use crate::sockets::run_lsof;
use std::net::SocketAddr;

mod parse;
mod process;
mod project;
mod scan;
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
                eprintln!("could not parse sockets: {err}");
                std::process::exit(1);
            }
        };

        let mut listeners = group_into_listeners(&entries);
        listeners.sort_by_key(|listener| listener.port);

        for listener in &listeners {
            let socket_address = SocketAddr::new(listener.address, listener.port);
            // TODO: the PID list is development output. Stage 9 replaces it with a table.
            println!("{}: {:?}", socket_address, listener.pids);
        }
        println!("count {}, actual {}", socket_lines, entries.len());
    }

    #[cfg(target_os = "linux")]
    {
        let listeners = match scan() {
            Ok(list) => list,
            Err(err) => {
                eprintln!("could not read sockets: {err}");
                std::process::exit(1);
            }
        };

        for listener in &listeners {
            let socket_address = SocketAddr::new(listener.address, listener.port);

            let pids: Vec<u32> = listener.owners.iter().map(|owner| owner.pid).collect();
            if pids.is_empty() {
                println!("{}: -", socket_address);
            } else {
                println!("{}: {:?}", socket_address, pids);
            }

            let project = listener.project.as_deref().unwrap_or("-");

            for owner in &listener.owners {
                let cwd = match &owner.cwd {
                    Some(path) => path.display().to_string(),
                    None => "-".to_string(),
                };

                println!(
                    "   {} {} {} {} {}",
                    owner.pid, owner.name, owner.command, cwd, project
                );
            }
        }
        println!("count {}", listeners.len());
    }
}
