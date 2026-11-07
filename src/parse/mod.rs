#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "macos")]
pub use macos::*;

use std::fmt;

/// Why a line of socket output could not be parsed.
#[derive(PartialEq, Debug)]
pub enum ParseError {
    /// A field the parser needs is not there.
    MissingField(&'static str),
    /// A numeric field holds text that is not a number in the expected base.
    BadNumber { field: &'static str, text: String },
    /// An address has the wrong length or form for an IPv4 or IPv6 address.
    BadAddress(String),
    /// `lsof` reported a wildcard address under a family other than IPv4 or IPv6.
    #[cfg(target_os = "macos")]
    UnknownFamily(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MissingField(field) => write!(f, "missing field: {field}"),
            ParseError::BadNumber { field, text } => {
                write!(f, "{field} is not a valid number: {text:?}")
            }
            ParseError::BadAddress(text) => write!(f, "not a valid address: {text:?}"),
            #[cfg(target_os = "macos")]
            ParseError::UnknownFamily(family) => {
                write!(f, "unknown socket family: {family:?}")
            }
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_a_missing_field() {
        assert_eq!(
            ParseError::MissingField("port").to_string(),
            "missing field: port"
        );
    }

    #[test]
    fn describes_a_bad_number() {
        let err = ParseError::BadNumber {
            field: "uid",
            text: "abc".to_string(),
        };
        assert_eq!(err.to_string(), "uid is not a valid number: \"abc\"");
    }

    #[test]
    fn describes_a_bad_address() {
        assert_eq!(
            ParseError::BadAddress("0100007".to_string()).to_string(),
            "not a valid address: \"0100007\""
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn describes_an_unknown_family() {
        assert_eq!(
            ParseError::UnknownFamily("IPv5".to_string()).to_string(),
            "unknown socket family: \"IPv5\""
        );
    }
}
