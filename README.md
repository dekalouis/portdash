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
- [x] Stage 7: read process details (Linux reads `/proc/<pid>/stat`, `cmdline` and `cwd` and shows name, command and working directory under each listener. macOS not done yet)
- [x] Stage 8: detect the project (Linux walks up from the working directory to the nearest folder with a marker such as `.git`, `package.json` or `Cargo.toml`, stops before `$HOME`, and shows the folder name as the last column. macOS not done yet)
- [x] Stage 8a: readable errors (`ParseError` implements `Display` and `Error`, with a separate variant for bad addresses, and warnings print with `{}` instead of `{:?}`)
- [x] Stage 9: the list command (`portdash list` prints an aligned table with one row per socket, `--json` prints the full data, a footer counts ports with unknown owners, and a closed pipe exits quietly. macOS not done yet)
- [x] Stage 10: the kill command (`portdash kill <port>` asks first, sends SIGTERM to the main process, waits up to 5 seconds, then sends SIGKILL to any owner still alive. It refuses PID 0, 1 and its own, and re-checks the start time so a reused PID is never hit. macOS not done yet)

### Progress per device

| Device | Last finished stage |
|---|---|
| macOS | Stage 6 |
| Linux | Stage 10 |

## Run

```sh
cargo run -- list
cargo run -- list --json
cargo run -- kill 8765
cargo run -- kill 8765 --yes --force
cargo run -- --help
```

On macOS this runs `/usr/sbin/lsof`, so no extra install is needed. It prints each listening socket as `address:port: [pids]`, sorted by port, with IPv6 in brackets, like `[::1]:6379: [1593]`. Without `sudo`, only your own user's ports are visible, so other users' ports are missing from the list instead of showing an unknown owner. On my machine that hides Postgres on 5432 and `launchd` on 8021. (I don't feel it's worth the trouble writing unsafe code for a dev server dashboard :P)

On Linux it reads `/proc/net/tcp` and `/proc/net/tcp6` (a missing `tcp6` file is fine), finds each socket's owners by scanning `/proc/<pid>/fd`, and prints one row per listening socket, sorted by port:

```
PORT   ADDRESS     PID         PROCESS  PROJECT
5432   ::1         -           -        -
8765   127.0.0.1   1948479     python3  demo-app
8767   127.0.0.1   2460215 +3  python3  portdash
```

When several processes share a socket (a forked server), the row shows the first PID and `+N` for the others. The project is found by walking up from the owner's working directory to the nearest folder with a marker such as `.git`, `package.json` or `Cargo.toml`. The walk stops before your home folder, so home itself is never a project. Known limit: home is read from `$HOME`, which is the home of the user running portdash and not of each process owner. Under `sudo` that is `/root`, so `/home/<user>` is not protected and a marker directly in it would label processes running there.

`--json` prints every socket with its address, port, project and owners. Each owner has its PID, name, full command and working directory, with `null` for anything unknown. The full command is left out of the table because it can run to thousands of characters.

Without `sudo` you can only read your own processes, so other users' sockets show `-` and a footer on stderr says how many, like `11 ports have unknown owners (try sudo)`. Results go to stdout and messages to stderr, so `portdash list --json | jq` stays clean, and `portdash list | head -1` exits quietly instead of panicking on the closed pipe.

`kill <port>` lists every process holding the port and asks `stop it? [y/N]` (skip it with `--yes`). It sends SIGTERM to the main process, meaning an owner whose parent is not also an owner, so a server with workers can shut them down itself. If anything is still running after 5 seconds, every remaining owner gets SIGKILL. `--force` sends SIGKILL right away. Before each signal it refuses PID 0, 1 and its own PID, and re-reads the process start time, so a PID that was reused by another process is skipped. A zombie (exited, waiting for its parent) counts as stopped. It exits with 1 if nothing is listening, the owner belongs to another user, or something could not be stopped. Known limit: a PID can still be reused in the tiny gap between the check and the signal, which a pidfd would close.

Run the tests with:

```sh
cargo test
```

## Debug output

Errors read as sentences that name the input, like `skipping <line>: uid is not a valid number: "abc"`. On macOS the PID list after each address (`[1234, 1235]`) is still Rust's `Debug` format and is temporary until its side of Stage 9 replaces it with a table.
