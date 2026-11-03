#[cfg(target_os = "macos")]
use crate::parse::parse_lsof;
#[cfg(target_os = "linux")]
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
        Ok(lsof_text) => {
            let socket_lines = lsof_text
                .lines()
                .filter(|line| line.starts_with('n'))
                .count();

            match parse_lsof(&lsof_text) {
                Ok(entries) => {
                    for entry in &entries {
                        // TODO: {:?} is development output. Replace it (Stage 8a).
                        println!("{:?}", entry);
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
    match read_entries() {
        Ok(entries) => {
            for line in &entries {
                match parse_socket(line) {
                    Ok(socket) => {
                        // TODO: {:?} is development output. Replace it (Stage 8a).
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
