# Stage 10 on a Mac: The kill command

## What differs on a Mac, and why

Signals and `nix::sys::signal::kill` work the same, including the `None` signal. The polite and stubborn servers behaved as described. Three things change.

**Zombie checks use `ps`.** `kill(pid, None)` succeeds on a zombie here as well. There is no `/proc/<pid>/stat`, so read `stat` with `ps -o stat= -p PID`. A first letter of `Z` means it is gone, and so does no row at all.

**The start time check uses `lstart`.** Compare the text from Stage 7, again with `LC_ALL=C`.

**"No listener" may be false.** This is the Stage 6 limit, in its most misleading form. `portdash kill 5432` finds no row on this machine, yet PostgreSQL is listening there under another user. Telling the user "nothing is listening" would be wrong.

You can tell the two cases apart without root. When a port has no visible row, try to connect to it on `127.0.0.1` and `::1` using `TcpStream::connect_timeout` with a short limit. If something answers, the port is held by a process you cannot see.

Only do this for the single port the user asked about, and never in `scan()`.

**There is no `pidfd`.** The original suggests it as a future fix for the remaining race. It is a Linux feature, so on a Mac your `shortcut:` comment should describe the limit without promising that fix.

**Supervisors have other names.** `launchd` restarts services marked KeepAlive, as `systemd` would. Many Mac processes have parent PID 1 here (`rapportd`, `Spotify`, even Homebrew's Redis), so a parent of 1 says little about whether something will come back.

## Your task on a Mac

Change these tasks:

2. For "no listener", probe the port as above. Give one of two messages: nothing is listening, or something answers that you cannot see (suggest `sudo`). Both exit non-zero.
4. Re-read `lstart` immediately before signaling.
5. While polling, count a `Z` state or a missing `ps` row as stopped.

## Verify

The polite and stubborn commands work as written. Results here:

```
stopping python3.13 (PID 57094)
Graceful                                 # exit 0, right away

stopping python3.13 (PID 57130)
Escalated                                # exit 0, after 5.2 seconds
```

The other cases, with `631` replaced. `cupsd` was not listening here, so use a port you know another user holds:

```sh
cargo run -- kill 59999      # error: nothing is listening on port 59999       exit 1
cargo run -- kill notaport   # error: invalid value 'notaport' for '<PORT>'
cargo run -- kill 5432       # error: port 5432 answers, but lsof cannot see its owner.
                             # It probably belongs to another user: try sudo    exit 1
```

**A neglectful parent,** which the original only describes. The parent never collects its child, so the child becomes a zombie the moment it stops:

```sh
python3 -c '
import socket, os, time
pid = os.fork()
if pid == 0:
    s = socket.socket(); s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind(("127.0.0.1", 8769)); s.listen(); time.sleep(30); os._exit(0)
time.sleep(25)' &
PARENT=$!
cargo run -- kill 8769
kill $PARENT
```

Here it reported `Graceful` in 0.16 seconds. Without the zombie check, it would wait out the grace period and escalate.

**A permission failure.** `lsof` hides other users' processes, so a normal scan will rarely lead you to `EPERM`. You still need the message for when a PID comes from another source. To see the error without harming anything, send the null signal, which delivers nothing, to a process you do not own:

```sh
python3 -c 'import os; os.kill(1, 0)'
# PermissionError: [Errno 1] Operation not permitted
```

## Pitfalls

- **Scanning before you probe.** Do the scan first. The probe only explains an empty result, and an answer from your own visible server should not reach it.
- **Thinking the probe proves who owns it.** It shows only that something accepts connections.
- **Probing only IPv4.** A server on `::1` alone would look free.
- **Root sees launchd as an owner.** With `sudo lsof`, port 8021 here belonged to `launchd`, PID 1, because launchd holds sockets for services it starts on demand. Your rule refusing PIDs at or below 1 protects you from a very bad day, so test it. The useful message is that a launchd service owns the port.
- **Exact time matches across locales.** Run both reads with the same `LC_ALL=C`.

## Changes to "Done when"

Replace "Permission failures give a helpful message" with:

- [ ] A port held by a hidden owner gets its own message, not "nothing is listening"
- [ ] `EPERM` still gets a helpful message, for when a PID comes from elsewhere

"Zombies count as stopped" stays, using `ps`.

## Tested on

macOS 27.2 (build 26B123), Apple M1 Max, 2026-11-02. `nix` 0.31.3. `cargo run` results came from a throwaway reference build in a temp folder, not from a finished `portdash`, so your wording will differ.

Not tested: running the tool as root and a real Intel Mac. In the local domains guide's tests, a KeepAlive agent restarted after `kill -9` and showed `spawn scheduled` after `SIGTERM`, so expect similar behavior from any service set up that way.
