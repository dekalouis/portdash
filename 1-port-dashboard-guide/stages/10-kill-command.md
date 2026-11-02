# Stage 10: The kill command

On a Mac? Also read [this stage's Mac notes](10-kill-command-mac-support.md).

**Goal:** `portdash kill 3000` stops whatever is on port 3000: politely first, forcefully if necessary, and never the wrong process.

This is the first thing your tool does that can cause damage. Treat it with care.

## Background

### Signals

A **signal** is a message the kernel delivers to a process. Despite the name, `kill` just sends signals, and two matter here:

| Signal | Effect | Can the process refuse? |
|---|---|---|
| `SIGTERM` | "Please shut down." It can save data and close connections | Yes, it can catch or ignore this |
| `SIGKILL` | Ended immediately by the kernel, with no cleanup | No |

The right pattern is: send `SIGTERM`, wait a few seconds, and escalate to `SIGKILL` only if it is still alive. Jumping straight to `SIGKILL` can leave half-written files and lock files behind.

The `nix` crate provides safe bindings: `nix::sys::signal::kill(pid, signal)`. It needs the `signal` feature. Passing `None` as the signal sends nothing, but still checks whether the process exists. That is how you poll for it being gone.

### Dangerous PID values

These are real behaviors of the `kill` system call, and they are why the validation below matters:

| PID value | What `kill` does |
|---|---|
| `0` | Signals every process in **your own process group**, including you |
| `-1` | Signals **every process you are allowed to signal**. Run as root, this takes down the machine |
| `1` | Targets `init`. As root, this is very bad news |

Your PIDs are `u32`, which rules out negatives until you convert to `i32` to call `kill`. Validate before converting, and refuse anything `<= 1`. Also refuse your own PID.

### The PID reuse race

Picture this:

1. You scan: port 3000 is PID 5000.
2. Process 5000 exits by itself.
3. The kernel reuses 5000 for something unrelated.
4. You send a signal to 5000, and kill an innocent process.

It is rare, but this is exactly why Stage 7 stored the start time. Immediately before signaling, read the start time again and compare. If it differs, then it is not the same process, so stop and report it.

A small gap remains between checking and signaling. Closing it fully takes a Linux feature called `pidfd`, which you can look into later. Leave a `shortcut:` comment about the limit.

### Permission errors

Signaling a process you do not own fails with `EPERM`. That is expected, and deserves a helpful message, such as "PID 1203 belongs to postgres, try sudo". Do not suggest it automatically, and never call `sudo` yourself.

### Multiple owners

If a socket has four owners, which do you signal? For a server with workers, you want the parent, which then shuts down its workers. Signaling only one worker often just causes it to be replaced. A simple rule is to signal all the listed owners. A better one is to find the owner whose parent is not also an owner. Pick one, and tell the user what you did.

## Your task

1. Add a `kill <port>` subcommand with `--force` (skip to `SIGKILL` right away) and `--yes` (no prompt).
2. Find the port's owners with `scan()`. Handle no listener, and unknown owner (suggest `sudo`).
3. Print what you are about to do, and ask for confirmation unless `--yes` is passed.
4. Validate the PID, and re-check the start time immediately before signaling.
5. Send `SIGTERM`, poll until it is gone or a timeout passes (5 seconds is reasonable), and then escalate to `SIGKILL`.
6. Report clearly what happened. Exit non-zero if the process could not be stopped.

## Generic example

The graceful stop pattern, using something other than ports:

```rust
use nix::{sys::signal::{kill, Signal}, unistd::Pid};
use std::{thread, time::{Duration, Instant}};

/// Returns true if the process exited within the grace period.
fn stop_gracefully(pid: i32, grace: Duration) -> nix::Result<bool> {
    let pid = Pid::from_raw(pid);
    kill(pid, Signal::SIGTERM)?;

    let deadline = Instant::now() + grace;
    while Instant::now() < deadline {
        // Passing None sends no signal, but still fails if the process is gone.
        if kill(pid, None).is_err() {
            return Ok(true);
        }
        thread::sleep(Duration::from_millis(50));
    }
    Ok(false)
}
```

Note that this example has no safety checks. Yours needs them.

Reading a confirmation:

```rust
use std::io::{self, BufRead, Write};

fn confirm(question: &str) -> io::Result<bool> {
    print!("{question} [y/N] ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().lock().read_line(&mut answer)?;
    Ok(answer.trim().eq_ignore_ascii_case("y"))
}
```

## Verify

**A polite server:**

```sh
python3 -m http.server 8765 --bind 127.0.0.1 &
cargo run -- kill 8765
```

It should stop quickly, with no `SIGKILL`.

**A stubborn server that ignores `SIGTERM`:**

```sh
python3 -c '
import signal, socket, time
signal.signal(signal.SIGTERM, signal.SIG_IGN)
s = socket.socket(); s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("127.0.0.1", 8766)); s.listen(); time.sleep(600)' &
cargo run -- kill 8766
```

It should wait, then escalate, then report the escalation.

**Other cases:**

```sh
cargo run -- kill 59999            # nothing listening: clear message, non-zero exit
cargo run -- kill notaport         # clap rejects it
cargo run -- kill 631              # owned by another user, if you have CUPS: permission message
```

## Pitfalls

- **Zombies look alive.** When a process exits but its parent has not collected the exit status yet, it is a zombie. `kill(pid, None)` still succeeds on a zombie, so your loop thinks it is running. In the verify step above, your shell collects the status so it works, but a neglectful parent would make you escalate for nothing. Check the state in `/proc/<pid>/stat`, where `Z` means zombie, and count that as gone.
- **Converting without checking.** `some_u32 as i32` silently turns large numbers negative. Use `i32::try_from`.
- **Not re-reading right before the signal.** A check made when the scan ran is too old, especially after waiting for the user to answer a prompt.
- **The server you stopped is back.** A supervisor such as `systemd`, `nodemon` or `docker` may restart it. If the port has a new owner after the stop, tell the user.
- **Wrappers.** With `npm run dev`, `node` holds the port, and `npm` is its parent. You stop the child, and the parent generally exits too, but not always. Stage 18 looks at this properly.

## Done when

- [ ] The polite and stubborn servers both behave as described
- [ ] PIDs at or below 1, and your own PID, are refused, with a test
- [ ] Start time is re-checked before signaling
- [ ] Permission failures give a helpful message
- [ ] Zombies count as stopped
- [ ] Committed

**Milestone:** Part 3 is complete. You could release this as version 0.1 today.
