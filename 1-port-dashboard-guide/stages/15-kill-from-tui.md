# Stage 15: Kill from the dashboard

On a Mac? Also read [this stage's Mac notes](15-kill-from-tui-mac-support.md).

**Goal:** select a row, press a key, confirm in a popup, and the server stops while the UI stays responsive and reports the result.

## Background

### Modes

Your app now has more than one state of interaction. While a popup is open, `y` means "yes", and arrow keys should do nothing to the table. An enum describes this exactly:

```
Mode::Browse              the normal view
Mode::ConfirmKill(target) a popup is waiting for an answer
```

`on_key` starts by checking the mode. This pattern scales well: Stage 16 adds a filter mode the same way. Using an enum instead of a few booleans means invalid combinations cannot exist, such as "confirm open and filter open at once".

### Capture the target when asking

The confirm popup may stay open for a while, and the refresh keeps running behind it. If you resolve "the selected row" when the user presses `y`, the row may have changed underneath them.

So store the specific target (PID, start time, port, name) in the mode when the popup opens. That is what gets stopped, and it is what the popup displays. Then the existing start time check from Stage 10 covers the rest.

### Do not block the UI

Stopping can take seconds, because of the grace period from Stage 10. If you call that on the main thread, the whole app freezes. You already know the solution: another thread, and a message when it finishes.

While it is underway, show that something is happening, for example `Stopping shop-frontend...` in the status bar.

### Status messages

Since you can no longer `println!` (Stage 11), results need a place in the UI. Add a status line, with a success or error message. It should go away on its own after a few seconds, or on the next action. One easy approach is to store the message along with an `Instant`, and have `draw` skip it once it is old enough.

But note that nothing redraws the screen in an idle app, except the scan arriving every couple of seconds. Is that often enough for your timeout? Probably yes, and it costs nothing. Decide consciously.

### Reuse, do not rewrite

Stage 10 already has validation, the start time check, and escalation. Move that into a function returning a result type, which both the CLI and the TUI call. Do not make a second copy of safety code. If you find CLI prompts or `println!` inside it, those belong in the caller.

### Popups

A popup is drawing on top of what was already drawn. `ratatui` draws in order, so draw it last. Use the `Clear` widget first, to wipe out the cells below. Without it, table text shows through.

## Your task

1. Add a `Mode` enum to `App`, and route `on_key` through it.
2. Pressing your kill key on a row opens a popup that names the target clearly: port, process, project, PID.
3. `y` confirms. `n` or `Esc` cancels. Anything else does nothing. Make the default safe: `Enter` should not confirm unless you have decided so on purpose.
4. Run the stop on a background thread, using the shared function, and send back the result as an event.
5. Show progress and results in a status line.
6. Handle rows with no known owner by showing a message, and not opening a popup.
7. Trigger a refresh after the stop finishes, so the row goes away promptly.
8. Test the logic, with no terminal:
   - The kill key on a known row enters the confirm mode, with the correct target
   - `Esc` returns to browse mode with no action
   - Arrow keys do not move the selection while the popup is open
   - The target stays the same after a refresh arrives during confirm mode

## Generic example

Modes and a centered popup:

```rust
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Block, Clear, Paragraph},
};

enum Mode {
    Browse,
    ConfirmDelete { book_id: u32, title: String },
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let [row] = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center).areas(area);
    let [cell] = Layout::horizontal([Constraint::Length(width)]).flex(Flex::Center).areas(row);
    cell
}

fn draw_popup(frame: &mut Frame, mode: &Mode) {
    if let Mode::ConfirmDelete { title, book_id } = mode {
        let area = centered(frame.area(), 40, 5);
        let text = format!("Delete {title} (#{book_id})?\n\ny: delete    n: cancel");
        frame.render_widget(Clear, area);
        frame.render_widget(Paragraph::new(text).block(Block::bordered().title("Confirm")), area);
    }
}
```

## Verify

**A polite server:**

```sh
python3 -m http.server 8765 --bind 127.0.0.1 &
```

In the dashboard, select it, press the kill key, and press `y`. The row should disappear, with a success message.

**The stubborn server from Stage 10:** run it, and during its grace period keep pressing the arrow keys. They must still work, because the UI is not frozen.

**Other situations:**

- Press the kill key, then `Esc`: nothing happens, and the server stays up.
- Open the popup, and in another terminal stop the server yourself. Then press `y`. You should get a clear message, not a crash.
- Try a port owned by another user. You should see a clear permission or unknown owner message.
- Make the terminal small, and open the popup. It must not panic. Check how `centered` behaves when the area is smaller than the popup.

## Pitfalls

- **Finding the target by row index at confirm time.** It is the dangerous bug described above.
- **Duplicated stop logic.** Two copies of safety checks eventually drift apart. Share it.
- **Text showing through the popup.** You forgot `Clear`.
- **Popup text longer than its box.** Project names vary. Wrap or truncate them, and test a very long name.
- **Pressing the kill key twice.** What if a user starts a stop while one is already running, on the same process? Decide what happens.

## Done when

- [ ] Kill works end to end with a confirmation
- [ ] The UI stays responsive during the grace period
- [ ] The target is captured when the popup opens
- [ ] CLI and TUI share one stop function
- [ ] Success and failure both appear in the status line
- [ ] The four tests pass
- [ ] Committed
