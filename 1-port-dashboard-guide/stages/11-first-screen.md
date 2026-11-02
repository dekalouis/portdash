# Stage 11: Your first screen

**Goal:** running `portdash` with no arguments opens a full-screen app that quits on `q`, and always leaves your terminal in good shape.

## Background

### How a terminal app takes over the screen

A normal terminal echoes what you type, sends input line by line, and scrolls. A TUI changes these settings:

- **Raw mode** delivers every keypress at once, without echoing it. In this mode `Ctrl+C` no longer interrupts automatically, and your app has to handle it.
- **The alternate screen** is a separate buffer, like the one `vim` and `less` use. When it is closed, your previous terminal contents come back untouched.

### Restore the terminal, no matter what

If your program crashes in raw mode without undoing it, the user's shell is left broken: nothing echoes when they type. This is the classic TUI bug. You can recover with `reset`, but your users should never need to. Always restore on normal exit, on errors, and on panics.

### `ratatui` and `crossterm`

- **`ratatui`** draws widgets into a buffer, and then updates only the cells that changed.
- **`crossterm`** handles the low-level terminal work and keyboard input. `ratatui` re-exports it, but using it directly for events is common.

`ratatui::init()` turns on raw mode and the alternate screen, and it installs a panic hook that restores the terminal. `ratatui::restore()` undoes it. Between them runs your main loop.

### Immediate mode rendering

`ratatui` has no persistent "button object" or "label object". Instead, on every frame you describe the complete screen from your current data, and the library works out what changed. It is the same idea as React. This makes the app simpler, since the screen can never get out of step with your state: it is always computed from it.

### The main loop

Nearly every TUI has this structure:

```
loop:
    draw the screen from the current state
    wait for an event
    update the state
    exit if asked
```

## Your task

1. Add `ratatui` and `crossterm`:

   ```sh
   cargo add ratatui crossterm
   ```

2. Create a module `src/tui.rs` with a `run()` function.
3. Display the app name and "Press q to quit".
4. Quit on `q`, `Esc` and `Ctrl+C`.
5. Run the TUI when no subcommand is given. `portdash list` still works as before.
6. If stdout is not a terminal, for example `portdash | cat`, do not start the TUI. Print a helpful message and exit non-zero. `std::io::IsTerminal` tells you.

## Generic example

A minimal app, which always restores the terminal, even if the loop returns an error:

```rust
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::widgets::Paragraph;
use std::io;

fn run_app() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let result = loop {
        let drawn = terminal.draw(|frame| {
            frame.render_widget(Paragraph::new("Weather: sunny. Press x to leave."), frame.area());
        });
        if let Err(err) = drawn {
            break Err(err);
        }

        match event::read() {
            Ok(Event::Key(key))
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('x') =>
            {
                break Ok(());
            }
            Ok(_) => {}
            Err(err) => break Err(err),
        }
    };

    ratatui::restore();
    result
}
```

The key point is that the loop `break`s with its result instead of using `?`. A `?` inside the loop would return early and skip `ratatui::restore()`.

`Ctrl+C` is `KeyCode::Char('c')` with `KeyModifiers::CONTROL` set in `key.modifiers`.

## Verify

```sh
cargo run             # screen appears, q quits, your shell history is back
cargo run             # Esc quits
cargo run             # Ctrl+C quits
cargo run -- list     # still works
cargo run | cat       # a message, not a broken screen
```

**Test a panic.** Temporarily add `panic!("test")` inside the loop after the first draw, then run it:

- Your terminal should still work normally afterward, and typing should echo.
- The panic message should be readable.

Remove the panic when you are done.

**Resize your terminal window** while it runs. Since a resize event wakes the loop and you redraw from scratch, the screen should adjust.

## Pitfalls

- **Using `?` between init and restore.** It skips the restore. Your panic test does not catch this, so test an error path as well, or use the `break` pattern.
- **Key press and release.** Some terminals report key release and repeat as separate events, particularly with enhanced keyboard protocols. Check `key.kind == KeyEventKind::Press`, or a single keypress may act twice.
- **`println!` during the TUI.** It writes over your screen. Nothing may print to stdout or stderr while the TUI runs. Stage 15 shows how to report messages in the UI.
- **Debugging with a broken terminal.** If you do get stuck in raw mode, type `reset` and press Enter, even though you will not see the letters.

## Done when

- [ ] `portdash` opens a full-screen view
- [ ] All three quit keys work, and the terminal comes back clean
- [ ] A panic leaves the terminal working
- [ ] `portdash list` still works
- [ ] `portdash | cat` prints a message instead of a broken screen
- [ ] Committed
