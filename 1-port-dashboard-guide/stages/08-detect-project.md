# Stage 8: Detect the project

On a Mac? Also read [this stage's Mac notes](08-detect-project-mac-support.md).

**Goal:** label each listener with the project it belongs to. This is what makes your tool different from `lsof`.

## Background

### Why this feature matters

`ss` tells you: "node, PID 48121, port 3000". What you want to know is: "the `shop-frontend` dev server is on port 3000". Going from a process to a project is the difference between a raw system tool and a developer tool, so give this stage some thought even though the code is short.

### How to recognize a project

You have a working directory from Stage 7. A dev server normally runs from inside its project, so walk up the folder tree looking for a file that marks a project root:

| Marker | Project type |
|---|---|
| `.git` | Any git repository |
| `package.json` | JavaScript / TypeScript |
| `Cargo.toml` | Rust |
| `pyproject.toml`, `requirements.txt` | Python |
| `go.mod` | Go |
| `composer.json` | PHP |
| `Gemfile` | Ruby |
| `pom.xml`, `build.gradle` | Java |

Given `/home/you/shop/frontend/src`, check `src`, then `frontend`, and so on up. `Path::ancestors()` gives you exactly this walk.

### Design decisions

These are real questions with no single right answer. Think about what you would want to see, and choose.

**Which marker wins in a monorepo?** Suppose `shop/.git` and `shop/frontend/package.json` both exist. The nearest marker gives `frontend`, and the top one gives `shop`. The nearest is usually more specific, and so more useful.

**What is the project's name?** The folder name is simple and almost always good enough. A `package.json` might have a `name` field that is better, but reading it adds a dependency on file formats. Start with the folder name.

**Where should the walk stop?** See the first pitfall. It matters.

### Keep it testable with real folders

This function touches the filesystem, so its tests need folders. Create them in a temp directory. A test can build `tmp/shop/frontend/src`, put a marker in `shop/frontend`, and assert on the result. To make tests independent, give each its own folder name.

## Your task

1. Create a module `src/project.rs`.
2. Write a function that takes a starting directory and returns the project's root and name, or `None`.
3. Include the stopping rule, so you do not label things as belonging to your home folder.
4. Add tests:
   - Finds a marker in the starting folder
   - Finds a marker in a parent
   - Prefers the nearest of two markers
   - Returns `None` when there is no marker
5. Add a project column to your output.

## Generic example

Walking upward until you find something:

```rust
use std::path::{Path, PathBuf};

fn find_recipe_root(start: &Path, markers: &[&str]) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| markers.iter().any(|marker| dir.join(marker).exists()))
        .map(Path::to_path_buf)
}
```

`ancestors()` starts with the path itself, and ends at `/`. `Path::file_name()` returns the last part of a path as an `Option<&OsStr>`.

## Verify

```sh
mkdir -p /tmp/demo-app/src && touch /tmp/demo-app/package.json
cd /tmp/demo-app/src && python3 -m http.server 8765 --bind 127.0.0.1 &
cargo run
kill %1
rm -r /tmp/demo-app
```

The listener on 8765 should be labeled `demo-app`, even though the server started in `src/`.

Now look at the real output. Do the labels make sense for the other things running on your machine? Does anything look wrong? This is the best bug-finding step you have.

## Pitfalls

- **Your home folder as a project.** Many people keep dotfiles in `~/.git`. A background service running from anywhere under home would then be labeled as your dotfiles "project". Stop the walk at the home folder (`std::env::home_dir()` or `$HOME`), and do not count home itself as a project.
- **Other users' home folders.** If you run as root, your home is not their home. Compare against the process owner when you can, or accept the limit and note it in a comment.
- **System folders.** A service with a working directory of `/` should have no project.
- **Tests using one shared temp folder.** Tests run in parallel by default, so they will interfere with each other. Give each test a unique folder.
- **`.git` can be a file.** In git worktrees and submodules, `.git` is a file rather than a folder. Using `.exists()` handles this. `.is_dir()` would not.

## Done when

- [ ] Listeners display a project name when one exists
- [ ] A process in `src/` is labeled with its project root
- [ ] The home folder never counts as a project
- [ ] All four tests pass
- [ ] Committed
