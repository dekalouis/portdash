#![cfg(target_os = "macos")]
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::ParseError;

#[derive(PartialEq, Debug)]
pub struct LsofEntry {
    pub pid: u32,
    pub parent_pid: u32,
    pub cmd_name: String,
    pub uid: u32,
    pub family: String,
    pub socket_id: String,
    pub raw_address: IpAddr,
    pub port: u16,
}

#[derive(PartialEq, Debug)]
pub struct LsofListener {
    pub address: IpAddr,
    pub port: u16,
    pub pids: Vec<u32>,
}

pub fn parse_lsof(text: &str) -> Result<Vec<LsofEntry>, ParseError> {
    let mut pid: Option<u32> = None;
    let mut parent_pid: Option<u32> = None;
    let mut cmd_name = String::new();
    let mut uid: Option<u32> = None;
    let mut family = String::new();
    let mut socket_id = String::new();
    let mut entries = Vec::new();

    for line in text.lines() {
        let mut chars = line.chars();
        let Some(tag) = chars.next() else { continue };
        let value = chars.as_str();
        match tag {
            'p' => pid = Some(parse_u32("pid", value)?),
            'R' => parent_pid = Some(parse_u32("parent_pid", value)?),
            'c' => cmd_name = value.to_string(),
            'u' => uid = Some(parse_u32("uid", value)?),
            'f' => {
                family = String::new();
                socket_id = String::new();
            }
            't' => family = value.to_string(),
            'd' => socket_id = value.to_string(),
            'n' => {
                // One odd socket should not lose the whole list: warn and move on.
                let (raw_address, port) = match parse_endpoint(value, &family) {
                    Ok(endpoint) => endpoint,
                    Err(err) => {
                        eprintln!("skipping {value}: {err:?}");
                        continue;
                    }
                };

                entries.push(LsofEntry {
                    pid: pid.ok_or(ParseError::MissingField("pid"))?,
                    parent_pid: parent_pid.ok_or(ParseError::MissingField("parent_pid"))?,
                    cmd_name: cmd_name.clone(),
                    uid: uid.ok_or(ParseError::MissingField("uid"))?,
                    family: family.clone(),
                    socket_id: socket_id.clone(),
                    raw_address,
                    port,
                });
            }
            _ => {}
        }
    }
    Ok(entries)
}

fn parse_u32(field: &'static str, value: &str) -> Result<u32, ParseError> {
    value.parse().map_err(|_| ParseError::BadNumber {
        field,
        text: value.to_string(),
    })
}

/// Splits an `n` value like `[::1]:8772` at the last `:` into address and port.
fn parse_endpoint(value: &str, family: &str) -> Result<(IpAddr, u16), ParseError> {
    let (address, port) = value
        .rsplit_once(':')
        .ok_or(ParseError::MissingField("port"))?;
    let port = port.parse().map_err(|_| ParseError::BadNumber {
        field: "port",
        text: port.to_string(),
    })?;
    Ok((parse_lsof_address(address, family)?, port))
}

fn parse_lsof_address(text: &str, family: &str) -> Result<IpAddr, ParseError> {
    if text == "*" {
        match family {
            "IPv4" => Ok(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            "IPv6" => Ok(IpAddr::V6(Ipv6Addr::UNSPECIFIED)),
            _ => Err(ParseError::BadNumber {
                field: "address",
                text: family.to_string(),
            }),
        }
    } else {
        let trimmed = match text.strip_prefix("[") {
            Some(rest) => rest
                .strip_suffix("]")
                .ok_or_else(|| ParseError::BadNumber {
                    field: "address",
                    text: text.to_string(),
                })?,
            None => text,
        };
        trimmed.parse().map_err(|_| ParseError::BadNumber {
            field: "address",
            text: text.to_string(),
        })
    }
}

/// Turns one record per process per socket into one listener per socket,
/// keyed by socket ID, address and port, with the PIDs that own it.
pub fn group_lsof_entry(lsof_entry: &[LsofEntry]) -> Vec<LsofListener> {
    let mut seen: Vec<((String, IpAddr, u16), LsofListener)> = Vec::new();
    for entry in lsof_entry {
        let key = (entry.socket_id.clone(), entry.raw_address, entry.port);
        match seen.iter_mut().find(|(k, _)| *k == key) {
            Some((_, listener)) => {
                if !listener.pids.contains(&entry.pid) {
                    listener.pids.push(entry.pid);
                }
            }
            None => seen.push((
                key,
                LsofListener {
                    address: entry.raw_address,
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

    const ONE_SOCKET: &str = "p28161\n\
        R1\n\
        copencode\n\
        u501\n\
        f7\n\
        tIPv4\n\
        d0x28282d27564c8a55\n\
        n127.0.0.1:49374\n";

    /// Tests for the function parse_lsof
    #[test]
    fn parses_a_good_line() {
        let lsof_entry = parse_lsof(ONE_SOCKET).unwrap();
        assert_eq!(lsof_entry.len(), 1);
        assert_eq!(
            lsof_entry[0],
            LsofEntry {
                pid: 28161,
                parent_pid: 1,
                cmd_name: "opencode".into(),
                uid: 501,
                family: "IPv4".into(),
                socket_id: "0x28282d27564c8a55".into(),
                raw_address: IpAddr::V4(Ipv4Addr::LOCALHOST),
                port: 49374,
            }
        );
    }

    #[test]
    fn parses_two_sockets_for_one_process() {
        let lsof_entry = parse_lsof(
            "p1593\n\
             R1\n\
             credis-server\n\
             u501\n\
             f6\n\
             tIPv4\n\
             d0xce9c9535e69391e9\n\
             n127.0.0.1:6379\n\
             f7\n\
             tIPv6\n\
             d0xfdf7acbc3acbc9c6\n\
             n[::1]:6379\n",
        )
        .unwrap();
        assert_eq!(lsof_entry.len(), 2);
        assert_eq!(
            lsof_entry[0],
            LsofEntry {
                pid: 1593,
                parent_pid: 1,
                cmd_name: "redis-server".into(),
                uid: 501,
                family: "IPv4".into(),
                socket_id: "0xce9c9535e69391e9".into(),
                raw_address: IpAddr::V4(Ipv4Addr::LOCALHOST),
                port: 6379,
            }
        );
        assert_eq!(
            lsof_entry[1],
            LsofEntry {
                pid: 1593,
                parent_pid: 1,
                cmd_name: "redis-server".into(),
                uid: 501,
                family: "IPv6".into(),
                socket_id: "0xfdf7acbc3acbc9c6".into(),
                raw_address: IpAddr::V6(Ipv6Addr::LOCALHOST),
                port: 6379,
            }
        );
    }

    #[test]
    fn does_not_leak_fields_between_processes() {
        let entries = parse_lsof(
            "p1593\n\
             R1\n\
             credis-server\n\
             u501\n\
             f6\n\
             tIPv4\n\
             d0xce9c9535e69391e9\n\
             n127.0.0.1:6379\n\
             f7\n\
             tIPv6\n\
             d0xfdf7acbc3acbc9c6\n\
             n[::1]:6379\n\
             p28161\n\
             R1\n\
             copencode\n\
             u501\n\
             f7\n\
             tIPv4\n\
             d0x28282d27564c8a55\n\
             n127.0.0.1:49374\n",
        )
        .unwrap();
        assert_eq!(entries.len(), 3);
        let opencode = &entries[2];
        assert_eq!(opencode.pid, 28161);
        assert_eq!(opencode.cmd_name, "opencode");
        assert_eq!(opencode.family, "IPv4");
        assert_eq!(opencode.socket_id, "0x28282d27564c8a55");
        assert_eq!(opencode.port, 49374);
    }

    #[test]
    fn keeps_a_name_with_a_space() {
        let entries = parse_lsof(
            "p28161\n\
             R1\n\
             cLM Studio\n\
             u501\n\
             f7\n\
             tIPv4\n\
             d0x28282d27564c8a55\n\
             n127.0.0.1:49374\n",
        )
        .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].cmd_name, "LM Studio");
    }

    #[test]
    fn skips_a_bad_socket_and_keeps_the_rest() {
        let entries = parse_lsof(
            "p28161\n\
             R1\n\
             copencode\n\
             u501\n\
             f7\n\
             tIPv4\n\
             d0x28282d27564c8a55\n\
             n127.0.0.1:ZZZZ\n\
             f8\n\
             tIPv4\n\
             d0x28282d27564c8a56\n\
             n127.0.0.1:49374\n",
        )
        .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].port, 49374);
    }

    #[test]
    fn skips_a_bad_address() {
        let entries = parse_lsof(
            "p28161\n\
             R1\n\
             copencode\n\
             u501\n\
             f7\n\
             tIPv4\n\
             d0x28282d27564c8a55\n\
             nnot-an-address:49374\n",
        )
        .unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn returns_empty_list_for_empty_string() {
        let entries = parse_lsof("").unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn rejects_a_socket_with_no_process() {
        let result = parse_lsof(
            "f7\n\
             tIPv4\n\
             d0x28282d27564c8a55\n\
             n127.0.0.1:49374\n",
        );
        assert_eq!(result, Err(ParseError::MissingField("pid")));
    }

    /// Tests for the function parse_endpoint
    #[test]
    fn rejects_a_bad_port() {
        assert_eq!(
            parse_endpoint("127.0.0.1:ZZZZ", "IPv4"),
            Err(ParseError::BadNumber {
                field: "port",
                text: String::from("ZZZZ")
            })
        );
    }

    #[test]
    fn parses_a_bracketed_ipv6_endpoint() {
        assert_eq!(
            parse_endpoint("[::1]:8766", "IPv6"),
            Ok((IpAddr::V6(Ipv6Addr::LOCALHOST), 8766))
        );
    }

    #[test]
    fn parses_a_star_ipv6_endpoint() {
        assert_eq!(
            parse_endpoint("*:8000", "IPv6"),
            Ok((IpAddr::V6(Ipv6Addr::UNSPECIFIED), 8000))
        );
    }

    #[test]
    fn parses_ipv4_text_under_ipv6_family_endpoint() {
        assert_eq!(
            parse_endpoint("127.0.0.1:8783", "IPv6"),
            Ok((IpAddr::V4(Ipv4Addr::LOCALHOST), 8783))
        );
    }

    #[test]
    fn rejects_a_bad_port_on_an_ipv6_endpoint() {
        assert_eq!(
            parse_endpoint("[::1]:ZZZZ", "IPv6"),
            Err(ParseError::BadNumber {
                field: "port",
                text: String::from("ZZZZ")
            })
        );
    }

    /// Tests for the function parse_lsof_address
    #[test]
    fn parses_a_plain_address() {
        assert_eq!(
            parse_lsof_address("127.0.0.1", "IPv4"),
            Ok(IpAddr::V4(Ipv4Addr::LOCALHOST))
        );
    }

    #[test]
    fn star_with_ipv4_family_is_unspecified() {
        assert_eq!(
            parse_lsof_address("*", "IPv4"),
            Ok(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
        );
    }

    #[test]
    fn rejects_a_bad_address() {
        assert_eq!(
            parse_lsof_address("not-an-address", "IPv4"),
            Err(ParseError::BadNumber {
                field: "address",
                text: String::from("not-an-address")
            })
        );
    }

    #[test]
    fn star_with_ipv6_family_is_unspecified() {
        assert_eq!(
            parse_lsof_address("*", "IPv6"),
            Ok(IpAddr::V6(Ipv6Addr::UNSPECIFIED))
        );
    }

    #[test]
    fn rejects_unknown_family() {
        assert_eq!(
            parse_lsof_address("*", "IPv5"),
            Err(ParseError::BadNumber {
                field: "address",
                text: String::from("IPv5")
            })
        );
    }

    #[test]
    fn parses_a_non_star_address() {
        assert_eq!(
            parse_lsof_address("[::1]", "IPv6"),
            Ok(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)))
        );
    }

    #[test]
    fn ipv4_text_under_ipv6_family_still_parses() {
        assert_eq!(
            parse_lsof_address("127.0.0.1", "IPv6"),
            Ok(IpAddr::V4(Ipv4Addr::LOCALHOST))
        );
    }

    #[test]
    fn parses_an_ipv6_address_without_brackets() {
        assert_eq!(
            parse_lsof_address("::1", "IPv6"),
            Ok(IpAddr::V6(Ipv6Addr::LOCALHOST))
        );
    }

    #[test]
    fn rejects_missing_bracket() {
        assert_eq!(
            parse_lsof_address("[::1", "IPv6"),
            Err(ParseError::BadNumber {
                field: "address",
                text: String::from("[::1")
            })
        );
    }

    /// Builds one record, so each grouping test only states what it cares about.
    fn entry(pid: u32, socket_id: &str, address: IpAddr, port: u16) -> LsofEntry {
        LsofEntry {
            pid,
            parent_pid: 1,
            cmd_name: "test".into(),
            uid: 501,
            family: "IPv4".into(),
            socket_id: socket_id.into(),
            raw_address: address,
            port,
        }
    }

    /// Tests for the function group_lsof_entry
    #[test]
    fn groups_two_pids_on_one_socket() {
        let local = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let listeners = group_lsof_entry(&[
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
        let listeners = group_lsof_entry(&[
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
        let listeners = group_lsof_entry(&[
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
        let listeners = group_lsof_entry(&[
            entry(100, "0xA", local, 8000),
            entry(100, "0xB", local, 8001),
        ]);
        assert_eq!(listeners.len(), 2);
        assert_eq!(listeners[0].port, 8000);
        assert_eq!(listeners[1].port, 8001);
    }

    #[test]
    fn splits_star_ipv4_from_star_ipv6() {
        let listeners = group_lsof_entry(&[
            entry(100, "0xA", IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8000),
            entry(100, "0xB", IpAddr::V6(Ipv6Addr::UNSPECIFIED), 8000),
        ]);
        assert_eq!(listeners.len(), 2);
        assert_eq!(listeners[0].address, IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        assert_eq!(listeners[1].address, IpAddr::V6(Ipv6Addr::UNSPECIFIED));
    }
}
