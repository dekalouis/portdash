#[cfg(target_os = "linux")]
use std::fs;
use std::io;
#[cfg(target_os = "macos")]
use std::process::{Command, Stdio};

/// Reads the raw text of `/proc/net/tcp` and, if it exists, `/proc/net/tcp6`.
///
/// A missing `tcp6` file means IPv6 is turned off, which is not an error.
#[cfg(target_os = "linux")]
pub fn read_socket_tables() -> io::Result<Vec<String>> {
    let mut tables = Vec::new();
    tables.push(fs::read_to_string("/proc/net/tcp")?);
    match fs::read_to_string("/proc/net/tcp6") {
        Ok(text) => tables.push(text),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(err),
    }
    Ok(tables)
}

/// Runs `lsof` for listening TCP sockets and returns its `-F` output.
///
/// `lsof` exits with 1 and prints nothing to stderr when no socket matches,
/// so that case is an empty result, not an error.
#[cfg(target_os = "macos")]
pub fn run_lsof() -> io::Result<String> {
    let output = Command::new("/usr/sbin/lsof")
        .args(["-nP", "-iTCP", "-sTCP:LISTEN", "-FpcuRtdn"])
        .stdin(Stdio::null())
        .output()?;

    match output.status.code() {
        Some(0) => Ok(String::from_utf8_lossy(&output.stdout).into_owned()),
        Some(1) if output.stderr.is_empty() => {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        }
        _ => Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )),
    }
}
