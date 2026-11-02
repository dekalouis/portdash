# Stage 1: Project setup

On a Mac? Also read [this stage's Mac notes](01-project-setup-mac-support.md).

**Goal:** a Rust project that builds, runs, has one passing test, and is under version control.

## Background

### Cargo

Cargo is Rust's build tool and package manager. These commands will become habit:

| Command        | What it does                                            |
| -------------- | ------------------------------------------------------- |
| `cargo run`    | Compile and run                                         |
| `cargo test`   | Compile and run the tests                               |
| `cargo check`  | Check for errors without building a binary. Much faster |
| `cargo clippy` | Lint your code and suggest more idiomatic Rust          |
| `cargo fmt`    | Format your code                                        |

### Binary crate layout

`cargo new` gives you `Cargo.toml` (metadata and dependencies), `src/main.rs` (your program's entry point) and a fresh git repository. As the project grows, you will add modules: `src/sockets.rs` and so on.

### Why a test on day one

Most of this project is parsing text with fiddly formats. Tests are how you will know the parsing works, because you cannot eyeball hex all day. Rust has tests built in, so there is nothing to install. A test is a function marked `#[test]`, usually in a `tests` module at the bottom of the same file.

## Your task

1. Go to `rust_projects/` and create a binary project called `portdash`.
2. Make it print a short greeting from a small function, not directly from `main`. For example, a function that returns the app's name and version as a `String`.
3. Write one test for that function.
4. Make sure `cargo run`, `cargo test`, `cargo clippy` and `cargo fmt --check` all pass.
5. Make your first commit.

## Generic example

A function, a `main` that uses it, and a test for it:

```rust
fn greeting(name: &str) -> String {
    format!("Hello, {name}!")
}

fn main() {
    println!("{}", greeting("reader"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greets_by_name() {
        assert_eq!(greeting("Ada"), "Hello, Ada!");
    }
}
```

Tip: `env!("CARGO_PKG_VERSION")` gives you the version from `Cargo.toml` at compile time.

## Verify

```sh
cargo run             # prints your greeting
cargo test            # 1 passed
cargo clippy          # no warnings
cargo fmt --check     # no output
git log --oneline     # shows your commit
```

## Pitfalls

- **Running `cargo new` inside the guide folder.** Keep `portdash/` next to `1-port-dashboard-guide/`, not inside it.
- **Committing `target/`.** `cargo new` adds a `.gitignore` for it. Leave that in place, since build output gets very large.
- **Removing `Cargo.lock`.** For an application, commit it. It makes your builds reproducible.

## Done when

- [x] `cargo run` prints your greeting
- [x] `cargo test` passes with one test
- [x] `cargo clippy` and `cargo fmt --check` are clean
- [x] The project is committed
