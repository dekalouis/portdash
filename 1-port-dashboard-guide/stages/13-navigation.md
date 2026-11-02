# Stage 13: Move through the list

**Goal:** arrow keys and `j`/`k` move a highlighted selection, with scrolling that works when there are more rows than fit.

## Background

### `TableState`

The `Table` widget does not remember which row is selected. `TableState` holds that, along with the scroll offset, and you pass it each time you draw. That is why `draw` takes `&mut App`: rendering may adjust the offset to keep your selected row on screen. So scrolling is handled for you, as long as you use `render_stateful_widget`.

### Key handling as a pure function

Put all key handling in a method, such as `App::on_key(&mut self, key: KeyEvent)`. It only changes state. That makes it testable with no terminal at all: construct an `App`, call `on_key` with a fake key, and check the state.

Also include a "should quit" field in `App`, and have the main loop check it. That moves the quit logic into `on_key`, with everything else.

### Selection edge cases

What should happen at the edges? There are two usual choices: stop at the end, or wrap around to the start. Pick one. Either way, you must handle:

- **An empty list**, where the selection should be `None`. Nothing can be selected.
- **Going from empty to non-empty**, where you should select the first row.

### Common keys

| Key | Action |
|---|---|
| `↓` or `j` | Next row |
| `↑` or `k` | Previous row |
| `g` or `Home` | First row |
| `G` or `End` | Last row |
| `PageDown` / `PageUp` | Jump several rows |

Vim users expect `j`/`k`, and everyone else uses the arrows. Supporting both costs one line.

**Heads up:** the README's mockup has `k` for kill, and here it means "up". Decide now how to resolve this. You might use `x` or `d` for kill, or `K` with a capital. This is the sort of small design question you will meet often. Stage 15 assumes you have a key chosen.

## Your task

1. Add a `TableState` to `App`, and pick a row highlight style.
2. Move your key handling into `App::on_key`, along with a `should_quit` field.
3. Implement all the movement keys above.
4. Select the first row at startup, if there is one.
5. Write tests, with no terminal involved:
   - Moving down from the first row selects the second
   - Your chosen behavior at the last row, and at the first
   - All movement on an empty list leaves the selection as `None`, and does not panic
   - `q` sets `should_quit`

## Generic example

Moving a selection with wrap-around, on a list of books:

```rust
use ratatui::widgets::TableState;

struct Shelf {
    titles: Vec<String>,
    table: TableState,
}

impl Shelf {
    fn select_next(&mut self) {
        if self.titles.is_empty() {
            self.table.select(None);
            return;
        }
        let next = self.table.selected().map_or(0, |index| (index + 1) % self.titles.len());
        self.table.select(Some(next));
    }
}
```

Testing keys without a terminal:

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

struct Counter {
    value: u32,
}

impl Counter {
    fn on_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('+') {
            self.value += 1;
        }
    }
}

#[test]
fn plus_adds_one() {
    let mut counter = Counter { value: 0 };
    counter.on_key(KeyEvent::new(KeyCode::Char('+'), KeyModifiers::NONE));
    assert_eq!(counter.value, 1);
}
```

## Verify

Start enough servers to need scrolling:

```sh
for port in $(seq 8800 8840); do python3 -m http.server $port --bind 127.0.0.1 >/dev/null 2>&1 & done
cargo run
pkill -f 'http.server 88'
```

Check that:

- The selected row is obvious at a glance.
- Moving past the bottom of the screen scrolls the view.
- `G` then `g` jumps to each end.
- Your terminal stays responsive. Holding down an arrow key should feel smooth.
- With a shrunken window, the selection stays visible.

## Pitfalls

- **Subtracting from `0`.** `index - 1` when `index` is 0 panics in debug builds, and wraps to an enormous value in release. Use `saturating_sub`, or handle zero explicitly.
- **Modulo by zero.** `% len` panics on an empty list. Guard against it, as the example does.
- **`render_widget` instead of `render_stateful_widget`.** Your table draws fine, but with no highlight and no scrolling. It is a confusing one to debug.
- **Shift keys.** `G` arrives as `KeyCode::Char('G')`, and possibly with the `SHIFT` modifier too. Match on the character.

## Done when

- [ ] Arrows, `j`/`k`, `g`/`G`, and page keys all work
- [ ] The view scrolls when rows do not fit
- [ ] Empty lists are safe
- [ ] `on_key` is tested without a terminal
- [ ] You have chosen a kill key
- [ ] Committed
