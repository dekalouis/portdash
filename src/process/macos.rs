#![cfg(target_os = "macos")]
use std::net::IpAddr;

use crate::parse::LsofEntry;

/// One listening socket and the processes that hold it.
#[derive(PartialEq, Debug)]
pub struct LsofListener {
    /// Local IP address, IPv4 or IPv6.
    pub address: IpAddr,
    /// Local port.
    pub port: u16,
    /// PIDs holding the socket, each listed once, in the order `lsof` printed them.
    pub pids: Vec<u32>,
}

/// Turns one record per process per socket into one listener per socket,
/// keyed by socket ID, address and port, with the PIDs that own it.
pub fn group_into_listeners(entries: &[LsofEntry]) -> Vec<LsofListener> {
    let mut seen: Vec<((String, IpAddr, u16), LsofListener)> = Vec::new();
    for entry in entries {
        let key = (entry.socket_id.clone(), entry.address, entry.port);
        match seen.iter_mut().find(|(k, _)| *k == key) {
            Some((_, listener)) => {
                if !listener.pids.contains(&entry.pid) {
                    listener.pids.push(entry.pid);
                }
            }
            None => seen.push((
                key,
                LsofListener {
                    address: entry.address,
                    port: entry.port,
                    pids: vec![entry.pid],
                },
            )),
        }
    }
    seen.into_iter().map(|(_, listener)| listener).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    /// Builds one record, so each grouping test only states what it cares about.
    fn entry(pid: u32, socket_id: &str, address: IpAddr, port: u16) -> LsofEntry {
        LsofEntry {
            pid,
            parent_pid: 1,
            cmd_name: "test".into(),
            uid: 501,
            family: "IPv4".into(),
            socket_id: socket_id.into(),
            address,
            port,
        }
    }

    #[test]
    fn groups_two_pids_on_one_socket() {
        let local = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let listeners = group_into_listeners(&[
            entry(100, "0xA", local, 8767),
            entry(200, "0xA", local, 8767),
        ]);
        assert_eq!(
            listeners,
            [LsofListener {
                address: local,
                port: 8767,
                pids: vec![100, 200],
            }]
        );
    }

    #[test]
    fn lists_a_pid_once_for_two_descriptors_of_one_socket() {
        let local = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let listeners = group_into_listeners(&[
            entry(100, "0xA", local, 8767),
            entry(100, "0xA", local, 8767),
        ]);
        assert_eq!(
            listeners,
            [LsofListener {
                address: local,
                port: 8767,
                pids: vec![100],
            }]
        );
    }

    #[test]
    fn splits_two_sockets_of_one_pid_on_one_port() {
        let local = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let listeners = group_into_listeners(&[
            entry(100, "0xA", local, 8785),
            entry(100, "0xB", local, 8785),
        ]);
        assert_eq!(listeners.len(), 2);
        assert_eq!(listeners[0].pids, [100]);
        assert_eq!(listeners[1].pids, [100]);
    }

    #[test]
    fn splits_one_pid_on_two_ports() {
        let local = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let listeners = group_into_listeners(&[
            entry(100, "0xA", local, 8000),
            entry(100, "0xB", local, 8001),
        ]);
        assert_eq!(listeners.len(), 2);
        assert_eq!(listeners[0].port, 8000);
        assert_eq!(listeners[1].port, 8001);
    }

    #[test]
    fn splits_star_ipv4_from_star_ipv6() {
        let listeners = group_into_listeners(&[
            entry(100, "0xA", IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8000),
            entry(100, "0xB", IpAddr::V6(Ipv6Addr::UNSPECIFIED), 8000),
        ]);
        assert_eq!(listeners.len(), 2);
        assert_eq!(listeners[0].address, IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        assert_eq!(listeners[1].address, IpAddr::V6(Ipv6Addr::UNSPECIFIED));
    }
}
