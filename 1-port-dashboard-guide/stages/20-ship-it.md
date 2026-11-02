# Stage 20: Ship it

On a Mac? Also read [this stage's Mac notes](20-ship-it-mac-support.md).

**Goal:** someone who has never heard of your tool can find it, understand it in 30 seconds, install it with one command, and use it.

Your goal from the very start was a tool others actually use. Everything before this stage was only preparation.

## Background

### Check your name and your competition

Do this first, since renaming later hurts.

- **The name.** `portdash` is only a working name. Check that the one you choose is free on [crates.io](https://crates.io), on GitHub, and in the AUR. It should be short and easy to type, since people type it constantly. When this guide was written, `cargo info` found no crate named `portdash`, but check again yourself.
- **The competition.** Try `lsport` and anything else you find when you search for port TUIs. Be honest: what does yours do better? The answer goes in your README. If you cannot answer, spend another week on what makes yours special before launching.

### The README sells the tool

Most people decide in seconds. In order, your README needs:

1. **One sentence** saying what it does.
2. **A GIF or screenshot.** For a TUI this matters more than anything else. Use [VHS](https://github.com/charmbracelet/vhs) to record from a script, or `asciinema`. Keep it under 15 seconds.
3. **Install commands** to copy and paste.
4. **Usage**, with the keys.
5. **Why this over `lsof`/`ss`?** Show the project column.
6. **Known limits**, from Stage 19. Honesty builds trust.
7. **A license.**

### License

Without a license, nobody may legally use your code. The Rust community's usual choice is dual `MIT OR Apache-2.0`. It is your decision, and worth a read of what each one means. Add the license files, and set the field in `Cargo.toml`.

### Distribution

Rank these by how many people can use them, and how easy they are:

| Channel | Who it reaches | How |
|---|---|---|
| **GitHub Releases** | Everyone, no Rust needed | Prebuilt binaries, built by CI |
| **`cargo install`** | Rust users | `cargo publish` |
| **AUR** | Arch users (your own community) | A `PKGBUILD` |
| **Homebrew** | macOS and Linux users | Later, once macOS works |
| **`cargo binstall`** | Rust users, with no compile wait | Works from GitHub Releases |

### Static binaries

A normal Rust binary links to your system's `glibc`, so a binary built on your up-to-date Arch machine may fail on an older Ubuntu with a "GLIBC version not found" error. Building for the `musl` target gives you a static binary that runs on nearly any Linux:

```sh
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

Check it:

```sh
ldd target/x86_64-unknown-linux-musl/release/portdash
# statically linked
```

Also think about `aarch64` for ARM machines. [`cargo-dist`](https://github.com/axodotdev/cargo-dist) can automate most of this, along with installers and the GitHub release. Look at it before writing your own workflow.

### CI

Every push should run your quality checks automatically. Every version tag should build and publish a release. GitHub Actions is free for public repositories.

### Versioning

Follow [SemVer](https://semver.org). Start at `0.1.0`. Before `1.0`, you are allowed to change things as you learn. Your `--json` output and command names are your public API.

Add a `CHANGELOG.md`. Tools like `git-cliff` can generate one from commit messages.

### Telling people

A launch is a one-time chance, so use it when the tool is ready. Share in places where your audience already hangs out, for example `r/rust`, `r/commandline`, Hacker News (Show HN), the `ratatui` showcase, and the Omarchy community. Lead with the GIF, and with the problem you solve. Then watch the issues closely in the first week, because that is when people actually use the tool and tell you what is wrong.

## Your task

1. Pick your final name, and rename the project if needed.
2. Fill in `Cargo.toml`: description, license, repository, keywords, categories, and `readme`.
3. Set a release profile.
4. Write the README, with a recorded GIF.
5. Add license files.
6. Create a public GitHub repository.
7. Add a CI workflow that runs format, clippy and tests on each push.
8. Add a release workflow that builds static binaries on a tag.
9. Test the install on a clean system. A Docker container is perfect for this.
10. Publish with `cargo publish`. Be sure before running it, since crates.io versions are permanent. Run `cargo publish --dry-run` first.
11. Tag `v0.1.0`.
12. Submit to the AUR, if you want.
13. Share it.

## Generic example

Metadata and a release profile in `Cargo.toml`:

```toml
[package]
name = "bookshelf"
version = "0.1.0"
edition = "2024"
description = "Track the books on your shelf from the terminal"
license = "MIT OR Apache-2.0"
repository = "https://github.com/you/bookshelf"
readme = "README.md"
keywords = ["cli", "tui", "books"]
categories = ["command-line-utilities"]

[profile.release]
lto = true
codegen-units = 1
strip = true
```

A minimal CI workflow:

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - run: cargo fmt --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test
```

Check the current major versions of these actions when you set this up.

## Verify

**The clean machine test.** This is the most important one. Run a fresh container, install the way a stranger would, and try it:

```sh
docker run --rm -it ubuntu:22.04 bash
# inside: download your release binary from GitHub, using only your README's instructions
```

Does it run? Do your README's instructions work exactly as written?

**The stranger test.** Show your README to a developer friend, without saying anything else. Can they tell what it does, and install it? Watch where they get stuck.

**Checks:**

```sh
cargo publish --dry-run
ls -lh target/release/portdash    # a reasonable size, likely a few MB
```

## Pitfalls

- **Publishing a secret or junk.** `cargo package --list` shows exactly what goes up. Check it. crates.io is permanent: you can `yank` a version, but it never truly goes away.
- **A broken first impression.** Do the clean machine test before you tell anyone.
- **No GIF.** A TUI without a GIF gets skipped.
- **Launching too early, or never.** Someone will find something wrong either way. Ship when core features work and limits are documented. Restart may be marked experimental.
- **Manually editing the changelog when you use a generator.** Pick one approach and stay with it.

## Done when

- [ ] Installs on a clean machine using only your README
- [ ] The README has a GIF, install steps and known limits
- [ ] CI runs on each push
- [ ] Version `0.1.0` is published, with binaries on GitHub
- [ ] At least one other person has run it
- [ ] You told people about it

**You have shipped a real tool.**
