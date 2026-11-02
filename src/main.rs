use crate::sockets::read_entries_mac;

mod sockets;

fn main() {
    match read_entries_mac() {
        Ok(entries) => {
            // println!("{}", entries);
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
}
