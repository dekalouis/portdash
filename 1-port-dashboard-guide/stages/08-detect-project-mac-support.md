# Stage 8 on a Mac: Detect the project

Nearly everything here works unchanged. The Verify steps gave `demo-app` for a server started in `/tmp/demo-app/src`. One thing goes wrong on a Mac with Homebrew.

## What differs on a Mac, and why

Homebrew's prefix is itself a git repository on Apple Silicon:

```sh
ls -d /opt/homebrew/.git
# /opt/homebrew/.git
```

A Homebrew service runs from a folder below it, such as `/opt/homebrew/var/db/redis`. So the walk up finds `.git` and labels the service as a project called `homebrew`. That is what happened here:

```
6379   127.0.0.1    1912    redis-server    homebrew
```

## Your task on a Mac

Extend the stopping rule from task 3, so the walk stops without a match at `/opt/homebrew`. `/usr/local/Homebrew` is the Intel location. Add a test using a function argument rather than the real path, so that it runs anywhere.

## Verify

The Stage 8 steps work as written. Then, if you run a Homebrew service:

```sh
cargo run | grep 6379
```

Result here, after adding the rule:

```
6379   127.0.0.1    1912    redis-server    -
```

## Pitfalls

- **Other package managers.** The same mistake can happen with any tool whose folder is a repository, so look through your real output, as the original stage asks.
- **Apps with a working folder of `/`.** Mac GUI apps and services commonly have this, and they correctly show no project.

## Changes to "Done when"

Add:

- [ ] A Homebrew service is not labeled `homebrew`

## Tested on

macOS 27.2 (build 26B123), Apple M1 Max, 2026-11-02. Homebrew at `/opt/homebrew`. `cargo run` results came from a throwaway reference build in a temp folder, not from a finished `portdash`, so your wording will differ.

Not tested: the Intel path `/usr/local/Homebrew`, which I took from Homebrew's usual layout and could not check on this machine.
