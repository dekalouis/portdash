# Project structure plan

The guide names only a few files. This plan fills in the rest, so you never have to guess where code goes.

Each file is marked **Guide** (a stage names it) or **Plan** (my proposal, which you can change).

## The rules

1. **One module per job.** If you need "and" to describe a file, it is two files.
2. **Getting data is separate from parsing it.** Parsers take a `&str` and do no I/O, so tests can use plain strings.
3. **`scan()` is the only way to get data** (from Stage 9). Everything above it is shared, and everything below it is OS-specific.
4. **`main.rs` only wires things together.** From Stage 9 it holds argument handling and calls into other modules.
5. **Tests live at the bottom of the file they test**, in `#[cfg(test)] mod tests`. Only `tests/cli.rs` is separate.

## Where the project ends up

```
portdash/
├── Cargo.toml
├── README.md
├── CHANGELOG.md                  Stage 20, generated
├── LICENSE-*                     Stage 20
├── .github/workflows/
│   ├── ci.yml                    Stage 20
│   └── release.yml               Stage 20
├── tests/
│   └── cli.rs                    Stage 19
└── src/
    ├── main.rs                   entry point and argument handling
    ├── sockets.rs                raw socket data from the OS
    ├── parse.rs                  raw text to typed values
    ├── process.rs                socket inodes to processes, process details
    ├── project.rs                process folder to project name
    ├── listener.rs               Listener type and scan()
    ├── output.rs                 table and JSON for `list`
    ├── kill.rs                   stopping a process safely
    ├── restart.rs                RestartPlan and the restart sequence
    └── tui/
        ├── mod.rs                run(), the event loop, the scan thread
        ├── app.rs                App state and on_key (state and update)
        └── view.rs               draw functions (view)
```

## Every file, and when it appears

### Part 1: Reading sockets (Stages 1 to 5)

| File | Stage | Source | Job |
|---|---|---|---|
| `src/main.rs` | 1 | Guide | Entry point |
| `src/sockets.rs` | 2 | Guide | Read raw text from the OS. No parsing |
| `src/parse.rs` | 3 | Plan | Turn raw text into `TcpEntry` and friends. No I/O |

Stages 4 and 5 add no files:

- **Stage 4:** the hex to `Ipv4Addr` function, the whole-file parser and the LISTEN filter go in `parse.rs`. Sorting and printing stay in `main.rs` for now.
- **Stage 5:** IPv6 decoding goes in `parse.rs`. Reading `/proc/net/tcp6` goes in `sockets.rs`.

### Part 2: Who owns each port (Stages 6 to 8)

| File | Stage | Source | Job |
|---|---|---|---|
| `src/process.rs` | 6 | Guide | Map inodes to PIDs. Stage 7 adds process details here |
| `src/project.rs` | 8 | Guide | Walk up from a folder to find the project |

### Part 3: A useful CLI (Stages 8a to 10)

| File | Stage | Source | Job |
|---|---|---|---|
| `src/listener.rs` | 9 | Plan | The `Listener` type and `scan()`, the one entry point for data |
| `src/output.rs` | 9 | Plan | Aligned table and JSON. Display logic leaves `main.rs` |
| `src/kill.rs` | 10 | Plan | Signal logic, shared with the TUI in Stage 15 |

Stage 8a adds no files. Each error enum gets its `Display` in the file where it is defined.

### Part 4: The TUI (Stages 11 to 17)

| File | Stage | Source | Job |
|---|---|---|---|
| `src/tui.rs` | 11 | Guide | `run()` and the first screen |
| `src/tui/app.rs` | 12 | Plan | `App` and `on_key`. Plain state, no drawing |
| `src/tui/view.rs` | 12 | Plan | `draw`. Reads state, makes no system calls |

Stage 12 separates state, update and view, which is the moment to turn `tui.rs` into a `tui/` folder (`tui.rs` becomes `tui/mod.rs`).

Stages 13 to 17 add features to these files:

- **Stage 13 (navigation):** `app.rs`.
- **Stage 14 (live refresh):** the scan thread and channel in `tui/mod.rs`, and keeping the selection in `app.rs`.
- **Stage 15 (kill):** the popup in `app.rs` and `view.rs`, calling `kill.rs`.
- **Stage 16 (filter and sort):** the visible-rows logic in `app.rs`.
- **Stage 17 (details pane):** new fields in `process.rs` and `listener.rs`, drawn in `view.rs`.

### Part 5: Finishing (Stages 18 to 21)

| File | Stage | Source | Job |
|---|---|---|---|
| `src/restart.rs` | 18 | Plan | `RestartPlan` and the sequence, shared by the TUI and the CLI |
| `tests/cli.rs` | 19 | Guide | Integration tests against the built binary |
| `.github/workflows/ci.yml` | 20 | Guide | Format, clippy and tests on each push |
| `.github/workflows/release.yml` | 20 | Plan | Build binaries on a tag. The guide asks for it but does not name the file |
| `LICENSE-*`, `CHANGELOG.md` | 20 | Guide | Release files |

## OS-specific code

`sockets.rs`, `parse.rs` and `process.rs` have a Linux and a macOS version of their code, behind `#[cfg(target_os = "...")]`. `scan()` hides that choice from everything above it.

`parse.rs` already holds both parsers and is about 400 lines. If that gets hard to read, split it at Stage 9, when the guide asks for a tidy layout:

```
src/parse/
├── mod.rs      ParseError, and the right parser for this OS
├── linux.rs    TcpEntry, parse_socket
└── macos.rs    LsofEntry, parse_lsof
```

This split is optional. `sockets.rs` and `process.rs` can split the same way if they grow.

## Where does this new code go?

1. Does it read from the OS? `sockets.rs` or `process.rs`.
2. Does it turn text into a type? `parse.rs`.
3. Does it print for the CLI? `output.rs`.
4. Does it change TUI state? `tui/app.rs`.
5. Does it draw? `tui/view.rs`.
6. Does it do something to a process? `kill.rs` or `restart.rs`.

If none of these fit, it may be a new job, so give it a new file and add it here.

Remember that each new file needs a `mod name;` line in its parent, or Rust will not compile it. For `src/` files the parent is `main.rs`. For `src/tui/app.rs` it is `src/tui/mod.rs`.
