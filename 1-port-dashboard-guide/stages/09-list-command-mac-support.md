# Stage 9 on a Mac: The list command

## What differs on a Mac, and why

**The platform split gets its final shape here.** `scan()` remains the single entry point. Everything above it is shared, including `Listener`, the table, JSON and clap. Everything below it depends on the operating system.

**The hidden-owner footer cannot be written as stated.** Task 6 says `2 ports have unknown owners (try sudo)`. Without root, `lsof` gives you no such rows, so you cannot count them (Stage 6). Your tool can only say that the limit exists.

**A person's first thought will be wrong.** On Linux, a missing port really is free or at least visible as unknown. On a Mac, an absent row may be another user's server. Say so somewhere visible.

## Your task on a Mac

Replace task 6:

6. If you are not root, add a footer such as `note: other users' ports are not shown (try sudo)`. Print it to stderr, or leave it out of `--json`, so that scripts are not affected. Do not invent a count.

Also, for task 1, put the platform code behind `scan()`:

- `#[cfg(target_os = "linux")]` for the `/proc` code
- `#[cfg(target_os = "macos")]` for the `lsof` and `ps` code
- One shared `Listener` type, with both returning it

## Generic example

One public function, with the platform choice hidden inside:

```rust
struct Record {
    title: String,
}

#[cfg(target_os = "macos")]
mod shelf {
    pub fn titles() -> Vec<String> {
        vec!["read the Mac way".to_string()]
    }
}

#[cfg(target_os = "linux")]
mod shelf {
    pub fn titles() -> Vec<String> {
        vec!["read the Linux way".to_string()]
    }
}

/// The only function the rest of the program calls.
fn load_records() -> Vec<Record> {
    shelf::titles()
        .into_iter()
        .map(|title| Record { title })
        .collect()
}

fn main() {
    println!("{}", load_records()[0].title);
}
```

## Verify

The original commands all work, with `python3` and `head` as they are:

```sh
python3 -m http.server 8765 --bind 127.0.0.1 &
SERVER=$!
cargo run -- list
cargo run -- list --json | python3 -m json.tool
cargo run -- list | head -1
cargo run -- --help
kill $SERVER
```

Results here:

```
PORT   ADDRESS      PID      PROCESS      PROJECT
8765   127.0.0.1    56922    python3.13   -
note: other users' ports are not visible without sudo
```

- `json.tool` exited with 0.
- `| head -1` printed the header and exited with 0, with no panic.
- 18 rows showed on an ordinary day here, including system services such as `rapportd` and `ControlCenter`, each with its `*` address shown as `0.0.0.0` or `::`.

## Pitfalls

- **Treating an empty result as "nothing is listening".** Word your friendly message carefully: "nothing you can see is listening".
- **A failed `lsof` that prints an empty table.** A scan error and an empty list are different results. Check that a broken `lsof` reaches your error path.
- **A JSON field that only exists on one system.** The socket ID is a good example. If `--json` is your public API, decide whether it belongs there before people depend on it.

## Changes to "Done when"

Add:

- [ ] The note about hidden ports appears when you are not root, and stays out of `--json`
- [ ] Platform code sits behind `#[cfg]`, with `scan()` as its only way out

## Tested on

macOS 27.2 (build 26B123), Apple M1 Max, 2026-11-02. clap 4.6.7, serde_json 1.0.154. `cargo run` results came from a throwaway reference build in a temp folder, not from a finished `portdash`, so your wording will differ.

The generic example passed `cargo check --target x86_64-unknown-linux-gnu`, so its Linux side compiles, and an `x86_64-apple-darwin` build of the reference ran under translation. Not tested: running on real Linux, a real Intel Mac, and running as root.
