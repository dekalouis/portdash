# Stage 7: Read process details

On a Mac? Also read [this stage's Mac notes](07-process-details-mac-support.md).

**Goal:** turn a bare PID into something a human recognizes: name, full command, working directory, owner and start time.

## Background

Each process's folder in `/proc` holds what you need:

| Path | What it holds | Format |
|---|---|---|
| `/proc/<pid>/comm` | Short process name | Text, ending in a newline |
| `/proc/<pid>/cmdline` | Full command with arguments | Arguments separated by NUL bytes |
| `/proc/<pid>/cwd` | Current working directory | Symlink |
| `/proc/<pid>/stat` | Status numbers (parent PID, start time, ...) | One line of space-separated fields |

Look at them for your own shell:

```sh
cat /proc/$$/comm
tr '\0' ' ' < /proc/$$/cmdline; echo
ls -l /proc/$$/cwd
cat /proc/$$/stat
```

### `cmdline` uses NUL separators

Arguments can contain spaces, so a space is no use as a separator. The kernel separates them with the NUL byte (`\0`), and there is usually a trailing one. This also means the content may not be valid UTF-8, so read it as bytes (`fs::read`) rather than as a string. `String::from_utf8_lossy` converts it, replacing anything invalid.

### `comm` is often not what you want

`comm` is cut off at 15 characters, and for many dev servers it says just `node` or `python3`. Keep it, but expect that the project name (next stage) and command line will carry most of the meaning.

### Parsing `stat` has a trap

`stat` looks like this:

```
4146070 (cut) R 4146057 4146057 ...
```

The second field is the process name in parentheses. That name can contain **spaces and parentheses**, since a program can name itself almost anything. So `(my (weird) app)` is legal, and splitting the line on spaces falls apart.

The reliable way is to find the **last** `)` in the line. Everything after that is well-behaved numbers, so splitting on whitespace works there.

Field numbers are listed in `man 5 proc_pid_stat`, or `man 5 proc` on some systems. You need field 4 (parent PID) and field 22 (start time). They are numbered from 1, counting the PID as field 1 and the name as field 2. So work out carefully what index those become in the part after the `)`.

### Start time, and why it identifies a process

Field 22 is when the process started, in **clock ticks since boot**. It does two jobs:

1. **Uptime.** Current uptime (the first number in `/proc/uptime`) minus start time gives how long the process has been running. Ticks convert to seconds using the system tick rate, normally 100 per second, which you can read with `nix::unistd::sysconf(SysconfVar::CLK_TCK)`.
2. **Identity.** PIDs get reused. Once a process exits, its number can go to something new. But the pair of *(PID, start time)* really does identify one specific process. That prevents a very bad bug in Stage 10.

### Owner

Stage 3 gave you a numeric UID. People prefer names. You can look one up with `nix::unistd::User::from_uid`, which needs the `user` feature of `nix`:

```toml
nix = { version = "0.31", features = ["user", "feature"] }
```

The `feature` feature is what enables `sysconf`. Yes, it really is named that.

## Your task

1. Define a struct holding: PID, parent PID, name, command line, working directory (`Option`, since it may be unreadable), and start time.
2. Write a pure parsing function for `stat` contents that takes `&str`, and test it.
   - `1 (bash) S 2 ...` works
   - `1 (my (weird) app) S 2 ...` works and gives the name `my (weird) app`
3. Write a pure function that converts `cmdline` bytes to arguments, and test it.
4. Write a function that gathers all of it for a PID, from the real files.
5. Decide what is required and what is optional. If a process vanishes partway through, the function should return "no details" rather than half-made data.
6. Show the name, command and working directory next to each listener.

## Generic example

Text with a messy middle part, where you cut from the last delimiter:

```rust
fn after_title(line: &str) -> Option<(&str, &str)> {
    let open = line.find('(')?;
    let close = line.rfind(')')?;
    Some((&line[open + 1..close], line[close + 1..].trim()))
}

fn main() {
    let (title, rest) = after_title("7 (Mr (Big) Book) 300 pages").unwrap();
    assert_eq!(title, "Mr (Big) Book");
    assert_eq!(rest, "300 pages");
}
```

Splitting bytes on a separator, and tolerating bad text:

```rust
fn nul_fields(bytes: &[u8]) -> Vec<String> {
    bytes
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty())
        .map(|part| String::from_utf8_lossy(part).into_owned())
        .collect()
}

fn main() {
    assert_eq!(nul_fields(b"cook\0--fast mode\0"), vec!["cook", "--fast mode"]);
}
```

## Verify

```sh
cd /tmp && python3 -m http.server 8765 --bind 127.0.0.1 &
cargo run
ps -o pid,ppid,comm,args -p $!
kill %1
```

- The name, command and parent PID should match `ps`.
- The working directory should be `/tmp`, because the server started there.
- The process's uptime should go up each time you run it.

## Pitfalls

- **Splitting `stat` on spaces from the start.** Your test with a weird name catches this.
- **Off-by-one field numbers.** The man page counts from 1, and you are counting after removing two fields. Check by comparing the parent PID with `ps`.
- **Empty `cmdline`.** Kernel threads have none. Fall back to `comm`.
- **`cwd` is not UTF-8 or gets removed.** A path may contain any bytes, so `to_string_lossy` is your friend when displaying it. If the folder was deleted, the link target ends in ` (deleted)`.
- **Reading `/proc/<pid>/environ`.** It holds environment variables, which often include secrets. You do not need it here. Stage 18 discusses it carefully.

## Done when

- [ ] Each listener with a known owner shows name, command and working directory
- [ ] The weird-name test passes
- [ ] A process that disappears mid-read does not crash anything
- [ ] Parent PID and start time are stored for later
- [ ] Committed
