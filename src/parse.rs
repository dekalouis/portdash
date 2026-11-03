/// One row of /proc/net/tcp with only the fields used
#[cfg(target_os = "linux")]
#[derive(PartialEq, Debug)]
pub struct TcpEntry {
    /// Local IP address as the raw hex text from the file (decoded in Stage 4).
    pub local_address: String,
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
        local_address: address.to_string(),
        port,
        state,
        uid,
        inode,
    })
}

#[cfg(target_os = "macos")]
pub fn parse_lsof(text: &str) -> Result<Vec<LsofEntry>, String> {
    let mut pid = String::new();
    let mut parent_pid = String::new();
    let mut cmd_name = String::new();
    let mut uid = String::new();
    let mut family = String::new();
    let mut socket_id = String::new();
    let mut raw_address = String::new();
    let mut entries = Vec::new();

    for line in text.lines() {
        let mut chars = line.chars();
        let Some(tag) = chars.next() else { continue };
        let value = chars.as_str();
        match tag {
            'p' => {
                pid = value.parse().map_err(|_| ParseError::BadNumber {
                    field: "pid",
                    text: pid.to_string(),
                })?
            }
            'R' => parent_pid = value.to_string(),
            'c' => cmd_name = value.to_string(),
            'u' => uid = value.to_string(),
            'f' => {
                family = String::new();
                socket_id = String::new();
                raw_address = String::new();
            }
            't' => family = value.to_string(),
            'd' => socket_id = value.to_string(),
            'n' => raw_address = value.to_string(),
            _ => {}
        }
    }
    Ok(entries)
}

#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {
    use super::*;

    const GOOD_LINE: &str = "1: 0100007F:C0DE 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 492943 1 0000000015983799 100 0 0 10 0";

    #[test]
    fn parses_a_good_line() {
        let tcp_entry = parse_socket(GOOD_LINE).unwrap();
        assert_eq!(tcp_entry.local_address, "0100007F");
        assert_eq!(tcp_entry.port, 49374);
        assert_eq!(tcp_entry.state, 10);
        assert_eq!(tcp_entry.uid, 1000);
        assert_eq!(tcp_entry.inode, 492943);
    }

    #[test]
    fn parses_a_second_line() {
        let tcp_entry = parse_socket("0: 3500007F:0035 00000000:0000 0A 00000000:00000000 00:00000000 00000000   975        0 15411 1 0000000083f5dc00 100 0 0 10 5").unwrap();
        assert_eq!(tcp_entry.local_address, "3500007F");
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
}
