#![cfg(target_os = "linux")]

/// Splits the NUL-separated bytes of `/proc/<pid>/cmdline` into arguments.
///
/// The bytes may not be valid UTF-8, so invalid parts are replaced, not rejected.
fn parse_cmdline(bytes: &[u8]) -> Vec<String> {
    bytes
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty())
        .map(|part| String::from_utf8_lossy(part).into_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_nul_bytes() {
        assert_eq!(
            parse_cmdline(b"python3\0-m\0http.server\0"),
            ["python3", "-m", "http.server"]
        );
    }

    #[test]
    fn parses_empty_bytes() {
        assert!(parse_cmdline(b"").is_empty());
    }

    #[test]
    fn parses_fastmode_as_one_arg() {
        assert_eq!(
            parse_cmdline(b"cook\0--fast mode\0"),
            ["cook", "--fast mode"]
        );
    }
}
