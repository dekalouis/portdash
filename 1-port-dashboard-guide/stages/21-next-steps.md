# Stage 21: Where to go next

On a Mac? Also read [this stage's Mac notes](21-next-steps-mac-support.md).

**Goal:** there is none in particular. These are optional directions. Let your real users' issues guide which you pick, because what people ask for beats what sounds cool.

## macOS support

This is the most valuable extension, since a large share of developers use Macs and they feel the same pain. It is also the biggest change, because macOS has no `/proc`.

**Concept.** Hide the platform behind a small interface. Your TUI should not know how the data is gathered. If you kept `scan()` as the single entry point in Stage 9, the rest of your code will not need to change.

```rust
struct Record {
    title: String,
}

#[cfg(target_os = "linux")]
fn load_records() -> Vec<Record> {
    vec![Record { title: "from the Linux path".into() }]
}

#[cfg(not(target_os = "linux"))]
fn load_records() -> Vec<Record> {
    vec![Record { title: "from the other path".into() }]
}
```

**Options on macOS:** run and parse `lsof -nP -iTCP -sTCP:LISTEN`, which is simple but slower, or call `libproc` through bindings, which is fast but more work. Starting with `lsof` is reasonable. You will need a Mac or macOS CI runner to test.

## `portdash run`

Stage 18 described this alternative. Instead of reconstructing how a server started, your tool starts it:

```sh
portdash run -- npm run dev
```

Your tool then owns the recipe, the process tree and its logs. That gets you reliable restarts and a live log view in the dashboard. It starts turning your tool into a light process manager, so look at `overmind` and `mprocs` first to understand that territory, and to learn what not to rebuild.

## Named local domains and HTTPS

This is idea #4 in [`IDEAS.md`](../../IDEAS.md). Your dashboard knows which project runs on which port, so it can offer `https://shop-frontend.localhost` automatically. A reverse proxy routes by hostname to the right port, using certificates from a local authority.

It is a big project, and a natural follow-up because you already have the hardest data: the mapping of project to port. Portless from Vercel Labs and Caddy already cover much of this, so read the full guide in [`4-local-domains-guide`](../../4-local-domains-guide/README.md), which starts with that comparison.

**What you would learn:** async Rust with `tokio`, HTTP proxying, TLS with `rustls`, and how certificate trust works.

## Docker awareness

From Stage 19: containers publish ports your tool may misreport. Reading `docker ps --format json` could fill the gap, showing container names as the "project". Developers use Docker constantly, so this would likely be requested soon.

## Smaller features

- **Port conflict helper:** `portdash why 3000` explains in a sentence what holds a port, and how to free it.
- **Free port finder:** `portdash free` prints an unused port, handy in scripts.
- **Exposure warning:** highlight servers on `0.0.0.0`, which may be reachable from your network. You collected this in Stage 4.
- **Open in browser:** press a key to open `http://localhost:<port>`. `xdg-open` handles Linux.
- **Log tailing:** for servers you restarted, you already know their log file.
- **Config file:** let users set refresh rate, key bindings and hidden ports. Wait for someone to ask.
- **UDP sockets:** `/proc/net/udp` has a similar format.
- **Themes:** only if people ask.
- **Mouse support:** clicking rows. `crossterm` reports mouse events.

## Faster, tighter code

- **`pidfd`:** close the race window left over from Stage 10 by holding a handle to the process itself rather than a reusable number. This is a good lesson in modern Linux APIs.
- **Netlink `sock_diag`:** this is what `ss` uses instead of parsing `/proc/net/tcp`, and it is much faster on machines with many sockets. Only worth it if your measurements say so. You probably will not need it.

## Keep it healthy

- Respond to issues, even if only to say "not planned".
- Run `cargo update` and `cargo audit` occasionally.
- Write a "how this works" blog post. You now understand `/proc`, signals and TUI architecture. That post is also strong evidence of your skills, and it brings new users.

## What you learned

| Area | Skills |
|---|---|
| Rust | Ownership, enums, error handling, traits, threads, channels, modules, testing |
| Linux | `/proc`, sockets, file descriptors, signals, process groups, permissions, zombies |
| Software design | Separating state from view, testable pure functions, safe destructive operations |
| Shipping | Packaging, static builds, CI, documentation, working with users |

Nice work.
