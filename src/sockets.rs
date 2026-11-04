#[cfg(target_os = "linux")]
use std::fs;
use std::io;
#[cfg(target_os = "macos")]
use std::process::{Command, Stdio};

#[cfg(target_os = "linux")]
pub fn read_entries() -> io::Result<Vec<String>> {
    let mut res = Vec::new();
    let tcp = fs::read_to_string("/proc/net/tcp")?;
    res.push(tcp);
    match fs::read_to_string("/proc/net/tcp6") {
        Ok(text) => res.push(text),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => return Err(err),
    }
    Ok(res)
}

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
