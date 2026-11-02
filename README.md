# portdash

A terminal dashboard, written in Rust, that shows which dev servers are listening on which ports, and which project each one belongs to.

Work in progress. I'm building it stage by stage as a way to learn Rust and systems programming.

## Status

- [x] Stage 1: project setup
- [x] Stage 2: read the listening TCP sockets (macOS, through `lsof`)

## Run

```sh
cargo run
```

On macOS this runs `/usr/sbin/lsof`, so no extra install is needed. Without `sudo`, only your own user's ports are visible.

## Debug output

The output is temporary for now. It prints with `{:?}`, Rust's `Debug` format, which is meant for me. Stage 8a of the guide replaces it with proper `Display` messages.
