#![cfg(target_os = "macos")]
use super::ParseError;

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

fn parse_u32(field: &'static str, value: &str) -> Result<u32, ParseError> {
    value.parse().map_err(|_| ParseError::BadNumber {
        field,
        text: value.to_string(),
    })
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
