# Stage 19: Survive the real world

On a Mac? Also read [this stage's Mac notes](19-robustness-mac-support.md).

**Goal:** the tool handles odd machines and odd situations gracefully, and integration tests prove the entire thing works against real sockets.

## Background

### Your machine is not everyone's machine

So far you have tested on one Arch Linux laptop. Other people have containers, unusual setups, and strange process names. Most of the bugs reported against a CLI tool come from this. Fixing them now is what separates a project from a tool people trust.

### Unit tests versus integration tests

Your unit tests check small pure functions. They cannot catch a mistake in how those pieces connect. **Integration tests** run the real binary, as a user would.

Files in `tests/` are compiled as separate programs. Cargo gives them the path to your built binary through `env!("CARGO_BIN_EXE_<name>")`.

The trick for reliable socket tests is **port 0**: binding to port 0 asks the OS for any free port. Your test can then read which one it got. Tests never fight over a hardcoded port, even when running in parallel.

### Containers and network namespaces

`/proc/net/tcp` only shows sockets in **your network namespace**. So:

- **Docker published ports** might appear as `docker-proxy` owned by root, or might not appear at all, depending on the Docker configuration. They are forwarded by firewall rules. A user may see their app work on a port your tool says is free.
- **Inside a container**, you see that container's sockets only.

You probably cannot solve this now. But `docker ps` being available means you could add it in the future. For now, document the limit clearly in your README. Honest documentation beats a mysterious wrong answer.

### Going through failure modes

Go through this table and test each one. Decide what the right behavior is, and whether yours already does it.

| Scenario | How to test it | What should happen |
|---|---|---|
| Nothing listening | Filter to something absent, or use a clean container | Friendly message |
| Terminal very small | Resize down to 10x3 | No panic |
| Very long project or command | A long folder name | Cut off or wrapped neatly |
| Non-UTF-8 folder name | `mkdir $'/tmp/bad\xff'` and run a server there | Shows with a replacement character |
| Non-ASCII project name | `mkdir /tmp/プロジェクト`, with a marker | Aligned columns |
| Working folder was deleted | Start a server in a folder, then delete it | Shows without crashing |
| Process name with spaces and brackets | `ln -s "$(which python3)" '/tmp/my (app)'`, then run `'/tmp/my (app)' -m http.server 8791` | Parses correctly |
| Port with many owners | The forking server from Stage 6 | All shown |
| Run as root | `sudo ./target/debug/portdash` | All owners visible, nothing breaks |
| Pipe closed | `portdash list \| head -1` | Exits with no panic |
| No TTY | `portdash \| cat` | A clear message |
| Hundreds of sockets | The `seq` loop from Stage 13, with more ports | Stays fast |
| `/proc/net/tcp6` missing | Hard to test for real. Cover the function with a unit test | Works with IPv4 only |
| `NO_COLOR` is set | `NO_COLOR=1 portdash` | Readable, no colors |

Non-UTF-8 and unusual names are not theoretical: file names on Linux are bytes, and users will eventually run into these.

### Clippy and CI

Before you publish, tighten quality checks. Make sure these pass with no warnings:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Consider adding these to Cargo.toml's `[lints]` so they apply to everyone. Stage 20 runs them in CI on each push.

## Your task

1. Go through the table above. For every failure, write a test where you can, and fix the bug.
2. Create `tests/cli.rs` with integration tests:
   - `list` shows a port your test is listening on
   - `list --json` includes that port, and parses as valid JSON
   - `kill` on an unused port exits non-zero
   - `kill` really stops a server you spawned in the test, and frees the port
   - `list` shows your project name when the server's folder has a marker
3. Document known limits in your README: Docker, permissions, Linux-only, restart caveats.
4. Run your tests repeatedly to find flakiness. They must pass every time:

   ```sh
   for i in $(seq 1 20); do cargo test --quiet || break; done
   ```

## Generic example

An integration test that binds a real port, then runs the binary:

```rust
// tests/cli.rs
use std::{net::TcpListener, process::Command};

#[test]
fn echoes_the_port_it_was_given() {
    // Port 0 means the OS picks a free port, so tests never collide.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port().to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_echo_tool"))
        .arg(&port)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains(&port));
}
```

Your listener stays open for as long as the variable lives, which is what you want. `drop(listener)` closes it.

To spawn a separate process that holds a port, which your `kill` test needs, use `Command::spawn` with a small server, and read back the `Child`'s PID. Always clean it up, even when the test fails. A struct with a `Drop` implementation that kills the child handles that for you:

```rust
use std::process::Child;

struct CleanupOnDrop(Child);

impl Drop for CleanupOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
```

## Verify

Work through the failure table by hand, and then:

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

If you have Docker, try it on a container: `docker run --rm -p 8090:80 nginx`. What does your tool show? Does your README tell the truth about it?

## Pitfalls

- **Sleeping for a fixed time in tests.** A `sleep(1)` is flaky and slow. Poll in a loop until the condition holds or a generous deadline passes.
- **Leftover server processes** from failed tests. Use the `Drop` pattern above.
- **Depending on Python in tests.** It works here, but makes contributing harder. Your test can use a tiny helper binary, or `TcpListener` itself when a separate process is unnecessary.
- **A `kill` test that signals your test process.** A listener bound inside the test belongs to the test process, so `kill` on that port targets your test runner itself. Your `kill` tests must use a child process.
- **Assuming your own machine's ports.** Tests must never rely on what else happens to be running.

## Done when

- [ ] Every row in the failure table is handled or documented
- [ ] Integration tests cover `list`, `--json`, `kill` and project detection
- [ ] 20 test runs in a row pass
- [ ] Clippy with `-D warnings` and `fmt --check` are clean
- [ ] Known limits are in the README
- [ ] Committed
