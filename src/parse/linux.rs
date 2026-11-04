#![cfg(target_os = "linux")]
use super::ParseError;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// One row of /proc/net/tcp or /proc/net/tcp6 with only the fields used
#[derive(PartialEq, Debug)]
pub struct TcpEntry {
    /// Local IP address, IPv4 or IPv6.
    pub local_address: IpAddr,
    /// Local port, converted from hex.
    pub port: u16,
    /// Connection state code, converted from hex (`0x0A` is LISTEN).
    pub state: u8,
    /// Owner's user ID. Decimal in the file.
    pub uid: u32,
    /// Socket inode. Decimal in the file.
    pub inode: u64,
}

/// Parses one data line of `/proc/net/tcp` or `/proc/net/tcp6` (not the header).
///
/// The address length decides the kind: 8 hex characters is IPv4, 32 is IPv6.
/// Fields are split on whitespace. The row number is skipped. Returns
/// `MissingField` if the line is too short, and `BadNumber` if a number is
/// malformed.
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
        local_address: parse_hex_address(address)?,
        port,
        state,
        uid,
        inode,
    })
}

fn parse_hex_address(address: &str) -> Result<IpAddr, ParseError> {
    match address.len() {
        8 => Ok(IpAddr::V4(parse_hex_to_ipv4(address)?)),
        32 => Ok(IpAddr::V6(parse_hex_to_ipv6(address)?)),
        _ => Err(ParseError::BadNumber {
            field: "address",
            text: address.to_string(),
        }),
    }
}

fn decode_group(address: &str) -> Result<[u8; 4], ParseError> {
    let raw = u32::from_str_radix(address, 16).map_err(|_| ParseError::BadNumber {
        field: "address",
        text: address.to_string(),
    })?;
    Ok(raw.to_ne_bytes())
}

fn parse_hex_to_ipv4(address: &str) -> Result<Ipv4Addr, ParseError> {
    let as_bytes = decode_group(address)?;
    Ok(Ipv4Addr::from(as_bytes))
}

fn parse_hex_to_ipv6(address: &str) -> Result<Ipv6Addr, ParseError> {
    if !address.is_ascii() || address.len() != 32 {
        return Err(ParseError::BadNumber {
            field: "address",
            text: address.to_string(),
        });
    }
    let mut bytes = Vec::new();
    for piece in address.as_bytes().chunks(8) {
        let piece_text = std::str::from_utf8(piece).map_err(|_| ParseError::BadNumber {
            field: "address",
            text: address.to_string(),
        })?;
        let group = decode_group(piece_text)?;
        bytes.extend(group);
    }
    let array: [u8; 16] = bytes.try_into().map_err(|_| ParseError::BadNumber {
        field: "address",
        text: address.to_string(),
    })?;
    Ok(Ipv6Addr::from(array))
}

pub fn text_to_entries(text: &str) -> Vec<TcpEntry> {
    text.lines()
        .skip(1)
        .filter_map(|line| parse_socket(line).ok())
        .collect()
}

pub fn filter_and_sort_listeners(list: Vec<TcpEntry>) -> Vec<TcpEntry> {
    let mut listeners: Vec<TcpEntry> = list
        .into_iter()
        .filter(|entry| entry.state == 0x0A)
        .collect();
    listeners.sort_by_key(|entry| entry.port);
    listeners
}

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
    fn parses_an_ipv6_port_line() {
        let tcp_entry = parse_socket("0: 00000000000000000000000001000000:18EB 00000000000000000000000000000000:0000 0A 00000000:00000000 00:00000000 00000000   955        0 24604 1 00000000ae4dee8f 100 0 0 10 0")
            .unwrap();
        assert_eq!(
            tcp_entry.local_address,
            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)
        );
        assert_eq!(tcp_entry.port, 6379);
        assert_eq!(tcp_entry.state, 10);
        assert_eq!(tcp_entry.uid, 955);
        assert_eq!(tcp_entry.inode, 24604);
    }

    #[test]
    fn parses_an_ipv6_postgres_line() {
        let tcp_entry = parse_socket("1: 00000000000000000000000001000000:1538 00000000000000000000000000000000:0000 0A 00000000:00000000 00:00000000 00000000   957        0 21286 1 0000000021bf0adb 100 0 0 10 0")
            .unwrap();
        assert_eq!(
            tcp_entry.local_address,
            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)
        );
        assert_eq!(tcp_entry.port, 5432);
        assert_eq!(tcp_entry.state, 10);
        assert_eq!(tcp_entry.uid, 957);
        assert_eq!(tcp_entry.inode, 21286);
    }

    /// Tests for the function parse_hex_to_ipv4
    #[test]
    fn ipv4_loopback() {
        let parsed_address = parse_hex_to_ipv4("0100007F").unwrap();
        assert_eq!(parsed_address, Ipv4Addr::new(127, 0, 0, 1));
    }

    #[test]
    fn ipv4_dns_stub() {
        let parsed_address = parse_hex_to_ipv4("3500007F").unwrap();
        assert_eq!(parsed_address, Ipv4Addr::new(127, 0, 0, 53));
    }

    #[test]
    fn ipv4_all_zeros() {
        let parsed_address = parse_hex_to_ipv4("00000000").unwrap();
        assert_eq!(parsed_address, Ipv4Addr::new(0, 0, 0, 0));
    }

    #[test]
    fn ipv4_rejects_non_hex() {
        assert_eq!(
            parse_hex_to_ipv4("ZZZZZZZZ"),
            Err(ParseError::BadNumber {
                field: "address",
                text: String::from("ZZZZZZZZ"),
            })
        );
    }

    /// Tests for the function parse_hex_to_ipv6
    #[test]
    fn ipv6_loopback() {
        let parsed_address = parse_hex_to_ipv6("00000000000000000000000001000000").unwrap();
        assert_eq!(parsed_address, Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
    }

    #[test]
    fn ipv6_all_zeros() {
        let parsed_address = parse_hex_to_ipv6("00000000000000000000000000000000").unwrap();
        assert_eq!(parsed_address, Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0));
    }

    #[test]
    fn ipv6_mapped_ipv4() {
        let parsed_address = parse_hex_to_ipv6("0000000000000000FFFF00000100007F").unwrap();
        assert_eq!(
            parsed_address,
            Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x7f00, 1)
        );
        assert_eq!(
            parsed_address,
            "::ffff:127.0.0.1".parse::<Ipv6Addr>().unwrap()
        );
    }

    #[test]
    fn ipv6_rejects_wrong_length() {
        assert_eq!(
            parse_hex_to_ipv6("00000"),
            Err(ParseError::BadNumber {
                field: "address",
                text: String::from("00000")
            })
        );
    }

    #[test]
    fn ipv6_rejects_non_hex() {
        assert_eq!(
            parse_hex_to_ipv6("ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ"),
            Err(ParseError::BadNumber {
                field: "address",
                text: String::from("ZZZZZZZZ")
            })
        );
    }

    /// Tests for the function parse_hex_address
    #[test]
    fn hex_address_of_8_characters_is_ipv4() {
        assert_eq!(
            parse_hex_address("0100007F"),
            Ok(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
        );
    }

    #[test]
    fn hex_address_of_32_characters_is_ipv6() {
        assert_eq!(
            parse_hex_address("00000000000000000000000001000000"),
            Ok(IpAddr::V6(Ipv6Addr::LOCALHOST))
        );
    }

    #[test]
    fn hex_address_rejects_other_lengths() {
        assert_eq!(
            parse_hex_address("0100007"),
            Err(ParseError::BadNumber {
                field: "address",
                text: String::from("0100007")
            })
        );
    }

    /// Tests for the function text_to_entries
    const HEADER: &str = "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode";
    const LISTEN: &str = "   0: 010011AC:0035 00000000:0000 0A 00000000:00000000 00:00000000 00000000   975        0 5590 1 000000004c1e4911 100 0 0 10 5";
    const ESTABLISHED: &str = "  13: 0100007F:D87C 0100007F:91FA 01 00000000:00000000 02:000007F7 00000000  1000        0 7997159 2 000000002ac490fd 20 4 31 10 -1";

    #[test]
    fn returns_every_line() {
        let text = format!("{HEADER}\n{LISTEN}\n{ESTABLISHED}\n");
        let entries = text_to_entries(&text);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].port, 53);
        assert_eq!(entries[1].port, 55420);
        assert_eq!(entries[1].state, 1);
    }

    #[test]
    fn skips_a_bad_line() {
        let text = format!("{HEADER}\n{LISTEN}\nthis is not a socket\n");
        let entries = text_to_entries(&text);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].port, 53);
    }

    #[test]
    fn empty_text() {
        assert!(text_to_entries("").is_empty());
    }

    /// Tests for the function filter_and_sort_listeners
    const LISTEN_HIGH_PORT: &str = "   1: 0100007F:2CAA 00000000:0000 0A 00000000:00000000 00:00000000 00000000   954        0 23570 2 0000000030222c1a 100 0 0 10 0";

    #[test]
    fn keeps_only_listen() {
        let text = format!("{HEADER}\n{ESTABLISHED}\n{LISTEN}\n");
        let listeners = filter_and_sort_listeners(text_to_entries(&text));
        assert_eq!(listeners.len(), 1);
        assert_eq!(listeners[0].port, 53);
        assert_eq!(listeners[0].state, 10);
    }

    #[test]
    fn sort_by_port() {
        let text = format!("{HEADER}\n{LISTEN_HIGH_PORT}\n{LISTEN}\n");
        let ports: Vec<u16> = filter_and_sort_listeners(text_to_entries(&text))
            .iter()
            .map(|entry| entry.port)
            .collect();
        assert_eq!(ports, [53, 11434]);
    }
}
