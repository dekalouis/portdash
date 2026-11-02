# Stage 19 on a Mac: Survive the real world

## What differs on a Mac, and why

Most of the failure table carries over. These rows change, each from a test here:

| Scenario | On a Mac | Result here |
|---|---|---|
| Non-UTF-8 folder name | **APFS refuses to create one**: `mkdir: Illegal byte sequence` | Cannot be tested with a real folder. Keep your lossy conversion, since mounted disks may differ |
| Unicode forms | APFS finds a folder under either form of `café` | A lookup using the other form succeeded |
| Non-ASCII project name | Works as written, but use your temp folder | `プロジェクト` detected, and `lsof` printed it as plain UTF-8 |
| Newline in a folder name | `lsof` escapes it | Printed as the two characters `\n`, so the path no longer matches the real one. Rare, and `exists()` will catch it |
| Working folder deleted | No ` (deleted)` marker | `lsof` kept printing the old path. The row displayed without a crash, and project detection returned nothing |
| Process name with spaces and brackets | The `ln -s` step can fail | With pyenv, the link hits a shim: `pyenv: my (app): command not found`. Copy a real binary instead, as below |
| Port with many owners | Works | One row, `+1` |
| Pipe closed, no TTY | Work | Exit 0, and a clear message with exit 1 |
| Hundreds of sockets | Your shell may limit you first | 500 sockets in one process needed a raised limit. `launchctl limit maxfiles` printed a soft limit of 256. The scan listed 517 sockets in 0.13 s |
| `/proc/net/tcp6` missing | Does not apply | Replace with "`lsof` fails or is missing", and test that it reaches your error path |
| Run as root | The tool was not run as root | `sudo lsof` did show extra rows, including `launchd` as PID 1. Ask yourself what `HOME` and "mine only" should mean |

**Add one row:** a port owned by another user, which your tool must not call free (Stages 6 and 10).

**Docker differs.** The stage's Linux notes about `docker-proxy` and firewall rules do not describe a Mac, where Docker runs containers inside a VM. I did not run a container here, so check what your tool shows with the command in the original, and put the true result in your README.

**Temp paths need care in tests.** `std::env::temp_dir()` is under `/var/folders/...`, while `lsof` reports `/private/var/folders/...`. A test that compares paths fails unless you canonicalize.

## Your task on a Mac

Change task 3: document Docker, permissions, restart caveats, and your supported systems. Replace "Linux-only" with what is true now, and include the hidden ports limit.

In `tests/cli.rs`, canonicalize any temp folder that you compare with tool output.

## Generic example

A unique test folder, with its real path:

```rust
use std::{env, fs, io, path::PathBuf};

/// Makes a unique folder and returns its real path, with symlinks such as /var resolved.
fn recipe_folder(name: &str) -> io::Result<PathBuf> {
    let folder = env::temp_dir().join(format!("{name}-{}", std::process::id()));
    fs::create_dir_all(&folder)?;
    fs::canonicalize(folder)
}

fn main() {
    let folder = recipe_folder("soups").unwrap();
    assert!(folder.starts_with("/private"));
    fs::remove_dir(&folder).unwrap();
    println!("ok");
}
```

## Verify

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
for i in $(seq 1 20); do cargo test --quiet || break; done
```

The five integration tests from the task passed here, 20 runs out of 20, using a reference build and `python3` for the child server.

Odd process name, without the pyenv problem:

```sh
cp ./target/debug/your-helper-binary '/tmp/my (app)'
'/tmp/my (app)' 8791 &
SERVER=$!
cargo run -- list | grep 8791
kill $SERVER
```

Here I copied a tiny listener program, and the row showed `my (app)`.

## Pitfalls

- **Assuming your project folder is under `/Users/...` in tests.** Temp folders are not, and CI runners differ again.
- **A `kill` test that targets a hidden port.** Tests should only use ports they opened.
- **`mkdir $'/tmp/bad\xff'` in a setup script.** It fails here, so a script with `set -e` stops at it.

## Changes to "Done when"

Change "Every row in the failure table is handled or documented" to:

- [ ] Every row of the original table, as changed above, is handled or documented
- [ ] Hidden ports are in your README

## Tested on

macOS 27.2 (build 26B123), Apple M1 Max, 2026-11-02. `cargo run` results came from a throwaway reference build in a temp folder, not from a finished `portdash`, so your wording will differ.

Not tested: `sudo`, Docker, a case-sensitive volume, external disks with non-UTF-8 names, and an Intel Mac. `NO_COLOR` printed normally, but the reference uses no colors, so that says little.
