# Stage 14: Live refresh

On a Mac? Also read [this stage's Mac notes](14-live-refresh-mac-support.md).

**Goal:** the table updates by itself every couple of seconds, the UI never freezes, and your selection stays on the same server while rows change around it.

This is the hardest stage so far. It is your first concurrency, so take your time.

## Background

### The problem with a blocking loop

Your loop waits on `event::read()`, which blocks until a key arrives. If nobody presses a key, the screen never updates. You could scan on the same thread between keys, but then each scan freezes the UI. On a busy machine that means visible lag every couple of seconds.

### Threads and channels

The clean design is several threads with one job each, all sending messages to one place:

```
 input thread  --(key pressed)-------+
                                     v
                                  channel --> main thread: updates state, draws
                                     ^
 scan thread   --(new listeners)-----+
```

- **Main thread:** owns `App`, and does all the drawing. It is the only thing changing state.
- **Input thread:** waits for keys and forwards them.
- **Scan thread:** runs `scan()` in a loop, and sends each result.

A **channel** (`std::sync::mpsc`) is a one-way queue between threads. Senders can be cloned, so many threads can send, and one receiver gets everything in order.

Rust helps a lot here. It will not compile code that shares mutable data unsafely. With channels, each value moves from one thread to another, so there is no shared state to protect at all, and no locks.

### One event type

Define an enum with one variant for each kind of message: a key, a resize, a new scan result, and so on. The main loop becomes wonderfully simple: receive a message, update state, draw.

Blocking on `rx.recv()` also means your app uses no CPU while idle. It wakes only when there is something to do.

### Clean shutdown

When `main` exits, other threads are stopped with the process. That is fine here. But a thread's `tx.send(...)` returns an error once the receiver is gone, and that is your signal for a thread to stop if it is ever still running. Handle it with a `break`, as shown below, instead of `unwrap`.

### Keeping the selection in place

This is the subtle part, and it separates a good dashboard from an annoying one.

`TableState` stores a row *index*. After a refresh, a new server may sort in above your selection. Row 3 is now a different server, and your highlight has silently moved to something else. Soon you will add a kill key, so that would be a dangerous bug.

So the selection has to follow **identity**, not position:

1. Before replacing the list, remember what is selected, using its stable key.
2. Replace the list.
3. Find that key in the new list, and select its new index.
4. If it is gone, pick something sensible, such as staying at the same index, limited to the new length.

Think about what makes a good key. Port alone? Port and address? Remember from Stage 5 that one port may appear twice.

### Measure the cost

Your scan runs every few seconds, forever, in the background. How much CPU does a dashboard get to use? Use your measurement from Stage 6 and check this for real. A refresh every 2 seconds is a good start. Consider skipping redraws when nothing has changed.

## Your task

1. Define an `AppEvent` enum.
2. Start an input thread and a scan thread that send into a channel.
3. Rewrite the main loop around `rx.recv()`.
4. Keep the selection on the same listener across refreshes, and test this. It is pure logic, so no threads are needed:
   - The selected item moves to a new index when a row is inserted above it
   - The selection falls back gracefully when the selected row disappears
   - The list becoming empty gives `None`
5. Handle scan failure by keeping the last good data and showing a small error, instead of crashing.
6. Add `Ctrl+R` or `R` for "refresh now". This is a second, smaller design problem, since your scan thread is asleep: think about how to wake it. `recv_timeout` on a second channel is one way.

## Generic example

Two workers sending into one channel:

```rust
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::{sync::mpsc, thread, time::Duration};

enum Message {
    Key(KeyCode),
    Weather(String),
}

fn slow_weather_lookup() -> String {
    thread::sleep(Duration::from_millis(500));
    "sunny".to_string()
}

fn start_workers() -> mpsc::Receiver<Message> {
    let (tx, rx) = mpsc::channel();

    let weather_tx = tx.clone();
    thread::spawn(move || loop {
        let report = slow_weather_lookup();
        // The receiver is gone, so the app is exiting. Stop quietly.
        if weather_tx.send(Message::Weather(report)).is_err() {
            break;
        }
        thread::sleep(Duration::from_secs(2));
    });

    thread::spawn(move || loop {
        match event::read() {
            Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                if tx.send(Message::Key(key.code)).is_err() {
                    break;
                }
            }
            Ok(_) => {}
            Err(_) => break,
        }
    });

    rx
}

fn main_loop(rx: mpsc::Receiver<Message>) {
    let mut report = String::from("waiting...");
    while let Ok(message) = rx.recv() {
        match message {
            Message::Weather(new_report) => report = new_report,
            Message::Key(KeyCode::Char('q')) => break,
            Message::Key(_) => {}
        }
        println!("{report}"); // A real TUI would redraw here instead.
    }
}
```

Your input thread also needs to forward `Event::Resize`, or the screen will not redraw when the window is resized. This example only shows keys.

## Verify

Run the app in one terminal, and in another:

```sh
python3 -m http.server 8765 --bind 127.0.0.1 &
sleep 5
kill %1
```

- The server appears, then disappears, with no keypress.
- Select a high port, start a server on a lower one, and the highlight must stay on the same server even though its row moves.
- Holding an arrow key stays smooth during a refresh.
- Quitting is instant, without waiting out a scan or a sleep.
- CPU stays low when idle. Check with `top -p $(pgrep portdash)`. Use a release build for this.

Test freeze resistance: put a temporary `thread::sleep(Duration::from_secs(3))` into `scan()`. Navigating must stay perfectly smooth. Then remove it.

## Pitfalls

- **Scanning on the main thread.** The sleep test above exposes it.
- **Spinning.** A `loop` with `try_recv` and no blocking uses 100% of a CPU core. Use `recv`.
- **Using the index as identity.** Your test should catch this.
- **Unstable sort.** If two rows compare equal and the order changes between scans, your rows flicker. Sort on enough fields to make the order fully determined.
- **`.unwrap()` on `send`.** It panics during shutdown, on a thread that was about to end anyway.
- **Drawing from a worker thread.** Only the main thread touches the terminal.

## Done when

- [ ] Rows update with no keypress
- [ ] The UI stays smooth with a slowed-down scan
- [ ] The selection follows the same listener, with tests
- [ ] Idle CPU use is low, with a number you measured
- [ ] Scan errors do not crash the app
- [ ] Committed
