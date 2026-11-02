# Stage 17 on a Mac: The details pane

## What differs on a Mac, and why

**Memory comes from `ps`.** There is no `/proc/<pid>/status` or `VmRSS` line. The `rss=` column from Stage 7 is in KiB, and you already fetch it, so this stage costs no new command.

**Uptime is already computed.** Stage 7's `etime` gives seconds directly, so the `/proc/uptime` pitfall does not apply. Keep the rule behind it: `draw` makes no system calls. To keep uptime ticking between scans, store the scan's `Instant` along with the seconds.

**Paths are real paths.** A server started in `/tmp` shows `/private/tmp`, and users may ask why. Showing the real path is fine.

**Unknown owners do not appear.** Task 6 asks for a message about what you know for such a row. Without root there are no such rows on a Mac (Stage 6). Keep the code path for root and Linux.

## Your task on a Mac

Change task 2:

2. Write and test these two pure functions:
   - Format a KiB value: `21872` becomes `21 MB`, and a missing value stays a clear placeholder
   - Format a duration, exactly as in the original

`VmRSS` parsing is Linux code, so keep it behind `#[cfg(target_os = "linux")]` if you have it.

## Verify

```sh
(cd /tmp && exec python3 -m http.server 8765 --bind 127.0.0.1) &
SERVER=$!
cargo run
ps -o pid,ppid,user,rss,etime,args -p $SERVER
kill $SERVER
```

Results here, and the pane's values matched:

```
  PID  PPID USER    RSS ELAPSED ARGS
60210 59020 you   21872   00:01 /Users/you/.pyenv/.../python3 -m http.server 8765 --bind 127.0.0.1
```

The reference tool reported `21872` KiB and `00:01`. Both come from `ps`, so this check confirms your plumbing and not the kernel's number.

**A long command.** The original's `/tmp/$(printf 'x%.0s' {1..150})` also works in zsh. Make the folder first:

```sh
LONG=/tmp/$(printf 'x%.0s' {1..150})
mkdir -p "$LONG"
python3 -m http.server 8770 --bind 127.0.0.1 --directory "$LONG" >/dev/null 2>&1 &
SERVER=$!
cargo run
kill $SERVER
```

I ran this with the long folder inside my temp directory. In an 80-column terminal the command wrapped across several lines, with no panic, and was cut off at the pane's edge.

## Pitfalls

- **Comparing RSS across systems.** macOS counts memory differently from Linux, so similar programs may show different numbers.
- **Using `ps -o comm=` for the name.** It gives a full path here. Keep the `c` value.

## Changes to "Done when"

Change "The parsing and formatting functions are tested" to:

- [ ] The KiB and duration formatting functions are tested

## Tested on

macOS 27.2 (build 26B123), Apple M1 Max, 2026-11-02. I drove the reference TUI through tmux.

Not tested: the auto-hide behavior when a terminal is short, because the reference has a fixed layout. Unknown-owner display as root, Terminal.app and an Intel Mac were not tested.
