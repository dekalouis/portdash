#[cfg(target_os = "linux")]
use crate::sockets::read_entries;
#[cfg(target_os = "macos")]
use crate::sockets::read_entries_mac;

mod sockets;

fn main() {
    #[cfg(target_os = "macos")]
    match read_entries_mac() {
        Ok(entries) => {
            let log = entries;
            for line in log.lines() {
                println!("{}", line)
            }
            let comments = log.lines().filter(|line| line.starts_with('n')).count();
            println!("{}", comments);
        }
        Err(err) => {
            eprintln!("could not read sockets: {err}");
            std::process::exit(1);
        }
    }

    #[cfg(target_os = "linux")]
    match read_entries() {
        Ok(entries) => {
            for line in &entries {
                println!("{}", line)
            }
            let comments = entries.len();
            println!("{}", comments);
        }
        Err(err) => {
            eprintln!("could not read sockets: {err}");
            std::process::exit(1);
        }
    }
}
