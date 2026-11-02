# Stage 12: Draw the real table

**Goal:** the TUI displays your actual listeners in a clean table, and a test proves what appears on screen.

## Background

### Separate state from drawing

Create an `App` struct that holds everything the UI needs to know. For now, that is the list of listeners. Your code will now have three distinct roles:

| Role | Signature, roughly | Rule |
|---|---|---|
| **State** | `struct App { ... }` | Plain data |
| **Update** | `fn on_key(&mut self, key)` | Changes state, draws nothing |
| **View** | `fn draw(frame, &mut App)` | Reads state, draws it, performs no system calls |

This split is what makes a TUI testable. You can give a fake `App` to `draw`, and check the output, with no real sockets or a real terminal. Keep to the rule that **`draw` never scans `/proc`**. It runs on every frame, and it must be fast.

### Layout

`Layout` divides a rectangle into smaller ones, using constraints:

- `Constraint::Length(n)` means exactly `n` cells.
- `Constraint::Fill(1)` means take what is left.
- `Constraint::Min(n)` means at least `n`.

A common screen is a body plus one line of footer.

### The `Table` widget

A `Table` takes rows and a list of column widths. Give fixed widths to the columns with predictable sizes (port, PID), and let the variable ones (project) `Fill`. Long text is cut off at the column edge.

### Testing with `TestBackend`

`ratatui` includes a fake terminal, which you can draw to in a test and then read. You could not otherwise test a TUI without a person looking at it.

## Your task

1. Create an `App` struct holding the listeners.
2. Write a `draw` function: bordered table, with a header, plus a footer listing the keys.
3. Include the columns: port, address, PID, process, project. Show a clear placeholder for unknown values, such as `-`.
4. Show a friendly centered message instead of an empty table when nothing is listening.
5. Scan once at startup. Refresh comes in Stage 14.
6. Write tests with `TestBackend`:
   - A listener's port and project appear in the output
   - An empty list shows your empty message
   - A very small terminal (say 10 by 3) does not panic

## Generic example

Drawing a table from state, and testing it:

```rust
use ratatui::{
    Frame,
    layout::Constraint,
    style::{Modifier, Style},
    widgets::{Block, Cell, Row, Table, TableState},
};

struct Book {
    title: String,
    pages: u32,
}

struct App {
    books: Vec<Book>,
    table: TableState,
}

fn draw(frame: &mut Frame, app: &mut App) {
    let rows = app.books.iter().map(|book| {
        Row::new([Cell::from(book.title.as_str()), Cell::from(book.pages.to_string())])
    });

    let table = Table::new(rows, [Constraint::Fill(1), Constraint::Length(6)])
        .header(Row::new(["TITLE", "PAGES"]).style(Style::new().add_modifier(Modifier::BOLD)))
        .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .block(Block::bordered().title("Books"));

    frame.render_stateful_widget(table, frame.area(), &mut app.table);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn shows_book_titles() {
        let mut app = App {
            books: vec![Book { title: "Dune".into(), pages: 412 }],
            table: TableState::default(),
        };
        let mut terminal = Terminal::new(TestBackend::new(30, 6)).unwrap();
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();

        let screen = format!("{:?}", terminal.backend().buffer());
        assert!(screen.contains("Dune"));
        assert!(screen.contains("PAGES"));
    }
}
```

Splitting an area into body and footer:

```rust
use ratatui::{Frame, layout::{Constraint, Layout}, widgets::Paragraph};

fn draw_shell(frame: &mut Frame) {
    let [body, footer] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());
    frame.render_widget(Paragraph::new("main content"), body);
    frame.render_widget(Paragraph::new("q quit"), footer);
}
```

## Verify

```sh
python3 -m http.server 8765 --bind 127.0.0.1 &
cargo run
kill %1
cargo test
```

Now look at it the way a picky user would:

- Are the columns aligned, and the widths sensible?
- Do IPv6 addresses fit? `::ffff:127.0.0.1` is long.
- Resize to very narrow and very wide. What gets cut off first? Is it the right thing?
- Is the footer readable?
- Do colors work in both a light and a dark terminal? Bold and reversed text work everywhere, so they are safer than fixed colors.

Fix what looks wrong. Polish is a large part of why people prefer one tool over another.

## Pitfalls

- **Scanning inside `draw`.** It looks convenient, but later you will be redrawing many times per second.
- **Slicing long strings yourself.** `Table` already cuts off what does not fit, safely. Slicing a `&str` at a byte index may panic on non-ASCII text.
- **Hardcoded colors.** A dark blue on black is unreadable, and your terminal theme is not your users' theme. Prefer modifiers. Use named colors sparingly, and test them.
- **Testing exact layouts.** Asserting whole screens makes tests brittle. Assert that the important content appears.

## Done when

- [ ] Real listeners appear in a table with a header
- [ ] Nothing listening produces a friendly message
- [ ] Three `TestBackend` tests pass
- [ ] A very small terminal does not panic
- [ ] `draw` makes no system calls
- [ ] Committed
