# portdash

A terminal dashboard, written in Rust, that shows which dev servers are listening on which ports, and which project each one belongs to.

Work in progress. I'm building it stage by stage as a way to learn Rust and systems programming.

## Status

- [x] Stage 1: project setup
- [x] Stage 2: read the TCP sockets (macOS through `lsof`, Linux through `/proc/net/tcp`)
- [x] Stage 3: parse one `/proc/net/tcp` line into a struct (Linux)
- [ ] Stage 4: decode IPv4 addresses and list only LISTEN sockets, sorted by port (Linux). Code works and matches `ss -4ltn`, tests still to write

## Run

```sh
cargo run
```

On macOS this runs `/usr/sbin/lsof`, so no extra install is needed. Without `sudo`, only your own user's ports are visible.

On Linux it reads `/proc/net/tcp` and prints the IPv4 listening sockets as `address:port`, sorted by port. Run the tests with:

```sh
cargo test
```

## Debug output

Some output is temporary for now. The macOS output still prints with `{:?}`, Rust's `Debug` format, which is meant for me. Parse errors have no readable message yet. Stage 8a of the guide replaces both with proper `Display` messages.
