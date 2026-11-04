#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "macos")]
pub use macos::*;

// TODO: implement `Display` for readable messages (Stage 8a).
#[derive(PartialEq, Debug)]
pub enum ParseError {
    MissingField(&'static str),
    BadNumber { field: &'static str, text: String },
}
