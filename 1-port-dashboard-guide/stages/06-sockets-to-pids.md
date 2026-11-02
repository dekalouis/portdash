# Stage 6: Map sockets to processes

On a Mac? Also read [this stage's Mac notes](06-sockets-to-pids-mac-support.md).

**Goal:** show which process ID (PID) owns each listening port.

This is the heart of the tool, and where it stops being a parser and starts being useful.

## Background

### There is no direct link

`/proc/net/tcp` gives you an inode, but no process. To connect them, you need to look at it from the other direction: go through the processes and see which sockets each one holds.

### Everything is a file descriptor

When a program opens a file or a socket, the kernel gives it a small number called a **file descriptor** (fd). `/proc` exposes every process's descriptors:

```
/proc/<pid>/fd/0  ->  /dev/pts/3
/proc/<pid>/fd/1  ->  /dev/pts/3
/proc/<pid>/fd/4  ->  socket:[492943]
```

Each entry is a symbolic link. If the link's target looks like `socket:[492943]`, this process holds the socket whose inode is `492943`. That is the same number you parsed in Stage 3.

Try it on your test server:

```sh
python3 -m http.server 8765 --bind 127.0.0.1 &
ls -l /proc/$!/fd
kill %1
```

### The algorithm

1. List everything in `/proc`. Numeric folder names are PIDs.
2. For each PID, list `/proc/<pid>/fd`.
3. For each entry, read the symlink target.
4. If it is a socket, record `inode -> pid` in a map.
5. Join the data: for each listener, look up its inode.

A `HashMap` fits well here, since lookups are constant time however many entries there are. Build the map once and then look up each listener. Do not rescan everything per listener.

Think about the map's value type. Can more than one process hold the same socket? Yes: when a process forks, the child inherits its open sockets. Servers with worker processes do this all the time. A map from inode to a single PID would silently lose owners.

### Permissions

You can only read `/proc/<pid>/fd` for your own processes. For others you get "permission denied":

```sh
ls /proc/1/fd
# Permission denied
```

This is normal, not an error. A port owned by a root process shows up in the socket table, but you cannot discover its PID. Your data model has to allow for that: a listener with no known owner is a valid state, and it needs to appear in your output anyway. You still know its UID from Stage 3, so you can say a little about it.

### Things disappear while you look

Processes exit constantly. Between listing `/proc` and reading a process's `fd` folder, that process may be gone. Every read can fail, so a failure for one process means "skip it", never "abort the scan".

## Your task

1. Create a module `src/process.rs`.
2. Write a function that scans `/proc` and returns the map from inode to owners.
3. Write a small helper that extracts the inode from a link target such as `socket:[492943]`, returning `None` for anything else, and test it. Cases to test: `socket:[123]`, `/dev/null`, `socket:[abc]`, `pipe:[123]`.
4. Join this with your listeners, and print the port and PIDs. Show something clear for unknown owners, such as `-`.

## Generic example

Going through a folder of symlinks, and tolerating failures:

```rust
use std::{collections::HashMap, fs, io, path::{Path, PathBuf}};

fn index_links(dir: &Path) -> io::Result<HashMap<PathBuf, PathBuf>> {
    let mut found = HashMap::new();
    for entry in fs::read_dir(dir)? {
        // One unreadable entry should not stop the whole scan.
        let Ok(entry) = entry else { continue };
        let Ok(target) = fs::read_link(entry.path()) else { continue };
        found.insert(target, entry.path());
    }
    Ok(found)
}
```

Building a map where one key has many values:

```rust
use std::collections::HashMap;

fn group_by_author(books: &[(String, String)]) -> HashMap<&str, Vec<&str>> {
    let mut by_author: HashMap<&str, Vec<&str>> = HashMap::new();
    for (author, title) in books {
        by_author.entry(author.as_str()).or_default().push(title.as_str());
    }
    by_author
}
```

Useful string helpers: `strip_prefix` and `strip_suffix` return `None` if the text does not match, so they do validation and extraction at once.

## Verify

```sh
python3 -m http.server 8765 --bind 127.0.0.1 &
echo "server PID: $!"
cargo run
ss -ltnp
kill %1
```

- Your output must show port 8765 with the same PID.
- `ss -ltnp` (`-p` adds processes) is your reference. It has the same permission limits as you, so it also shows no process for other users' ports.
- Run with `sudo` to see owners appear for the system ports: `cargo build && sudo ./target/debug/portdash`.

Test the shared socket case. This server forks, so two processes hold one socket, and you should see two PIDs:

```sh
python3 -c '
import socket, os, time
s = socket.socket(); s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("127.0.0.1", 8767)); s.listen(); os.fork(); time.sleep(60)' &
```

## Pitfalls

- **Stopping on permission errors.** If `read_dir` on `/proc/1/fd` returns an error and you use `?`, your scan dies on PID 1. Skip it instead.
- **Non-numeric entries.** `/proc` also contains `cpuinfo`, `net`, `self` and so on. Keep only names that parse as numbers.
- **Threads.** `/proc/<pid>/task/` holds threads, but they share the same descriptors. Scanning `/proc/<pid>/fd` is enough.
- **Performance.** A browser may hold thousands of descriptors. Scanning should still take only milliseconds, but measure it (`time ./target/release/portdash`) and remember the result for Stage 14.

## Done when

- [ ] Each listener shows its PIDs, or a clear "unknown"
- [ ] Your test server shows the right PID
- [ ] The forked server shows two PIDs
- [ ] Permission errors are skipped silently
- [ ] The helper has tests
- [ ] Committed
