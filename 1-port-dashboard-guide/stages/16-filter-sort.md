# Stage 16: Filter and sort

On a Mac? Also read [this stage's Mac notes](16-filter-sort-mac-support.md).

**Goal:** type `/` to filter rows, press a key to change the sort, and hide system noise by default.

## Background

### Filtering is about what you do not show

On a typical machine, half the listeners are system services: CUPS, DNS, databases, and so on. A dev dashboard should focus on your servers. You now have the data to do that.

Three controls would help:

1. **A text filter.** `/` opens a prompt, and typing narrows the rows live, matching on port, process and project.
2. **A "mine only" toggle.** It hides sockets owned by other users. This is the best filter for very little code, since you have the UID from Stage 3. It probably should be on by default. Your own UID comes from `nix::unistd::getuid()`.
3. **A sort key.** One keypress cycles through port, process, project and uptime.

### Derived state

Do not throw away data when filtering. Keep the full list, and compute what is visible from it.

```
all listeners  -->  filter  -->  sort  -->  visible rows
```

This creates an important trap: `TableState`'s selected index now refers to **visible** rows, not to the full list. Any lookup with `all_listeners[selected]` returns the wrong item as soon as a filter is active. In this app, that means killing the wrong process. Have one single method that returns the selected listener, and use it everywhere.

Compute the visible list whenever its inputs change, or compute it fresh when needed. With a few dozen rows, either is fast enough, and fresh is simpler, with no cached copy to get out of date.

### Text input mode

This is another `Mode` variant from Stage 15. While filtering:

- Printable characters go into the query, including `q` and `j`. They must not trigger their usual actions.
- `Backspace` deletes one character. `String::pop` handles this, even for multi-byte characters.
- `Esc` clears the filter and leaves.
- `Enter` keeps the filter and leaves, so that normal keys work again.

The list should narrow as you type, on each keystroke. That instant feedback is the entire point of `/` filtering.

### Keeping the selection when the view changes

You solved this in Stage 14. Filtering and sorting are just more ways of moving rows, so reuse that logic. If the selected row is filtered out, fall back sensibly.

### Stable sorting

`sort_by` in Rust keeps the original order of equal elements. But when sorted by process name, you have five `node` rows. Add a tie-breaker, like port, so rows do not shuffle between refreshes. `Ordering::then_with` chains comparisons.

Always show the current sort and filter in the UI. It should never be a mystery why rows are missing. "3 of 12 shown" is very helpful.

## Your task

1. Add filter text, a sort key and a "mine only" flag to `App`.
2. Write a method returning the visible rows, plus one that returns the selected listener.
3. Change every use of the old list to go through those methods.
4. Add `Mode::Filter`, with the behavior above.
5. Add keys for cycling the sort and toggling "mine only".
6. Show the filter, sort, and shown/total count.
7. Show a distinct empty message when rows exist, but the filter hides all of them. It is a very different situation from "nothing is listening".
8. Add tests:
   - Matching ignores case
   - A filter on the port number matches, for example `30` matches `3000`
   - With a filter active, the "selected listener" method returns the visible row and not the row at that index in the full list. **This test matters most.**
   - Typing `q` in filter mode adds to the query, and does not quit
   - Equal keys always sort in the same order

## Generic example

Deriving a filtered and sorted view, without changing the source:

```rust
use std::cmp::Reverse;

struct Book {
    title: String,
    pages: u32,
}

fn visible<'a>(books: &'a [Book], query: &str) -> Vec<&'a Book> {
    let query = query.to_lowercase();
    let mut hits: Vec<&Book> = books
        .iter()
        .filter(|book| book.title.to_lowercase().contains(&query))
        .collect();
    hits.sort_by_key(|book| Reverse(book.pages));
    hits
}
```

Sorting with a tie-breaker:

```rust
struct Song {
    artist: String,
    track: u32,
}

fn sort_songs(songs: &mut [Song]) {
    songs.sort_by(|a, b| a.artist.cmp(&b.artist).then_with(|| a.track.cmp(&b.track)));
}
```

## Verify

```sh
for port in 3000 3001 8000 8765; do python3 -m http.server $port --bind 127.0.0.1 >/dev/null 2>&1 & done
cargo run
pkill -f 'http.server (3000|3001|8000|8765)'
```

- `/` then `30` shows only 3000 and 3001.
- With the filter on, stop the row shown. **Confirm the popup names the same server you highlighted.** Check the PID.
- `Esc` restores all rows.
- Each sort key changes the order as expected, and the selected server stays selected.
- Toggling "mine only" shows and hides system services.
- During refreshes, a filtered view stays filtered and does not flicker.

## Pitfalls

- **The wrong-row bug.** Searching your code for every direct index into the list is worth the two minutes.
- **Filter keys triggering actions.** A filter for `kill` must not trigger your kill key. Your mode check takes care of it, if every key goes through it.
- **Case folding.** `to_lowercase` allocates. That is fine at this scale, and no reason to add complexity.
- **Forgetting numbers.** Users will search by port. Include `port.to_string()` in what you match.
- **Sort direction for uptime.** Think about which direction is useful. Newest first is likely what you want.

## Done when

- [ ] Filter narrows live on each keystroke
- [ ] "Mine only" works, and hides system services
- [ ] Sort cycling works, with stable ordering
- [ ] The shown/total count and the active filter are visible
- [ ] Killing with an active filter stops exactly the highlighted row, with a test
- [ ] Committed
