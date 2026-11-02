# Stage 1 on a Mac: Project setup

## What differs on a Mac, and why

Cargo works the same, but two things around it do not.

**Rust needs Apple's linker.** Rust links through `cc`, which comes with Xcode or the Command Line Tools. If `xcode-select -p` fails, run `xcode-select --install` first.

**Your notes folder may be in iCloud Drive.** If `rust_projects/` lives under `~/Library/Mobile Documents/`, iCloud also tries to sync whatever you build there. `target/` gets large quickly: a one-line project used 1.1 MB here, and a small TUI with this guide's crates used 486 MB for its debug and release builds. Build output belongs outside the synced folder.

The same layout also changes one claim in the stage. `cargo new` makes "a fresh git repository" only when the folder is not already inside one. If `rust_projects/` is part of a notes repository, you get no `.git` of your own.

## Your task on a Mac

Before step 1, do these two things if your folder is in iCloud or in another repository:

1. Point build output somewhere else, for example by adding this to `~/.zshrc`:

   ```sh
   export CARGO_TARGET_DIR="$HOME/.cargo-target/portdash"
   ```

   Later stages say `./target/...`. With this variable set, your binary is under that folder instead.

2. Decide where the project's history lives. Use `cargo new --vcs git portdash` for a separate repository inside the notes repository, or keep the project somewhere outside iCloud altogether.

## Verify

```sh
xcode-select -p
# /Applications/Xcode.app/Contents/Developer   (a Command Line Tools path is fine too)

cargo new --vcs git hello-a      # inside another repo: .git, .gitignore, Cargo.toml, src
cargo new inner                  # inside another repo: Cargo.toml and src only, no .git
CARGO_TARGET_DIR=/some/temp/folder cargo build -q   # no target/ in the project
```

All three behaved as the comments say, and `cargo run`, `cargo clippy` and `cargo fmt --check` passed on the new project.

## Pitfalls

- **Assuming you have your own repository.** `git log` inside `portdash/` shows the notes repository's history if you skipped `--vcs git`. Your commits would land there.
- **Git from another source.** `which git` may point to Nix or Homebrew instead of Apple's. It does not matter for this stage, but it matters for hooks in other guides.
- **Forgetting the variable in a new shell.** A build without `CARGO_TARGET_DIR` recreates `target/` inside iCloud.

## Changes to "Done when"

Add:

- [ ] `xcode-select -p` prints a path
- [ ] `target/` is not inside iCloud Drive
- [ ] `git log` shows the history you meant to use

## Tested on

macOS 27.2 (build 26B123), Apple M1 Max, 2026-11-02. Rust 1.97.0.

Not tested: how iCloud really behaves with a `target/` folder, because I did not build inside the synced folder. The sizes above are real, and the sync risk is reasoning from them. An Intel Mac was not tested.
