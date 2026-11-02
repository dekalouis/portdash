#[derive(PartialEq, Debug)]
pub struct TcpEntry {
    pub local_address: String,
    pub port: u16,
    pub state: u8,
    pub uid: u32,
    pub inode: u64,
}

#[derive(PartialEq, Debug)]
pub enum ParseError {
    MissingField(&'static str),
    BadNumber(String),
}

pub fn parse_socket(line: &str) -> Result<TcpEntry, ParseError> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    let raw_address = fields
        .get(1)
        .ok_or(ParseError::MissingField("raw_address"))?;
    let (address, port) = raw_address
        .split_once(":")
        .ok_or(ParseError::MissingField("port"))?;
    let port =
        u16::from_str_radix(port, 16).map_err(|_| ParseError::BadNumber(port.to_string()))?;
    let raw_state = fields.get(3).ok_or(ParseError::MissingField("state"))?;
    let state = u8::from_str_radix(raw_state, 16)
        .map_err(|_| ParseError::BadNumber(raw_state.to_string()))?;
    let uid: u32 = fields
        .get(7)
        .ok_or(ParseError::MissingField("uid"))?
        .parse()
        .map_err(|_| ParseError::BadNumber("uid".to_string()))?;
    let inode: u64 = fields
        .get(9)
        .ok_or(ParseError::MissingField("inode"))?
        .parse()
        .map_err(|_| ParseError::BadNumber("inode".to_string()))?;

    Ok(TcpEntry {
        local_address: address.to_string(),
        port,
        state,
        uid,
        inode,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_good_line() {
        let tcp_entry = parse_socket("1: 0100007F:C0DE 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 492943 1 0000000015983799 100 0 0 10 0").unwrap();
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
            Err(ParseError::BadNumber(String::from("ZZZZ")))
        );
    }

    #[test]
    fn rejects_empty_string() {
        assert_eq!(
            parse_socket(""),
            Err(ParseError::MissingField("raw_address"))
        );
    }

    #[test]
    fn rejects_bad_state() {
        assert_eq!(
            parse_socket(
                "1: 0100007F:C0DE 00000000:0000 0Z 00000000:00000000 00:00000000 00000000  1000        0 492943 1 0000000015983799 100 0 0 10 0"
            ),
            Err(ParseError::BadNumber(String::from("0Z")))
        );
    }

    #[test]
    fn rejects_bad_number_uid() {
        assert_eq!(
            parse_socket(
                "1: 0100007F:C0DE 00000000:0000 0A 00000000:00000000 00:00000000 00000000  abc        0 492943 1 0000000015983799 100 0 0 10 0"
            ),
            Err(ParseError::BadNumber(String::from("uid")))
        );
    }

    // inode is decimal, so a hex digit in it must fail instead of being read as hex
    #[test]
    fn rejects_mixed_bases_inode() {
        assert_eq!(
            parse_socket(
                "1: 0100007F:C0DE 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 49A943 1 0000000015983799 100 0 0 10 0"
            ),
            Err(ParseError::BadNumber(String::from("inode")))
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
