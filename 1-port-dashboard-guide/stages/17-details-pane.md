# Stage 17: The details pane

On a Mac? Also read [this stage's Mac notes](17-details-pane-mac-support.md).

**Goal:** a panel shows everything known about the selected row: full command, working directory, owner, uptime, memory and parent.

## Background

### Why a separate pane

The table is for scanning, so it must stay narrow. A command line can be hundreds of characters. The details pane is where it can be read in full, with wrapping. The user gets the short view by default, and the long view without leaving it.

### What belongs here

- **Command:** the entire line, wrapped.
- **Working directory and project root.**
- **Owner:** user name.
- **PID and parent:** with the parent's name. "Parent: nodemon" explains why a server came back after you killed it.
- **Uptime:** readable, as in `2h 14m`.
- **Memory.**
- **All PIDs** when several processes share the socket.

### Memory

`/proc/<pid>/status` is a text file with `Key: value` lines. The line you want:

```
VmRSS:     48212 kB
```

**RSS** (resident set size) is how much physical memory the process is using. It over-counts pages shared with other processes, but it is the standard number, and fine for this purpose.

It is missing for kernel threads, so make it optional. Convert to a readable unit (`47 MB`), and test the conversion.

### Fetching data without blocking

The table holds data from your background scan. Memory changes often, and a parent's name needs another read. There are two reasonable approaches:

1. **Include it in the scan.** Simple: everything arrives together. It may be slightly stale, but only by a refresh interval.
2. **Read on selection.** It seems more efficient, but `draw` must not do system calls (Stage 12), and so this approach requires more machinery.

Pick the first. It is simple, and your `/proc` reads are cheap. If your Stage 14 measurement says otherwise, look again then.

### Layout decisions

Show the pane below the table, using a `Layout` split. Let the user toggle it with a key, such as `Tab` or `i`. Consider short terminals: the table must still have room. When there is not enough height, hide the pane automatically.

## Your task

1. Add the extra fields to your data and to `scan()`.
2. Write and test two pure functions:
   - Parse `VmRSS` from `status` text
   - Format a duration: `59s`, `1h 1m`, `1d 1h`
3. Draw the pane below the table, showing the selected row's details.
4. Wrap long lines.
5. Add a toggle key, and auto-hide when the terminal is short.
6. Handle no selection and unknown owner clearly. For an unknown owner, say what you know (the port and UID), and why the rest is missing.
7. Add `TestBackend` tests:
   - The selected row's command appears
   - A very long command does not panic
   - A short terminal still shows the table

## Generic example

Splitting the screen, with a wrapped paragraph:

```rust
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    widgets::{Block, Paragraph, Wrap},
};

fn draw_split(frame: &mut Frame, synopsis: &str) {
    let [top, bottom] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(8)]).areas(frame.area());

    frame.render_widget(Block::bordered().title("Books"), top);

    let details = Paragraph::new(synopsis)
        .wrap(Wrap { trim: false })
        .block(Block::bordered().title("Synopsis"));
    frame.render_widget(details, bottom);
}
```

Readable formatting, using range patterns:

```rust
fn cooking_time(minutes: u64) -> String {
    match minutes {
        0..60 => format!("{minutes}m"),
        60..1440 => format!("{}h {}m", minutes / 60, minutes % 60),
        _ => format!("{}d {}h", minutes / 1440, minutes % 1440 / 60),
    }
}
```

Finding a value in `Key: value` text:

```rust
fn lookup<'a>(text: &'a str, wanted: &str) -> Option<&'a str> {
    text.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        (key == wanted).then(|| value.trim())
    })
}
```

## Verify

```sh
cd /tmp && python3 -m http.server 8765 --bind 127.0.0.1 &
cargo run
kill %1
```

- Compare with reality:

  ```sh
  ps -o pid,ppid,user,rss,etime,args -p <pid>
  ```

  `ps` reports RSS in KiB, which should match `VmRSS`, and `etime` should match your uptime.

- Move through the rows. The pane updates at once, with no lag.
- Try a long command. Python makes this easy:

  ```sh
  python3 -m http.server 8770 --bind 127.0.0.1 --directory /tmp/$(printf 'x%.0s' {1..150}) 2>/dev/null
  ```

  The directory must exist for the server to stay up, so `mkdir` it first.
- Shrink the window until the pane goes away. The table must still be usable.

## Pitfalls

- **Multi-line command output in a one-line slot.** A command may contain newlines. Replace them before rendering.
- **Secrets in commands.** Command lines may contain passwords and tokens, such as `--password=...`. You are displaying only what `ps` already shows to the user, so this is not a new exposure, but be aware of it if you ever add logging or a screenshot feature.
- **Calculating uptime with `draw`-time reads.** Reading `/proc/uptime` during `draw` breaks your rule. Store a start time, and calculate from it using `Instant` or the scan.
- **Scroll position with variable text.** A wrapped paragraph may exceed its pane. Cutting it off is fine for now.

## Done when

- [ ] Selected row details appear and match `ps`
- [ ] Long commands wrap with no panic
- [ ] The toggle works, and short terminals hide the pane
- [ ] The parsing and formatting functions are tested
- [ ] `draw` still does no system calls
- [ ] Committed
