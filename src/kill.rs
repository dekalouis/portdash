#![cfg(target_os = "linux")]

use std::{
    io::{self, BufRead, Write},
    time::{Duration, Instant},
};

use nix::{
    errno::Errno,
    sys::signal::{Signal, kill},
    unistd::Pid,
};

use crate::{
    process::read_details,
    scan::{Owner, scan},
};

/// Stops the processes listening on `port`, asking first unless `yes` is set.
///
/// Sends SIGTERM to the main owners and waits, then SIGKILL to any owner still alive.
/// With `force`, sends SIGKILL right away.
pub fn kill_port(port: u16, force: bool, yes: bool) -> io::Result<()> {
    let listeners = scan()?;

    let mut owners: Vec<&Owner> = Vec::new();

    for listener in listeners.iter().filter(|listener| listener.port == port) {
        for owner in &listener.owners {
            if !owners.iter().any(|seen| seen.pid == owner.pid) {
                owners.push(owner);
            }
        }
    }

    if owners.is_empty() {
        if listeners.iter().any(|listener| listener.port == port) {
            eprintln!("port {port} is held by another user's process (try sudo)");
        } else {
            eprintln!("nothing is listening on port {port}");
        }
        std::process::exit(1);
    }

    println!("port {port} is held by:");
    for owner in &owners {
        println!("  {} {} {}", owner.pid, owner.name, owner.command);
    }
    if !yes && !confirm("stop it?")? {
        println!("cancelled");
        return Ok(());
    }

    //main owners: those whose parent is not also an owner
    let roots: Vec<&Owner> = owners
        .iter()
        .copied()
        .filter(|owner| !owners.iter().any(|other| other.pid == owner.parent_pid))
        .collect();

    if force {
        for owner in &owners {
            if send(owner, Signal::SIGKILL) {
                println!("sent SIGKILL to PID {}", owner.pid);
            }
        }
    } else {
        for owner in &roots {
            if send(owner, Signal::SIGTERM) {
                println!("sent SIGTERM to PID {}", owner.pid);
            }
        }
        if !wait_until_gone(&owners, Duration::from_secs(5)) {
            println!("still running after 5 seconds, sending SIGKILL");
            for owner in &owners {
                send(owner, Signal::SIGKILL);
            }
        }
    }
    if wait_until_gone(&owners, Duration::from_secs(1)) {
        println!("port {port} is free");
        Ok(())
    } else {
        eprintln!("could not stop everything on port {port}");
        std::process::exit(1);
    }
}

fn confirm(question: &str) -> io::Result<bool> {
    print!("{question} [y/N] ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().lock().read_line(&mut answer)?;
    let answer = answer.trim();
    Ok(answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes"))
}

fn checked_pid(pid: u32) -> Option<Pid> {
    if pid <= 1 || pid == std::process::id() {
        return None;
    }
    let raw = i32::try_from(pid).ok()?;
    Some(Pid::from_raw(raw))
}

// shortcut: a PID can still be reused between this check and the signal; a pidfd closes that gap
// A zombie has exited and only waits for its parent to collect it, so it counts as gone.
fn is_same_process(owner: &Owner) -> bool {
    match read_details(owner.pid) {
        Some(details) => details.stat.start_time == owner.start_time && details.stat.state != 'Z',
        None => false,
    }
}

/// Sends signal to one owner after the safety checks. Returns false if it was not sent.
fn send(owner: &Owner, signal: Signal) -> bool {
    let Some(pid) = checked_pid(owner.pid) else {
        eprintln!("refusing to signal PID {}", owner.pid);
        return false;
    };
    if !is_same_process(owner) {
        eprintln!("PID {} is no longer the same process, skipped", owner.pid);
        return false;
    }
    match kill(pid, signal) {
        Ok(()) => true,
        Err(Errno::EPERM) => {
            eprintln!(
                "PID {} ({}) belongs to another user (try sudo)",
                owner.pid, owner.name
            );
            false
        }
        Err(Errno::ESRCH) => true,
        Err(err) => {
            eprintln!("could not signal PID {}: {err}", owner.pid);
            false
        }
    }
}

/// Polls every 100ms until none of the 'owners' is running, or 'timeout' passes
fn wait_until_gone(owners: &[&Owner], timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if !owners.iter().any(|owner| is_same_process(owner)) {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn refuses_pid_zero_and_one() {
        assert!(checked_pid(0).is_none());
        assert!(checked_pid(1).is_none());
    }

    #[test]
    fn refuses_std_process_id() {
        assert!(checked_pid(std::process::id()).is_none());
    }

    #[test]
    fn refuses_u32_max() {
        assert!(checked_pid(u32::MAX).is_none());
    }

    #[test]
    fn normal_number_gives_some() {
        assert_eq!(checked_pid(4242), Some(Pid::from_raw(4242)));
    }
}
