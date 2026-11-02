# Stage 18 on a Mac: Restart

## What differs on a Mac, and why

**The recipe has a different source.** There is no `/proc/<pid>/environ`. One kernel call returns arguments and environment together: `sysctl` with `KERN_PROCARGS2`, through the `libc` crate:

```toml
libc = "0.2"
```

| What | Mac source |
|---|---|
| Program and arguments | `KERN_PROCARGS2` |
| Working directory | `lsof -a -p PID -d cwd -Fn` |
| Environment | `KERN_PROCARGS2` |

The result is one buffer, in this order:

1. A 4-byte count of arguments (a C `int`)
2. The path of the executable, then NUL padding
3. That many arguments, each ending in NUL
4. The environment entries, each ending in NUL
5. An empty string, then Apple's private values, such as `ptr_munge`

Stop reading the environment at the empty string, or Apple's values end up in your new process.

Use this for the arguments as well. `ps -o command=` turned `--directory "/x/a b"` into text that cannot be split back, whereas this call kept `a b` as one argument, and the restarted server got it intact.

**`ps eww` is no replacement.** It showed the same variables here, but as one space-joined line that you cannot parse safely, and it still came back empty for the cases below.

### What it can and cannot read

This is the honest limit of restart on a Mac. I tested each with a fake variable, on processes I started:

| Process | Result |
|---|---|
| Python from pyenv | Environment read |
| `/usr/bin/python3` (Xcode's) | Environment read |
| Rust binary, ad-hoc signed | Environment read |
| Same binary with hardened runtime | Environment read |
| `node` from nvm, Developer ID signed, hardened runtime | Environment read |
| `/bin/sleep`, `/bin/bash`, `/usr/bin/nc` (Apple system binaries) | **Success, with zero variables** |
| Another user's process, and PID 1 | Error `EINVAL` (22) |

Look at the second-last row. The call **succeeds** and returns an empty environment, which looks like a process that truly has none. If you rebuild from that, your `env_clear` starts the server with no `PATH` and no secrets, and it may fail in a confusing way, or silently use defaults. I cannot tell you Apple's rule. The pattern here was Apple-signed system binaries.

Most dev servers are `node`, `python`, `ruby` or similar from a version manager, and those worked. Note also that on this Mac `/bin/sh` runs `/bin/bash`, so a server that is itself a shell script falls into the empty row.

Values a process sets after starting are not visible: a variable I set inside Python with `os.environ` did not appear. The original notes this for Linux too.

### The fallback

Do not guess. If the environment comes back empty, **refuse**, and say why:

```
error: macOS returned no environment for PID 61362 (/usr/bin/nc). Refusing to guess.
Re-run with --use-current-env to start it with this tool's environment instead.
```

The flag is an explicit choice by the user, and the popup in the TUI must say the same thing. A process with a truly empty environment gets refused too, which is rare and is the safe mistake to make.

### Logs and supervisors

Mac tools usually log to `~/Library/Logs/portdash/`. Respect `$XDG_STATE_HOME` if the user set it, because they asked for that.

`launchd` plays the part of `systemd` in the "Supervisors" warning, and a parent of PID 1 is an even weaker clue here, because many ordinary Mac apps have it.

The `CommandExt` and `process_group(0)` example from the original works unchanged on a Mac.

## Your task on a Mac

Change these tasks:

2. Build the plan from the sources above. Add one more clear error: "no environment returned". Map `EINVAL` to the same "another user or protected" message.
6. Add `--use-current-env` to `restart`, and an equivalent choice in the popup.
7. Test the parser on a hand-built buffer: normal arguments, an argument with a space, Apple's values after the empty string, and a buffer that is too short.

Start the new process from the executable path in the buffer, and pass the first argument with `CommandExt::arg0`. It keeps relative paths and odd names intact.

## Generic example

Two techniques: the two-step `sysctl` call, and reading a buffer with this layout. The topic is a recipe, not a process:

```rust
use std::{io, ptr};

/// Reads a text value from the kernel with sysctl: ask for the size, then fill a buffer.
fn kernel_version() -> io::Result<String> {
    let mut name = [libc::CTL_KERN, libc::KERN_VERSION];
    let mut size: libc::size_t = 0;

    // SAFETY: a null buffer asks only for the size, written to `size`.
    let status = unsafe {
        libc::sysctl(
            name.as_mut_ptr(),
            2,
            ptr::null_mut(),
            &mut size,
            ptr::null_mut(),
            0,
        )
    };
    if status != 0 {
        return Err(io::Error::last_os_error());
    }

    let mut buffer = vec![0u8; size];
    // SAFETY: `buffer` has room for `size` bytes, and `size` is updated to the bytes written.
    let status = unsafe {
        libc::sysctl(
            name.as_mut_ptr(),
            2,
            buffer.as_mut_ptr().cast(),
            &mut size,
            ptr::null_mut(),
            0,
        )
    };
    if status != 0 {
        return Err(io::Error::last_os_error());
    }
    buffer.truncate(size);
    Ok(String::from_utf8_lossy(&buffer)
        .trim_end_matches('\0')
        .to_string())
}

/// Reads a recipe blob: a 4-byte step count, a NUL-padded title, the steps, then the notes.
fn parse_recipe(blob: &[u8]) -> Option<(String, Vec<String>, Vec<String>)> {
    let count = usize::try_from(i32::from_ne_bytes(blob.get(..4)?.try_into().ok()?)).ok()?;
    let rest = &blob[4..];
    let title_end = rest.iter().position(|byte| *byte == 0)?;
    let title = String::from_utf8_lossy(&rest[..title_end]).into_owned();
    let body_start = title_end + rest[title_end..].iter().position(|byte| *byte != 0)?;

    let mut fields = rest[body_start..].split(|byte| *byte == 0);
    let steps: Vec<String> = fields
        .by_ref()
        .take(count)
        .map(|field| String::from_utf8_lossy(field).into_owned())
        .collect();
    if steps.len() != count {
        return None;
    }
    let notes = fields
        .take_while(|field| !field.is_empty()) // An empty field ends the notes.
        .map(|field| String::from_utf8_lossy(field).into_owned())
        .collect();
    Some((title, steps, notes))
}

fn main() {
    assert!(!kernel_version().unwrap().is_empty());

    let mut blob = 2i32.to_ne_bytes().to_vec();
    blob.extend_from_slice(b"Soup\0\0\0\0boil water\0add salt\0serves=2\0spicy=no\0\0hidden\0");
    let (title, steps, notes) = parse_recipe(&blob).unwrap();
    assert_eq!(title, "Soup");
    assert_eq!(steps, ["boil water", "add salt"]);
    assert_eq!(notes, ["serves=2", "spicy=no"]);
    println!("ok");
}
```

`take_while` stops at the empty field, so `hidden` stays out of the notes. Your parser needs the same care.

## Verify

These use a fake variable. **Never print a whole `ps eww` line**, because it contains real secrets. The `grep -o` keeps only the fake one.

**The simple case:**

```sh
cd /tmp
PORTDASH_FAKE_VAR=hello python3 -m http.server 8765 --bind 127.0.0.1 &
OLD=$!
cargo run -- restart 8765
NEW=$(/usr/sbin/lsof -nP -iTCP:8765 -sTCP:LISTEN -Fp | head -1 | cut -c2-)
echo "old=$OLD new=$NEW"
ps eww -o command= -p $NEW | grep -o 'PORTDASH_FAKE_VAR=[^ ]*'
/usr/sbin/lsof -a -p $NEW -d cwd -Fn | grep '^n'
curl -s --max-time 3 http://127.0.0.1:8765 | head -1
ps -o ppid=,pgid=,tty= -p $NEW
kill $NEW
```

Results here:

```
old=60646 new=60771
PORTDASH_FAKE_VAR=hello
n/private/tmp
<!DOCTYPE HTML>
    1 60771 ??
```

The new server is in its own process group, with no terminal. It stayed up after I closed the terminal session that started the restart.

**Crash on start:** `error: started PID 61027 but port 8795 never came up. See .../8795.log`. The log held the crash message.

**A watcher,** using PIDs rather than `pkill`, so you only stop your own:

```sh
(while true; do python3 -m http.server 8768 --bind 127.0.0.1 >/dev/null 2>&1; sleep 1; done) &
WATCHER=$!
sleep 2
cargo run -- restart 8768
kill $WATCHER
kill $(/usr/sbin/lsof -nP -iTCP:8768 -sTCP:LISTEN -Fp | head -1 | cut -c2-)
```

Result: `error: something else restarted it: PID 61322 now holds port 8768`, with exit 1.

**The limit:**

```sh
/usr/bin/nc -l 127.0.0.1 8793 </dev/null >/dev/null &
cargo run -- restart 8793                    # refused, exit 1, and the server stays up
cargo run -- restart 8793 --use-current-env  # restarts it
```

Both behaved as the comments say. `ps eww` shows no environment for `nc`, so I could confirm only that the second one came up, and not what it received.

**Other cases that worked:** a `node` server came back with its variable and answered `ok`, a hardened binary came back with its variable, and a deleted folder gave `working folder ... no longer exists`. `restart 5432` said nothing was visible on that port.

## Pitfalls

- **Treating zero variables as success.** This is the main trap on this platform. Your test should cover it.
- **Printing the buffer while debugging.** It holds every secret the process has. Use a fake variable and filter.
- **`EINVAL` read as a bug.** For another user's process, the call fails with "Invalid argument", not "permission denied".
- **Too small a buffer.** Size it from the kernel: `kern.argmax` was 1048576 here. A short read cuts off the environment.
- **`Debug` on the raw buffer.** The rules about hiding secrets apply to it as they do to the plan.
- **Printing `ps eww` output anywhere,** including a bug report.

## Changes to "Done when"

Add:

- [ ] A process with no readable environment is refused, with a message that offers the explicit flag
- [ ] Apple's private values never reach the new process
- [ ] The limits table is summarized in your README

## Tested on

macOS 27.2 (build 26B123), Apple M1 Max, 2026-11-02. `libc` 0.2.191. Only processes I started were read, using a fake variable, and no real environment values were printed. `cargo run` results came from a throwaway reference build in a temp folder, not from a finished `portdash`, so your wording will differ.

Not tested: restarting through a TUI key (the reference has this only as a CLI command, tested from a terminal session that I then closed), `~/Library/Logs` (I set `XDG_STATE_HOME` so that logs stayed in a temp folder), real `vite` or `next dev` projects, a real `launchd` service, and an Intel Mac. Why Apple binaries return nothing is a pattern I saw, not a documented rule that I confirmed.
