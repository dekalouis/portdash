# portdash

A terminal dashboard, written in Rust, that shows which dev servers are listening on which ports, and which project each one belongs to.

Work in progress. I'm building it stage by stage as a way to learn Rust and systems programming.

## Status

- [x] Stage 1: project setup
- [x] Stage 2: read the TCP sockets (macOS through `lsof`, Linux through `/proc/net/tcp`)
- [x] Stage 3: parse one socket into a struct (Linux from `/proc/net/tcp`, macOS from `lsof`)
- [x] Stage 4: decode IPv4 addresses and list only LISTEN sockets, sorted by port (Linux). Matches `ss -4ltn`
- [x] Stage 5: add IPv6, merged from `/proc/net/tcp6` (Linux). Matches `ss -ltn`

### Progress per device

| Device | Last finished stage |
|---|---|
| macOS | Stage 3 |
| Linux | Stage 5 |

## Run

```sh
cargo run
```

On macOS this runs `/usr/sbin/lsof`, so no extra install is needed. Without `sudo`, only your own user's ports are visible.

On Linux it reads `/proc/net/tcp` and `/proc/net/tcp6` (a missing `tcp6` file is fine) and prints the listening sockets as `address:port`, sorted by port. IPv6 addresses are written in brackets, like `[::1]:5432`. Run the tests with:

```sh
cargo test
```

## Debug output

Some output is temporary for now. The macOS output still prints with `{:?}`, Rust's `Debug` format, which is meant for me. Parse errors have no readable message yet. Stage 8a of the guide replaces both with proper `Display` messages.
