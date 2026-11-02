# Stage 2 on a Mac: Read the raw socket table

## What differs on a Mac, and why

macOS has no `/proc`, so there is no socket table to read as a file:

```sh
ls /proc
# "/proc": No such file or directory
```

`ss` is missing too. The Mac version asks a tool for the data: **`lsof`**, which ships with macOS at `/usr/sbin/lsof`, has a field format meant for programs. You run it with `std::process::Command` and keep the same rule as the original stage: getting the data stays separate from using it.

### Why `lsof`, and not a crate

I measured the three options on this Mac, in a release build:

| Option | Time per scan | Sees other users' sockets? | Gives you |
|---|---|---|---|
| `lsof` through `Command` | about 53 ms | No | Text you parse, as in the rest of this guide |
| `libproc` 0.14 | about 5 ms | No (`EPERM` on 363 of 1065 processes) | Fast native calls, but `unsafe` union access, and its `pidcwd` returns "not implemented for macos" |
| `listeners` 0.6 | about 3 ms | No | PID, name, socket, protocol, state, and no parent, working folder, memory or start time |

Use `lsof`. The speed is enough once scanning has its own thread (Stage 14 has the numbers), it adds no dependency or `unsafe`, and your parsing stays plain functions you can test with a string, as Stages 3 to 5 expect. The crates are faster but show you no more sockets. `libproc` stays a sensible upgrade if your Stage 14 numbers disappoint you.

### What to run

```sh
/usr/sbin/lsof -nP -iTCP -sTCP:LISTEN -FpcuRtdn
```

| Part | Meaning |
|---|---|
| `-nP` | Numbers only, with no name lookups for hosts or ports, which can stall |
| `-iTCP -sTCP:LISTEN` | Only TCP sockets in the LISTEN state |
| `-F...` | One field per line, for programs. Stage 3 decodes it |

Your first output looks like this:

```
p52733
R52704
cpython3.13
u501
f3
tIPv4
d0x62a0271f722ff24b
n127.0.0.1:8771
```

### Exit codes do not mean what you expect

`lsof` exits with 1 when it finds nothing **and** when you use it wrongly. Only stderr tells them apart:

| Situation | Exit code | stderr |
|---|---|---|
| Nothing matches | 1 | Empty |
| Bad option | 1 | 539 bytes of usage text |

Treat "exit 1 and empty stderr" as an empty list, and anything else on stderr as a failure.

## Your task on a Mac

Replace tasks 2 and 3:

2. Write a function in `src/sockets.rs` that runs `lsof` as above, with stdin closed, and returns its stdout as a `String` or an error. Apply the exit code rule. Use the full path `/usr/sbin/lsof`.
3. Print every line it returned, with a count of `n` lines at the end.

Task 1 and task 4 stay the same. Keep the Linux reader, if you have one, behind `#[cfg(target_os = "linux")]`. Stage 9 gives this split its final shape.

## Generic example

`grep` follows the same convention: exit 1 for "no match", so that is not a failure.

```rust
use std::{
    io,
    process::{Command, Stdio},
};

/// Counts matching lines in a recipe file. `grep` exits 1 for "no match" and 2 for a real error.
fn count_matches(word: &str, path: &str) -> io::Result<usize> {
    let output = Command::new("/usr/bin/grep")
        .args([word, path])
        .stdin(Stdio::null())
        .output()?;
    match output.status.code() {
        Some(0) | Some(1) => Ok(String::from_utf8_lossy(&output.stdout).lines().count()),
        _ => Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )),
    }
}

fn main() {
    std::fs::write("recipes-demo.txt", "soup\nbread\nsoup stock\n").unwrap();
    assert_eq!(count_matches("soup", "recipes-demo.txt").unwrap(), 2);
    assert_eq!(count_matches("cake", "recipes-demo.txt").unwrap(), 0);
    assert!(count_matches("soup", "no-such-file.txt").is_err());
    std::fs::remove_file("recipes-demo.txt").unwrap();
    println!("ok");
}
```

## Verify

```sh
cargo run
/usr/sbin/lsof -nP -iTCP -sTCP:LISTEN -Fn | grep -c '^n'   # should match your count

/usr/sbin/lsof -nP -iTCP:59997 -sTCP:LISTEN -Fp; echo "exit=$?"    # no output, exit=1
/usr/sbin/lsof --nonsense 2>/dev/null; echo "exit=$?"              # exit=1, with stderr hidden here
```

Both exit codes came out as shown. `lsof` took 40 to 50 ms each time from the shell.

To test the error path, temporarily point your code at a tool that does not exist. `Command` returns an error with kind `NotFound`, so you should get your message and a non-zero exit, not a panic.

## Pitfalls

- **`status.success()` as the only check.** It reports failure on a machine where nothing is listening, which would turn a quiet machine into an error.
- **Trusting `netstat` as your reference.** `netstat -an` printed no TCP rows at all here, with exit code 0, even outside my tool sandbox. I did not find out why. Compare against `lsof` instead.
- **Leaving stdin open.** Close it with `Stdio::null()`, as the example does. A child that wants input should fail, not hang your dashboard.
- **Relying on `PATH`.** A bare `lsof` can run something else, which matters once anyone runs your tool with `sudo`.

## Changes to "Done when"

Replace "Your output has every socket line without the header" with:

- [ ] Your output has every line `lsof` returned, and your count matches the `grep -c` command above
- [ ] A machine with no listeners gives an empty list, not an error

## Tested on

macOS 27.2 (build 26B123), Apple M1 Max, 2026-11-02. `lsof` 4.91. libproc 0.14.11, listeners 0.6.1. `cargo run` results came from a throwaway reference build in a temp folder, not from a finished `portdash`, so your wording will differ.

Not tested: an Intel Mac.
