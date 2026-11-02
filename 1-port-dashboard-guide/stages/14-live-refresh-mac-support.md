# Stage 14 on a Mac: Live refresh

The threads, channels and selection logic are the same. What changes is the cost of a scan, and the way you measure it.

## What differs on a Mac, and why

**A scan is now several child processes.** On Linux, `scan()` reads files inside your own process. Here it starts `lsof` and `ps`. I measured each combination 20 times on an otherwise normal machine, adding the children's CPU time:

| One scan runs | CPU per scan | Wall time | Share of one core at every 2 s |
|---|---|---|---|
| `lsof` for sockets | 52 ms | 54 ms | 2.6% |
| Plus `ps` for numbers | 65 ms | 69 ms | 3.3% |
| Plus `lsof` for working folders and `ps` for commands | 124 ms | 133 ms | 6.2% |

A full TUI doing all of it ran for 53.7 seconds and used 1.65 s user plus 2.25 s system, which is **7.3% of one core** while idle. On this 10-core machine that is under 1% of the total, but it is well above what `/proc` costs, and a laptop feels it.

With 58 listeners, or with 517, a full scan still took about 130 ms, so the process count matters far more than the socket count.

**`top` will mislead you.** It shows only your process, which mostly sleeps. It reported **0.0%** for the same TUI that was really using 7.3%, because the work happens in short-lived children.

**The `top` flag differs too.** `top -p PID` fails here with `invalid option or syntax: -p`.

## Your task on a Mac

Add one task:

7. Stop re-reading what cannot change. A process's working folder rarely changes and its command never does, so keep them in a map keyed on (PID, `lstart`). Each scan runs `lsof` and one batched `ps`, and only asks for working folders and commands for new keys. Forget keys that have gone, so the map cannot grow forever.

Then measure again, the way shown below. If the number still bothers you, this is the moment to look at `libproc` behind `scan()`: about 5 ms per scan in my test, in exchange for `unsafe` code, and it does not provide working folders on macOS (Stage 2 compares them).

## Generic example

A cache keyed on something that can be reused, so the key needs a second part:

```rust
use std::collections::HashMap;

/// A library card: the number can be reissued, so it is only unique together with its issue date.
type CardKey = (u32, String);

#[derive(Default)]
struct BorrowerCache {
    names: HashMap<CardKey, String>,
    lookups: u32,
}

impl BorrowerCache {
    fn name(&mut self, card: u32, issued: &str) -> String {
        let key = (card, issued.to_string());
        if let Some(name) = self.names.get(&key) {
            return name.clone();
        }
        self.lookups += 1; // The slow part: a real program would run a tool or read a file here.
        let name = format!("borrower-{card}");
        self.names.insert(key, name.clone());
        name
    }

    /// Forget cards that are no longer active, so the map cannot grow forever.
    fn keep_only(&mut self, active: &[CardKey]) {
        self.names.retain(|key, _| active.contains(key));
    }
}

fn main() {
    let mut cache = BorrowerCache::default();
    cache.name(7, "Mon");
    cache.name(7, "Mon");
    assert_eq!(cache.lookups, 1);
    cache.name(7, "Tue"); // The number was reissued, so this is a new card.
    assert_eq!(cache.lookups, 2);
    cache.keep_only(&[(7, "Tue".to_string())]);
    assert_eq!(cache.names.len(), 1);
    println!("ok");
}
```

## Verify

Replace the `top` step. The real measure counts children, and `/usr/bin/time` includes any it has waited for:

```sh
cargo build --release
/usr/bin/time -p ./target/release/portdash
# Leave it for about a minute, then press q
# real 53.72
# user 1.65
# sys 2.25
```

Add `user` and `sys`, then divide by `real`. Before caching, this gave 7.3% here. An uncached scan, as measured above, is your baseline. Expect yours to fall toward the 3.3% row.

To see what `top` reports, run this in a second terminal:

```sh
top -l 2 -s 2 -pid $(pgrep -x portdash) -stats pid,cpu,command | tail -1
# 58629  0.0  portdash
```

The server appearing and disappearing with no keypress worked as written. `kill %1` needs an interactive shell, as in the original.

## Pitfalls

- **Reporting the `top` number as your idle cost.** Your "Done when" asks for a number you measured, so it should be the `time` one.
- **Children you never wait for.** `Command::output` waits for you. Spawning and forgetting would leave zombies in your own table.
- **A cache keyed on PID alone.** PIDs get reused, as the example's card numbers do.
- **Scanning more often to feel responsive.** At 1 second, every figure above doubles.

## Changes to "Done when"

Change "Idle CPU use is low, with a number you measured" to:

- [ ] Idle CPU is measured with `/usr/bin/time`, children included, and working folders and commands are cached

## Tested on

macOS 27.2 (build 26B123), Apple M1 Max, 2026-11-02. ratatui 0.30.2, crossterm 0.29.0. `cargo run` results came from a throwaway reference build in a temp folder, not from a finished `portdash`, so your wording will differ. I drove the reference TUI through tmux.

Not tested: the caching task in a running app. The 3.3% row measures the two commands that remain, not a finished cache. The slow-scan freeze test does not depend on the operating system and was not repeated. Terminal.app and an Intel Mac were not tested.
