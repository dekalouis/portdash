# Build Your Own Dev Port Dashboard

A step-by-step guide to building `portdash` (working name) in Rust: a terminal dashboard that shows which dev servers are running, on which ports, from which project, and lets you stop or restart them.

You write the code. Each stage explains the concept, sets a task, and shows one generic example about something unrelated (books, sensors, weather). The example teaches the technique. Applying it to ports and processes is your job.

## What you will end up with

```
 portdash                                     filter: _        mine only
┌────────────────────────────────────────────────────────────────────┐
│ PORT   ADDRESS     PID     PROCESS   PROJECT        UPTIME         │
│ 3000   127.0.0.1   48121   node      shop-frontend  2h 14m         │
│ 5432   ::1         1203    postgres  -              3d 1h          │
│ 8000   0.0.0.0     50233   python    ml-api         12m            │
│ 8080   127.0.0.1   51877   portal    shop-backend   41s            │
├─ Details ──────────────────────────────────────────────────────────┤
│ cwd: /home/you/Developer/shop-frontend                             │
│ cmd: node node_modules/.bin/vite --port 3000                       │
└────────────────────────────────────────────────────────────────────┘
 ↑↓ move   k kill   r restart   / filter   s sort   q quit
```

## How to use this guide

1. Work through the stages in order. Each one builds on the last.
2. Write your code in `rust_projects/portdash/`, which you create in Stage 1. Keep this guide folder separate.
3. Finish every item under "Done when" before moving on.
4. Commit after each stage, so you can always get back to something that worked.
5. Tick off your progress in the roadmap below.

### Getting help from Claude Code

`prompt.md` tells Claude Code to act as a tutor: hints first, the answer only if you ask for it after trying. It does not load by itself, so start each session with:

```
@1-port-dashboard-guide/prompt.md I'm on stage 6 and my inode lookup returns nothing
```

### What each stage contains

| Section | What it gives you |
|---|---|
| Goal | The one thing that works by the end |
| Background | The concept, explained from scratch |
| Your task | What to build, without saying how to write it |
| Generic example | The technique, shown on an unrelated problem |
| Verify | Commands to prove it works |
| Pitfalls | The traps people actually fall into |
| Done when | The checklist for moving on |

## Roadmap

### Part 1: Reading what the kernel knows

- [ ] [Stage 1: Project setup](stages/01-project-setup.md) (Easy)
- [ ] [Stage 2: Read the raw socket table](stages/02-read-socket-table.md) (Easy)
- [ ] [Stage 3: Parse one line](stages/03-parse-one-line.md) (Easy)
- [ ] [Stage 4: Decode IPv4 and find listeners](stages/04-decode-ipv4.md) (Medium)
- [ ] [Stage 5: Add IPv6](stages/05-add-ipv6.md) (Medium)

### Part 2: Finding out who owns each port

- [ ] [Stage 6: Map sockets to processes](stages/06-sockets-to-pids.md) (Medium)
- [ ] [Stage 7: Read process details](stages/07-process-details.md) (Medium)
- [ ] [Stage 8: Detect the project](stages/08-detect-project.md) (Easy)

### Part 3: A useful CLI (your first shippable version)

- [ ] [Stage 9: The list command](stages/09-list-command.md) (Medium)
- [ ] [Stage 10: The kill command](stages/10-kill-command.md) (Medium)

### Part 4: The TUI

- [ ] [Stage 11: Your first screen](stages/11-first-screen.md) (Easy)
- [ ] [Stage 12: Draw the real table](stages/12-draw-table.md) (Medium)
- [ ] [Stage 13: Move through the list](stages/13-navigation.md) (Easy)
- [ ] [Stage 14: Live refresh](stages/14-live-refresh.md) (Hard)
- [ ] [Stage 15: Kill from the dashboard](stages/15-kill-from-tui.md) (Medium)
- [ ] [Stage 16: Filter and sort](stages/16-filter-sort.md) (Medium)
- [ ] [Stage 17: The details pane](stages/17-details-pane.md) (Easy)

### Part 5: Finishing

- [ ] [Stage 18: Restart](stages/18-restart.md) (Hard)
- [ ] [Stage 19: Survive the real world](stages/19-robustness.md) (Medium)
- [ ] [Stage 20: Ship it](stages/20-ship-it.md) (Medium)
- [ ] [Stage 21: Where to go next](stages/21-next-steps.md) (Optional)

## Working on a Mac

Stages with a Mac file link to it at the top. Each one holds only what differs. Stages 11, 12 and 13 work as written.

The biggest differences: macOS has no `/proc`, so the tool runs `lsof` and `ps` behind `scan()`. Without root, other users' ports do not appear at all. Restart cannot read the environment of Apple's own system binaries.

- [01-project-setup](stages/01-project-setup-mac-support.md): Rust needs Apple's linker, iCloud and `target/`, `cargo new` inside a repository
- [02-read-socket-table](stages/02-read-socket-table-mac-support.md): No `/proc`: run `lsof`, why not a crate, exit codes
- [03-parse-one-line](stages/03-parse-one-line-mac-support.md): Parse `lsof -F` records instead of columns
- [04-decode-ipv4](stages/04-decode-ipv4-mac-support.md): No hex or state column, and `*` addresses
- [05-add-ipv6](stages/05-add-ipv6-mac-support.md): One command for both families, and unusual address forms
- [06-sockets-to-pids](stages/06-sockets-to-pids-mac-support.md): Socket IDs, and why other users' ports are invisible
- [07-process-details](stages/07-process-details-mac-support.md): `ps` and `lsof` instead of per-process files
- [08-detect-project](stages/08-detect-project-mac-support.md): Homebrew's prefix looks like a project
- [09-list-command](stages/09-list-command-mac-support.md): The `#[cfg]` split, and an honest footer
- [10-kill-command](stages/10-kill-command-mac-support.md): Zombies through `ps`, and ports held by someone you cannot see
- [14-live-refresh](stages/14-live-refresh-mac-support.md): The real cost of a scan, and why `top` hides it
- [15-kill-from-tui](stages/15-kill-from-tui-mac-support.md): A Verify step that cannot happen
- [16-filter-sort](stages/16-filter-sort-mac-support.md): "Mine only" changes nothing without root
- [17-details-pane](stages/17-details-pane-mac-support.md): Memory and uptime from `ps`
- [18-restart](stages/18-restart-mac-support.md): `KERN_PROCARGS2`, and processes whose environment cannot be read
- [19-robustness](stages/19-robustness-mac-support.md): Changed rows in the failure table
- [20-ship-it](stages/20-ship-it-mac-support.md): `otool`, two architectures, Gatekeeper and notarization
- [21-next-steps](stages/21-next-steps-mac-support.md): Which ideas depend on Linux

## What you should know before starting

- **Rust:** the first 10 chapters of [The Rust Book](https://doc.rust-lang.org/book/) are enough to begin: ownership, structs, enums, `Result`. If you have not read them, start there. Later stages pick up traits, threads and channels, and each one says what it needs.
- **Linux:** being able to use a terminal is enough. Each stage explains the system concepts it uses.

## Crates used

Added only at the stage that needs them, and checked against these versions on 2026-11-01:

| Crate | Version | First used | For |
|---|---|---|---|
| `clap` | 4.6 | Stage 9 | Command line arguments |
| `serde`, `serde_json` | 1 | Stage 9 | `--json` output |
| `nix` | 0.31 | Stage 7 | Signals and system values |
| `ratatui` | 0.30 | Stage 11 | Drawing the TUI |
| `crossterm` | 0.29 | Stage 11 | Keyboard input |

If you use a newer major version and an example stops compiling, check that crate's docs or ask Claude Code.

## Two things to know up front

**This is Linux-only at first.** Everything is built on `/proc`, which macOS and Windows do not have. That is the right place to start, but it limits your audience. Stage 21 covers adding macOS, and earlier stages keep the code organized so that it stays possible.

**You have competition, so check it.** At least one similar tool exists: [`lsport`](https://crates.io/crates/lsport), "a TUI for managing local and remote ports via SSH". There are probably others, in other languages too. Before Stage 20, try them and be clear about what makes yours different. The strongest angle here is project awareness: showing "shop-frontend" instead of just "node" on port 3000.
