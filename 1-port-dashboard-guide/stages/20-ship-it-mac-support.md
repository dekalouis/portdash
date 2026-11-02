# Stage 20 on a Mac: Ship it

The README advice, license, versioning and launch sections stay as they are. Packaging is where a Mac differs.

## What differs on a Mac, and why

**No static binaries, and no `musl`.** A Mac binary links to Apple's system libraries. That is expected, and those libraries come with every Mac. `ldd` does not exist here, so use `otool -L`:

```sh
otool -L target/release/portdash
# /usr/lib/libiconv.2.dylib
# /usr/lib/libSystem.B.dylib
```

If anything outside `/usr/lib` or `/System` shows up, such as a Homebrew path, it will fail on someone else's Mac.

**Two architectures.** The targets are `aarch64-apple-darwin` and `x86_64-apple-darwin`. Ship both, or combine them with `lipo`:

```sh
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
lipo -create target/release/portdash target/x86_64-apple-darwin/release/portdash -output portdash-universal
lipo -archs portdash-universal      # x86_64 arm64
```

The combined file kept an ad-hoc signature and ran here, and the Intel build ran under translation and listed the real sockets.

**Gatekeeper is the big one.** A browser marks a download with a quarantine attribute. To simulate that, I set the attribute on a copy of my binary, which was only ad-hoc signed by the linker:

```sh
xattr -w com.apple.quarantine "0181;00000000;Safari;" ./copy
timeout 10 ./copy --version       # no output, killed by timeout (exit 124)
```

It **hung, with no message at all in a non-GUI shell**. Removing the attribute with `xattr -d` did not fix that same file in my test, though a fresh copy that had never been quarantined ran fine. `spctl --assess --type exec` printed `rejected` for the build.

So a stranger who downloads your binary with a browser gets a poor first impression. You have these options:

| Path | Quarantined? | Cost |
|---|---|---|
| `cargo install` | No, it builds locally | Needs Rust |
| Homebrew tap | Not tested | A formula to maintain |
| `curl` in a terminal | Generally no, but not tested here | Tell people plainly |
| Signed and notarized release | Designed for this | Apple Developer Program membership |

Apple's process is [notarization with a Developer ID](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution.md), using `notarytool`. See also [Apple's overview of distributing outside the store](https://developer.apple.com/macos/distribution/).

**CI runners.** A `macos-latest` runner is arm64. Intel builds need a separate label such as `macos-15-intel`. Labels change often, so check [GitHub's runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners) when you set this up. As of this writing, `macos-13` is retired and `macos-14` is ending. Add a macOS job to the CI matrix, so the `#[cfg]` split from Stage 9 gets built on both systems.

**There is no Docker clean-machine test.** A Linux container cannot run a Mac binary. A second macOS user account or a VM comes closest.

**AUR does not apply,** and Homebrew moves up in the distribution table: it is how many Mac developers install tools.

## Your task on a Mac

Change these tasks:

8. Build for both Mac targets in the release workflow, in addition to Linux, and run `otool -L` on the results.
9. Test the install as a stranger would: a new macOS user account, using only your README.
12. Replace the AUR with a Homebrew tap, if you want one.

Add to the README: what happens when a download is quarantined, and the install path you recommend.

## Generic example

Add a job to the guide's workflow, with a matrix:

```yaml
# .github/workflows/ci.yml
jobs:
  check:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    runs-on: ${{ matrix.os }}
```

## Verify

```sh
cargo build --release
otool -L target/release/portdash
codesign -dv target/release/portdash 2>&1 | grep -E 'Signature|flags'
lipo -archs target/release/portdash
cargo package --list
cargo publish --dry-run
```

Results with the guide's release profile, including `lto` and `strip`:

```
/usr/lib/libiconv.2.dylib, /usr/lib/libSystem.B.dylib
flags=0x20002(adhoc,linker-signed)
Signature=adhoc
arm64
```

After `strip = true` the signature was still present, and the stripped binary ran. `cargo package --list` showed only source files and metadata, and `cargo publish --dry-run --allow-dirty` ended with `aborting upload due to dry run`.

## Pitfalls

- **Testing only on your own Mac.** Your own builds have no quarantine attribute, so the problem above stays invisible until a stranger hits it.
- **Linking to a Homebrew library.** It works for you and fails for everyone else. `otool -L` catches it.
- **`ldd` in a release script.** It does not exist here.
- **Lost signatures.** Post-processing a binary can break its signature. Check with `codesign -dv` after every step.

## Changes to "Done when"

Change the first two items to:

- [ ] Installs on a clean macOS account using only your README
- [ ] The README states what happens to quarantined downloads, and which install path you recommend

Add:

- [ ] CI builds on macOS as well as Linux

## Tested on

macOS 27.2 (build 26B123), Apple M1 Max, 2026-11-02. Rust 1.97.0. `cargo run` results came from a throwaway reference build in a temp folder, not from a finished `portdash`, so your wording will differ.

Not tested: anything that publishes, including a real `cargo publish`, a GitHub release, or a CI run. Notarization, a Homebrew tap, a real Intel Mac, a browser download, and a second user account were not tested either. The dialog a person sees when opening a quarantined binary from Finder was not checked. The notarization and CI runner details come from the linked pages.
