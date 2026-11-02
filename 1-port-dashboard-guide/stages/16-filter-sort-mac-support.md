# Stage 16 on a Mac: Filter and sort

Filtering and sorting work as written. The "mine only" toggle does not do what the stage promises.

## What differs on a Mac, and why

The stage calls "mine only" the best filter for very little code, since it hides system services. Without root on a Mac, `lsof` already shows only your own processes, so every row is yours. Here, the toggle changed nothing:

```
21 of 21 shown  mine:true
21 of 21 shown  mine:false
```

The noise you see is different as well. It consists of your own background apps and Apple's per-user services, such as `ControlCenter` and `rapportd`, and "mine only" cannot remove them.

## Your task on a Mac

Change task 5:

5. Keep the toggle, because it matters under `sudo`, but make its state honest. When you are not root, show that other users' ports are not visible at all, rather than implying a filter is hiding them.

Add one more control if the noise bothers you: a filter that hides rows with no project. That removes `ControlCenter` and friends, and it needs only the data you already have.

## Verify

Replace `pkill` with PIDs you control, so that you only stop your own test servers:

```sh
SERVERS=()
for port in 3000 3001 8000 8765; do
  python3 -m http.server $port --bind 127.0.0.1 >/dev/null 2>&1 &
  SERVERS+=($!)
done
cargo run
kill "${SERVERS[@]}"
```

Results here:

- `/` then `30` showed `2 of 21 shown`, with only 3000 and 3001.
- With the filter on, the popup named `PID 60054 on 3000`, the highlighted row.
- "Mine only" changed nothing, as shown above. The guide's "shows and hides system services" holds only as root.

The `pkill -f 'http.server (3000|3001|8000|8765)'` form in the original does work with macOS `pkill`: its alternation matched and stopped a test server of mine. It can also match any similar server of yours, so I prefer the PID list.

## Pitfalls

- **A toggle that looks broken.** Users press it, nothing changes, and they assume a bug. Say why in the UI.
- **Sorting by uptime.** Sort on the seconds value from Stage 7, not the `etime` text, where `04-21:18:54` sorts before `59:00`.

## Changes to "Done when"

Change `"Mine only" works, and hides system services` to:

- [ ] "Mine only" works when run as root, and the UI explains what you cannot see otherwise

## Tested on

macOS 27.2 (build 26B123), Apple M1 Max, 2026-11-02. I drove the reference TUI through tmux. It has only the filter, toggle and popup, so sort keys were not exercised.

`sudo lsof` showed other users' rows on this machine, so the toggle would have something to hide as root, but I did not run the tool itself as root. Sort cycling, Terminal.app and an Intel Mac were not tested.
