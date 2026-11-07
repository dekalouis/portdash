#![cfg(target_os = "linux")]
use std::{collections::HashMap, fs, io};

use super::Listener;
use crate::parse::TcpEntry;

/// Pairs each socket with the PIDs that hold it, using one scan of `/proc`.
pub fn attach_owners(entries: &[TcpEntry]) -> io::Result<Vec<Listener>> {
    let owners = socket_owners()?;
    Ok(entries
        .iter()
        .map(|entry| Listener {
            address: entry.address,
            port: entry.port,
            pids: owners.get(&entry.inode).cloned().unwrap_or_default(),
        })
        .collect())
}

/// Scans `/proc/<pid>/fd` and maps each socket inode to the PIDs holding it.
///
/// Processes that cannot be read (other users, or already exited) are skipped,
/// so their sockets simply have no entry. Fails only if `/proc` itself is unreadable.
fn socket_owners() -> io::Result<HashMap<u64, Vec<u32>>> {
    let mut found: HashMap<u64, Vec<u32>> = HashMap::new();

    for entry in fs::read_dir("/proc")? {
        // One unreadable entry should not stop the whole scan.
        let Ok(entry) = entry else { continue };
        let name = entry.file_name();
        let Some(text) = name.to_str() else { continue };

        let Ok(pid) = text.parse::<u32>() else {
            continue;
        };

        for inode in socket_inodes(pid) {
            let list = found.entry(inode).or_default();
            if !list.contains(&pid) {
                list.push(pid);
            }
        }
    }
    Ok(found)
}

/// Inodes of the sockets one process holds. Empty if its `fd` folder cannot be read.
fn socket_inodes(pid: u32) -> Vec<u64> {
    let mut inodes = Vec::new();
    let Ok(fd_dir) = fs::read_dir(format!("/proc/{pid}/fd")) else {
        return inodes;
    };

    for fd_entry in fd_dir {
        let Ok(fd_entry) = fd_entry else { continue };
        let Ok(target) = fs::read_link(fd_entry.path()) else {
            continue;
        };
        let Some(target) = target.to_str() else {
            continue;
        };
        if let Some(inode) = socket_inode(target) {
            inodes.push(inode);
        }
    }
    inodes
}

fn socket_inode(link_target: &str) -> Option<u64> {
    let rest = link_target.strip_prefix("socket:[")?;
    let digits = rest.strip_suffix("]")?;
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{net::TcpListener, os::fd::AsRawFd};

    #[test]
    fn parses_socket_inode() {
        assert_eq!(socket_inode("socket:[123]"), Some(123));
    }

    #[test]
    fn rejects_dev_null() {
        assert_eq!(socket_inode("/dev/null"), None);
    }

    #[test]
    fn rejects_non_numeric_inode() {
        assert_eq!(socket_inode("socket:[abc]"), None);
    }

    #[test]
    fn rejects_a_pipe() {
        assert_eq!(socket_inode("pipe:[123]"), None);
    }

    #[test]
    fn rejects_missing_closing_bracket() {
        assert_eq!(socket_inode("socket:[123"), None);
    }

    #[test]
    fn rejects_empty_inode() {
        assert_eq!(socket_inode("socket:[]"), None);
    }

    /// Inode of a socket this process holds, read from its own `/proc/self/fd`.
    fn inode_of(listener: &TcpListener) -> u64 {
        let link = fs::read_link(format!("/proc/self/fd/{}", listener.as_raw_fd())).unwrap();
        socket_inode(link.to_str().unwrap()).unwrap()
    }

    #[test]
    fn finds_our_own_socket() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let owners = socket_owners().unwrap();
        assert_eq!(owners[&inode_of(&listener)], [std::process::id()]);
    }

    // try_clone duplicates the descriptor, so one socket sits on two fds of one process
    #[test]
    fn lists_a_pid_once_for_two_descriptors() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let _copy = listener.try_clone().unwrap();
        let owners = socket_owners().unwrap();
        assert_eq!(owners[&inode_of(&listener)], [std::process::id()]);
    }

    // u64::MAX stands in for a socket whose owner cannot be read
    #[test]
    fn attaches_owners_and_leaves_unknown_empty() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let local = listener.local_addr().unwrap();
        let entry = |inode| TcpEntry {
            address: local.ip(),
            port: local.port(),
            state: 0x0A,
            uid: 0,
            inode,
        };

        let listeners = attach_owners(&[entry(inode_of(&listener)), entry(u64::MAX)]).unwrap();
        assert_eq!(listeners[0].pids, [std::process::id()]);
        assert!(listeners[1].pids.is_empty());
    }
}
