# Stage 7 on a Mac: Read process details

## What differs on a Mac, and why

There are no per-process files to read, so each value comes from `ps` or `lsof`:

| Linux | Mac | Notes |
|---|---|---|
| `/proc/<pid>/comm` | `c` field from Stage 3 | Up to about 31 characters here. `ps -o comm=` prints a full path, so it is not the same thing |
| `/proc/<pid>/cmdline` | `ps -o command= -p PID` | One display string, so you cannot tell arguments apart. Stage 18 has the exact form |
| `/proc/<pid>/cwd` | `lsof -a -p PID -d cwd -Fn` | The path is on the line starting with `n` |
| `stat` parent PID | `R` field, or `ps -o ppid=` | |
| `stat` start time | `ps -o lstart=` | One-second resolution |
| `/proc/uptime` arithmetic | `ps -o etime=` | Already the process's age |

Ask for many PIDs at once, because it costs one process instead of many:

```sh
LC_ALL=C ps -o pid=,ppid=,uid=,rss=,etime=,stat=,lstart= -p 53077,53078
# 53077 51989   501   3104 00:02 Ss   Mon Nov  2 11:29:05 2026
# 53078 53077   501  21904 00:02 SN   Mon Nov  2 11:29:05 2026
```

### The weird-name trap moves

`stat` is gone, so the last-`)` trick is too, and `lsof -F` puts the name on its own line, safe from spaces and brackets. A binary named `my (app)` showed correctly here.

The new trap is the column layout. `lstart` is exactly five words, so put it last and rebuild it from the fields you have left. Never ask `ps` for `command=` in the same call, since that text can contain anything. Make a separate call for it.

### Locale changes `lstart`

```sh
LC_ALL=fr_FR.UTF-8 ps -o lstart= -p 53078
# lun.  2 nov. 11:29:05 2026
```

Set `LC_ALL=C` on the command. You will only compare this value for equality, so do not parse it into a date.

### `etime` has a variable shape

`MM:SS`, `HH:MM:SS`, or `DD-HH:MM:SS`. All three appeared here, such as `00:02` and `04-21:18:54`.

### Identity

The pair (PID, `lstart`) replaces (PID, start time ticks). It is coarser: a PID reused within the same second would match, which is very unlikely. Note this as a `shortcut:` comment when Stage 10 relies on it.

### Permissions differ from Linux here too

`ps` works on other users' processes, but `lsof -d cwd` returns nothing for them, with exit code 1 and no error text. A missing PID also gives exit 1. Treat a missing working folder as `None`, as in the original.

### Zombies

A zombie shows `Z` as the first letter of `stat`, and `<defunct>` as its name. `lsof -d cwd` returns nothing for it.

### `nix` features

You do not need `sysconf` on a Mac, because nothing is in clock ticks. This is enough for `User::from_uid`:

```toml
nix = { version = "0.31", features = ["user"] }
```

## Your task on a Mac

Replace tasks 2, 3 and 4:

2. Write a pure function that parses one row of the `ps` output above, and test it:
   - A normal row works
   - A row with too few fields gives `None`
   - `stat` of `SN` and `Z` are both kept as text
3. Write a pure function that turns `etime` into seconds, and test all three shapes.
4. Write a function that gathers details for a list of PIDs: one `ps` call for the numbers, one `lsof` call with `-p a,b,c` for working folders, and one `ps` call for commands.

Stage 18 adds the argument-by-argument form. For this stage, the display string is enough.

## Generic example

Fixed fields followed by a date of exactly five words, and a time with optional parts:

```rust
#[derive(Debug, PartialEq)]
struct Loan {
    id: u32,
    pages: u32,
    due: String,
}

/// Parses "17 412 Mon Nov  2 11:29:05 2026": two numbers, then a date of exactly five words.
fn parse_loan(line: &str) -> Option<Loan> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() != 7 {
        return None;
    }
    Some(Loan {
        id: fields[0].parse().ok()?,
        pages: fields[1].parse().ok()?,
        due: fields[2..].join(" "),
    })
}

/// Turns a baking time written as "[days-][hours:]minutes:seconds" into seconds.
fn bake_seconds(text: &str) -> Option<u64> {
    let (days, clock) = match text.split_once('-') {
        Some((days, clock)) => (days.parse::<u64>().ok()?, clock),
        None => (0, text),
    };
    let parts: Vec<u64> = clock
        .split(':')
        .map(|part| part.parse().ok())
        .collect::<Option<_>>()?;
    let (hours, minutes, seconds) = match parts[..] {
        [hours, minutes, seconds] => (hours, minutes, seconds),
        [minutes, seconds] => (0, minutes, seconds),
        _ => return None,
    };
    Some(days * 86_400 + hours * 3_600 + minutes * 60 + seconds)
}

fn main() {
    let loan = parse_loan("17 412 Mon Nov  2 11:29:05 2026").unwrap();
    assert_eq!(loan.due, "Mon Nov 2 11:29:05 2026");
    assert_eq!(parse_loan("17 412"), None);
    assert_eq!(bake_seconds("05:30"), Some(330));
    assert_eq!(bake_seconds("01:00:10"), Some(3_610));
    assert_eq!(bake_seconds("2-00:00:01"), Some(172_801));
    assert_eq!(bake_seconds("soon"), None);
    println!("ok");
}
```

## Verify

```sh
(cd /tmp && exec python3 -m http.server 8765 --bind 127.0.0.1) &
SERVER=$!
cargo run
ps -o pid,ppid,comm,args -p $SERVER
kill $SERVER
```

Results here:

```
pid 56922, ppid 56759, name python3.13, cwd /private/tmp, uptime 2s, stat SN
56922 56759 /Users/you/.pyenv/versions/3.13.12/bin/python3 ... -m http.server 8765 --bind 127.0.0.1
```

- The working folder prints as **`/private/tmp`**, not `/tmp`. `/tmp` is a symlink here, and that is the real path.
- Uptime rose to 4 on a second run two seconds later.
- `ps` prints the path in `COMM`. Yours should still show the `c` name.

## Pitfalls

- **Comparing `/tmp` with what `lsof` returned.** Canonicalize both sides. `/var` and `$TMPDIR` work the same way.
- **Trusting `cwd` to exist.** After I deleted a server's folder, `lsof` still printed the old path, with no ` (deleted)` marker. Check `Path::exists` when it matters.
- **One `ps` per PID.** It works, but costs a process each time, and Stage 14 pays that every two seconds.
- **`ps` exit 1 for a gone PID.** That is normal. Rows are simply missing from a batch.

## Changes to "Done when"

Replace "The weird-name test passes" with:

- [ ] A name with spaces and brackets displays correctly
- [ ] All three `etime` shapes are tested
- [ ] `ps` runs with `LC_ALL=C`

## Tested on

macOS 27.2 (build 26B123), Apple M1 Max, 2026-11-02. `lsof` 4.91, `nix` 0.31.3. `cargo run` results came from a throwaway reference build in a temp folder, not from a finished `portdash`, so your wording will differ.

Not tested: an Intel Mac. `User::from_uid` compiled in the reference but was not exercised for another user's UID.
