use crate::kill::kill_port;
#[cfg(target_os = "macos")]
use crate::parse::parse_lsof;
#[cfg(target_os = "macos")]
use crate::process::group_into_listeners;
#[cfg(target_os = "linux")]
use crate::scan::scan;
#[cfg(target_os = "macos")]
use crate::sockets::run_lsof;
use clap::{Parser, Subcommand};
use std::io::{self, Write};
#[cfg(target_os = "macos")]
use std::net::SocketAddr;

mod kill;
mod parse;
mod process;
mod project;
mod scan;
mod sockets;

#[derive(Parser)]
#[command(version, about = "Show which dev servers are listening on which ports")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// List listening ports
    List {
        /// Print as JSON
        #[arg(long)]
        json: bool,
    },
    /// Stop whatever is listening on a port: SIGTERM first, SIGKILL if it is still alive
    Kill {
        /// The port to free
        port: u16,
        /// Skip SIGTERM and send SIGKILL right away
        #[arg(long)]
        force: bool,
        /// Do not ask for confirmation
        #[arg(long)]
        yes: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Some(Command::List { json }) => print_list(json),
        Some(Command::Kill { port, force, yes }) => kill_port(port, force, yes),
        None => print_list(false),
    };

    if let Err(err) = result {
        if err.kind() == io::ErrorKind::BrokenPipe {
            return;
        }
        eprintln!("could not write output: {err}");
        std::process::exit(1);
    }
}

#[cfg(target_os = "macos")]
fn print_list(_json: bool) -> io::Result<()> {
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
    Ok(())
}

#[cfg(target_os = "linux")]
fn print_list(json: bool) -> io::Result<()> {
    let listeners = match scan() {
        Ok(list) => list,
        Err(err) => {
            eprintln!("could not read sockets: {err}");
            std::process::exit(1);
        }
    };

    let mut out = io::stdout().lock();

    if json {
        match serde_json::to_string_pretty(&listeners) {
            Ok(text) => writeln!(out, "{text}")?,
            Err(err) => {
                eprintln!("could not write JSON: {err}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    if listeners.is_empty() {
        writeln!(out, "nothing is listening!")?;
        return Ok(());
    }

    let mut rows: Vec<[String; 5]> = Vec::new();

    for listener in &listeners {
        let pid = match listener.owners.first() {
            None => "-".to_string(),
            Some(first) => {
                let others = listener.owners.len() - 1;
                if others == 0 {
                    first.pid.to_string()
                } else {
                    format!("{} +{}", first.pid, others)
                }
            }
        };

        let process = match listener.owners.first() {
            None => "-".to_string(),
            Some(first) => first.name.clone(),
        };

        let project = listener.project.as_deref().unwrap_or("-").to_string();

        rows.push([
            listener.port.to_string(),
            listener.address.to_string(),
            pid,
            process,
            project,
        ]);
    }

    let header = ["PORT", "ADDRESS", "PID", "PROCESS", "PROJECT"].map(String::from);

    let mut widths = header.clone().map(|text| text.len());
    for row in &rows {
        for (column, text) in row.iter().enumerate() {
            // chars().count() matches how format! pads, so accented names like "café" line up.
            // TODO: wide characters (CJK, emoji) take two screen columns each and still misalign.
            // Fix with the unicode-width crate: text.width() here, and pad by hand with " ".repeat(width - text.width()).
            widths[column] = widths[column].max(text.chars().count());
        }
    }

    for row in std::iter::once(&header).chain(&rows) {
        let line: Vec<String> = row
            .iter()
            .zip(widths)
            .map(|(text, width)| format!("{text:<width$}"))
            .collect();
        writeln!(out, "{}", line.join("  ").trim_end())?;
    }

    let unknown = listeners
        .iter()
        .filter(|listener| listener.owners.is_empty())
        .count();
    if unknown == 1 {
        eprintln!("1 port has an unknown owner (try sudo)");
    } else if unknown > 1 {
        eprintln!("{} ports have unknown owners (try sudo)", unknown);
    }
    Ok(())
}
