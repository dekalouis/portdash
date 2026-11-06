# portdash

A terminal dashboard, written in Rust, that shows which dev servers are listening on which ports, and which project each one belongs to.

Work in progress. I'm building it stage by stage as a way to learn Rust and systems programming.

## Status

- [x] Stage 1: project setup
- [x] Stage 2: read the TCP sockets (macOS through `lsof`, Linux through `/proc/net/tcp`)
- [x] Stage 3: parse one socket into a struct (Linux from `/proc/net/tcp`, macOS from `lsof`)
- [x] Stage 4: decode IPv4 addresses and list only LISTEN sockets, sorted by port (Linux matches `ss -4ltn`, macOS matches `lsof -nP -iTCP -sTCP:LISTEN -a -i4`)
- [x] Stage 5: add IPv6 (Linux merges `/proc/net/tcp6` and matches `ss -ltn`, macOS gets it from the same `lsof` call and matches `lsof -nP -iTCP -sTCP:LISTEN`)
- [x] Stage 6: map sockets to PIDs (Linux scans `/proc/<pid>/fd` and matches `ss -ltnp`, macOS groups the `lsof` records by socket). A forked server shows both PIDs on both

### Progress per device

| Device | Last finished stage |
|---|---|
| macOS | Stage 6 |
| Linux | Stage 6 |

## Run

```sh
cargo run
```

On macOS this runs `/usr/sbin/lsof`, so no extra install is needed. It prints each listening socket as `address:port: [pids]`, sorted by port, with IPv6 in brackets, like `[::1]:6379: [1593]`. Without `sudo`, only your own user's ports are visible, so other users' ports are missing from the list instead of showing an unknown owner. On my machine that hides Postgres on 5432 and `launchd` on 8021.

On Linux it reads `/proc/net/tcp` and `/proc/net/tcp6` (a missing `tcp6` file is fine) and prints each listening socket as `address:port: [pids]`, sorted by port. IPv6 addresses are written in brackets, like `[::1]:5432`. The owner PIDs come from scanning `/proc/<pid>/fd`. Without `sudo` you can only read your own processes, so other users' ports show `-` instead of a PID. Run the tests with:

```sh
cargo test
```

## Debug output

Some output is temporary for now. On macOS, the warning for a skipped socket still prints with `{:?}`, Rust's `Debug` format, which is meant for me. The Linux PID list (`[1234, 1235]`) is also temporary. Parse errors have no readable message yet. Stage 8a of the guide replaces both with proper `Display` messages.
