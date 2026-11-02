#[cfg(target_os = "linux")]
use std::fs;
use std::{
    io,
    process::{Command, Stdio},
};

#[cfg(target_os = "linux")]
pub fn read_entries(path: &str) -> io::Result<Vec<String>> {
    let contents = fs::read_to_string(path)?;
    Ok(contents.lines().skip(1).map(str::to_string).collect())
}

pub fn read_entries_mac() -> io::Result<String> {
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
