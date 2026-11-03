use std::net::Ipv4Addr;

/// One row of /proc/net/tcp with only the fields used
#[cfg(target_os = "linux")]
#[derive(PartialEq, Debug)]
pub struct TcpEntry {
    /// Local IP address as the raw hex text from the file (decoded in Stage 4).
    pub local_address: Ipv4Addr,
    /// Local port, converted from hex.
    pub port: u16,
    /// Connection state code, converted from hex (`0x0A` is LISTEN).
    pub state: u8,
    /// Owner's user ID. Decimal in the file.
    pub uid: u32,
    /// Socket inode. Decimal in the file.
    pub inode: u64,
}

#[cfg(target_os = "macos")]
#[derive(PartialEq, Debug)]
pub struct LsofEntry {
    pub pid: u32,
    pub parent_pid: u32,
    pub cmd_name: String,
    pub uid: u32,
    pub family: String,
    pub socket_id: String,
    pub raw_address: String,
    pub port: u16,
}

// TODO: implement `Display` for readable messages (Stage 8a).
#[derive(PartialEq, Debug)]
pub enum ParseError {
    MissingField(&'static str),
    BadNumber { field: &'static str, text: String },
}

/// Parses one data line of `/proc/net/tcp` (not the header).
///
/// Fields are split on whitespace. The row number is skipped. Returns
/// `MissingField` if the line is too short, and `BadNumber` if a number is
/// malformed.
#[cfg(target_os = "linux")]
pub fn parse_socket(line: &str) -> Result<TcpEntry, ParseError> {
    let fields: Vec<&str> = line.split_whitespace().collect();

    let raw_address = fields.get(1).ok_or(ParseError::MissingField("address"))?;
    let (address, raw_port) = raw_address
        .split_once(":")
        .ok_or(ParseError::MissingField("port"))?;
    let port = u16::from_str_radix(raw_port, 16).map_err(|_| ParseError::BadNumber {
        field: "port",
        text: raw_port.to_string(),
    })?;

    let raw_state = fields.get(3).ok_or(ParseError::MissingField("state"))?;
    let state = u8::from_str_radix(raw_state, 16).map_err(|_| ParseError::BadNumber {
        field: "state",
        text: raw_state.to_string(),
    })?;

    let raw_uid = fields.get(7).ok_or(ParseError::MissingField("uid"))?;
    let uid: u32 = raw_uid.parse().map_err(|_| ParseError::BadNumber {
        field: "uid",
        text: raw_uid.to_string(),
    })?;

    let raw_inode = fields.get(9).ok_or(ParseError::MissingField("inode"))?;
    let inode: u64 = raw_inode.parse().map_err(|_| ParseError::BadNumber {
        field: "inode",
        text: raw_inode.to_string(),
    })?;

    Ok(TcpEntry {
        local_address: parse_hex_to_ipv4(address)?,
        port,
        state,
        uid,
        inode,
    })
}

fn parse_hex_to_ipv4(address: &str) -> Result<Ipv4Addr, ParseError> {
    let raw = u32::from_str_radix(address, 16).map_err(|_| ParseError::BadNumber {
        field: "address",
        text: address.to_string(),
    })?;
    Ok(Ipv4Addr::from(raw.to_ne_bytes()))
}

pub fn text_to_entries(text: &str) -> Vec<TcpEntry> {
    let split = text.lines().skip(1);
    let mut res = Vec::new();
    for line in split {
        let parsed = parse_socket(line);
        if let Ok(entry) = parsed {
            res.push(entry);
        }
    }
    res
}

pub fn filter_listeners(list: Vec<TcpEntry>) -> Vec<TcpEntry> {
    let mut listeners: Vec<TcpEntry> = list
        .into_iter()
        .filter(|entry| entry.state == 0x0A)
        .collect();
    listeners.sort_by_key(|entry| entry.port);
    listeners
}

#[cfg(target_os = "macos")]
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
                let (address, port) = value
                    .rsplit_once(':')
                    .ok_or(ParseError::MissingField("port"))?;

                let port_number: u16 = port.parse().map_err(|_| ParseError::BadNumber {
                    field: "port",
                    text: port.to_string(),
                })?;

                entries.push(LsofEntry {
                    pid: pid.ok_or(ParseError::MissingField("pid"))?,
                    parent_pid: parent_pid.ok_or(ParseError::MissingField("parent_pid"))?,
                    cmd_name: cmd_name.clone(),
                    uid: uid.ok_or(ParseError::MissingField("uid"))?,
                    family: family.clone(),
                    socket_id: socket_id.clone(),
                    raw_address: address.to_string(),
                    port: port_number,
                });
            }
            _ => {}
        }
    }
    Ok(entries)
}

#[cfg(target_os = "macos")]
fn parse_u32(field: &'static str, value: &str) -> Result<u32, ParseError> {
    value.parse().map_err(|_| ParseError::BadNumber {
        field,
        text: value.to_string(),
    })
}

#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {
    use super::*;

    const GOOD_LINE: &str = "1: 0100007F:C0DE 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 492943 1 0000000015983799 100 0 0 10 0";

    #[test]
    fn parses_a_good_line() {
        let tcp_entry = parse_socket(GOOD_LINE).unwrap();
        assert_eq!(tcp_entry.local_address, Ipv4Addr::new(127, 0, 0, 1));
        assert_eq!(tcp_entry.port, 49374);
        assert_eq!(tcp_entry.state, 10);
        assert_eq!(tcp_entry.uid, 1000);
        assert_eq!(tcp_entry.inode, 492943);
    }

    #[test]
    fn parses_a_second_line() {
        let tcp_entry = parse_socket("0: 3500007F:0035 00000000:0000 0A 00000000:00000000 00:00000000 00000000   975        0 15411 1 0000000083f5dc00 100 0 0 10 5").unwrap();
        assert_eq!(tcp_entry.local_address, Ipv4Addr::new(127, 0, 0, 53));
        assert_eq!(tcp_entry.port, 53);
        assert_eq!(tcp_entry.state, 10);
    }

    #[test]
    fn rejects_a_short_line() {
        assert_eq!(
            parse_socket("1: 0100007F:C0DE 00000000:0000"),
            Err(ParseError::MissingField("state"))
        );
    }

    #[test]
    fn rejects_a_bad_port() {
        assert_eq!(
            parse_socket(
                "1: 0100007F:ZZZZ 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 492943 1 0000000015983799 100 0 0 10 0"
            ),
            Err(ParseError::BadNumber {
                field: "port",
                text: String::from("ZZZZ")
            })
        );
    }

    #[test]
    fn rejects_empty_string() {
        assert_eq!(parse_socket(""), Err(ParseError::MissingField("address")));
    }

    #[test]
    fn rejects_bad_state() {
        assert_eq!(
            parse_socket(
                "1: 0100007F:C0DE 00000000:0000 0Z 00000000:00000000 00:00000000 00000000  1000        0 492943 1 0000000015983799 100 0 0 10 0"
            ),
            Err(ParseError::BadNumber {
                field: "state",
                text: String::from("0Z")
            })
        );
    }

    #[test]
    fn rejects_bad_number_uid() {
        assert_eq!(
            parse_socket(
                "1: 0100007F:C0DE 00000000:0000 0A 00000000:00000000 00:00000000 00000000  abc        0 492943 1 0000000015983799 100 0 0 10 0"
            ),
            Err(ParseError::BadNumber {
                field: "uid",
                text: String::from("abc")
            })
        );
    }

    // inode is decimal, so a hex digit in it must fail instead of being read as hex
    #[test]
    fn rejects_mixed_bases_inode() {
        assert_eq!(
            parse_socket(
                "1: 0100007F:C0DE 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 49A943 1 0000000015983799 100 0 0 10 0"
            ),
            Err(ParseError::BadNumber {
                field: "inode",
                text: String::from("49A943")
            })
        );
    }

    #[test]
    fn rejects_missing_colon() {
        assert_eq!(
            parse_socket(
                "1: 0100007FC0DE 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 492943 1 0000000015983799 100 0 0 10 0"
            ),
            Err(ParseError::MissingField("port"))
        );
    }

    #[test]
    fn parse_ip_address_1() {
        let parsed_address = parse_hex_to_ipv4("0100007F").unwrap();
        assert_eq!(parsed_address, Ipv4Addr::new(127, 0, 0, 1));
    }

    #[test]
    fn parse_ip_address_2() {
        let parsed_address = parse_hex_to_ipv4("3500007F").unwrap();
        assert_eq!(parsed_address, Ipv4Addr::new(127, 0, 0, 53));
    }

    #[test]
    fn parse_ip_address_3() {
        let parsed_address = parse_hex_to_ipv4("00000000").unwrap();
        assert_eq!(parsed_address, Ipv4Addr::new(0, 0, 0, 0));
    }

    #[test]
    fn parse_ip_address_fail() {
        assert_eq!(
            parse_hex_to_ipv4("ZZZZZZZZ"),
            Err(ParseError::BadNumber {
                field: "address",
                text: String::from("ZZZZZZZZ"),
            })
        );
    }
}

#[cfg(target_os = "macos")]
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
                raw_address: "127.0.0.1".into(),
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
                raw_address: "127.0.0.1".into(),
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
                raw_address: "[::1]".into(),
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
    fn rejects_a_bad_port() {
        let result = parse_lsof(
            "p28161\n\
             R1\n\
             copencode\n\
             u501\n\
             f7\n\
             tIPv4\n\
             d0x28282d27564c8a55\n\
             n127.0.0.1:ZZZZ\n",
        );
        assert_eq!(
            result,
            Err(ParseError::BadNumber {
                field: "port",
                text: String::from("ZZZZ")
            })
        );
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
}
