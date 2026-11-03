use crate::parse::parse_socket;
#[cfg(target_os = "linux")]
use crate::sockets::read_entries;

#[cfg(target_os = "macos")]
use crate::sockets::run_lsof;

mod parse;
mod sockets;

fn main() {
    #[cfg(target_os = "macos")]
    match run_lsof() {
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
                // TODO: {:?} is development output. Replace it (Stage 8a).
                match parse_socket(line) {
                    Ok(socket) => {
                        println!("{:?}", socket);
                    }
                    Err(err) => {
                        eprintln!("{:?} {line}", err);
                    }
                }
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
