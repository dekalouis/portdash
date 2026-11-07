#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
mod details;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "macos")]
pub use macos::*;

use std::net::IpAddr;

/// One listening socket and the processes that hold it, the same on every platform.
#[derive(PartialEq, Debug)]
pub struct Listener {
    /// Local IP address, IPv4 or IPv6.
    pub address: IpAddr,
    /// Local port.
    pub port: u16,
    /// PIDs holding the socket, each listed once. Empty when the owner cannot be read.
    pub pids: Vec<u32>,
}
