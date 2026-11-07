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

/// The fields of `/proc/<pid>/stat` that portdash uses.
struct Stat {
    /// Process name from inside the parentheses. The kernel cuts it at 15 characters.
    name: String,
    /// Parent process ID (field 4).
    parent_pid: u32,
    /// Start time in clock ticks since boot (field 22).
    start_time: u64,
}

/// Parses the text of `/proc/<pid>/stat`.
///
/// The name sits in parentheses and can itself contain spaces and parentheses,
/// so it runs from the first `(` to the last `)`. Returns `None` if a
/// parenthesis is missing, or a field is absent or not a number.
fn parse_stat(text: &str) -> Option<Stat> {
    let open = text.find('(')?;
    let close = text.rfind(')')?;
    let name = &text[open + 1..close];
    let rest = text[close + 1..].trim();
    let fields: Vec<&str> = rest.split_whitespace().collect();
    Some(Stat {
        name: name.to_string(),
        parent_pid: fields.get(1)?.parse().ok()?,
        start_time: fields.get(19)?.parse().ok()?,
    })
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
    fn keeps_spaces_inside_an_argument() {
        assert_eq!(
            parse_cmdline(b"cook\0--fast mode\0"),
            ["cook", "--fast mode"]
        );
    }

    // Test for the parse_stat function
    const REST: &str = "S 762077 774246 2801186 34826 774246 4194304 128 0 0 0 0 0 0 0 20 0 2 0 16519640 73543680 767 18446744073709551615 94890250874048 94890251644800 140723654844240 0 0 0 0 4096 1088 0 0 0 17 7 0 0 0 0 0 94890251704304 94890251706952 94891093962752 140723654851832 140723654851948 140723654851948 140723654860704 0";

    #[test]
    fn parses_a_normal_name() {
        let stat = parse_stat(&format!("1 (bash) {REST}")).unwrap();
        assert_eq!(stat.name, "bash");
        assert_eq!(stat.parent_pid, 762077);
        assert_eq!(stat.start_time, 16519640);
    }

    #[test]
    fn parses_a_weird_name() {
        let stat = parse_stat(&format!("1 (my (weird) app) {REST}")).unwrap();
        assert_eq!(stat.name, "my (weird) app");
        assert_eq!(stat.parent_pid, 762077);
        assert_eq!(stat.start_time, 16519640);
    }

    #[test]
    fn rejects_a_missing_closing_paren() {
        assert!(parse_stat("1 (bash").is_none());
    }

    #[test]
    fn rejects_too_few_fields() {
        assert!(parse_stat("1 (bash) S 2 3").is_none());
    }

    #[test]
    fn rejects_a_non_numeric_parent_pid() {
        let text = format!("1 (bash) {}", REST.replacen("762077", "abc", 1));
        assert!(parse_stat(&text).is_none());
    }
}
