# Stage 18: Restart

On a Mac? Also read [this stage's Mac notes](18-restart-mac-support.md).

**Goal:** press a key to stop a dev server and start it again with the same command, folder and environment, then see it return on the same port.

This is the hardest feature, and the one where you need to be honest with users about what it can do. Think through the design before writing code.

## Background

### What "restart" means

To restart a process you need to know how it was started, and a process does not store a recipe. You can only rebuild it from what `/proc` shows:

| What | Where | Note |
|---|---|---|
| Program and arguments | `/proc/<pid>/cmdline` | From Stage 7 |
| Working directory | `/proc/<pid>/cwd` | From Stage 7 |
| Environment | `/proc/<pid>/environ` | NUL-separated, same format as `cmdline` |

So the plan is: snapshot those three, stop the process, wait for the port to free up, and start a new one.

Take the snapshot **before** stopping. They all disappear as soon as the process exits.

### The environment contains secrets

`/proc/<pid>/environ` often contains API keys and database passwords. You need it, because servers usually depend on those variables. So apply strict rules:

- Keep it in memory only for as long as the restart takes.
- Never display it. Never log it. Never include it in `--json`.
- Never write it to a file.
- Make sure your structs do not `#[derive(Debug)]` it into an error message somewhere.

Also, `environ` only shows the environment at process start. If the program changed its own variables later, you will not see that. This is usually fine.

### Detaching the new process

Your new server has to outlive your dashboard, and must never touch its screen. Three things make that work:

1. **Redirect stdin, stdout and stderr.** If a child inherits your terminal, its logs print right over your TUI. Send stdin to `/dev/null`, and output to a log file.
2. **Give it its own process group.** `process_group(0)` means `Ctrl+C` or a signal sent to your dashboard's group does not hit the server.
3. **Do not wait for it.** A child you never `.wait()` on becomes a zombie if it exits while you are running. That is a small problem, but it makes your list wrong (Stage 10). Think about who should collect it. A dedicated thread that calls `child.wait()` is a simple solution.

Log files belong in the XDG state folder: `$XDG_STATE_HOME/portdash/logs/`, or `~/.local/state/portdash/logs/` when the variable is not set. Show the path in your status message, because the user's server no longer prints to the terminal it started in.

### Waiting for the port

After the stop, the port is not always free immediately. Poll until no listener holds it, with a timeout. After starting the new process, poll until the port is listening again, and only then report success. A server may need many seconds to boot, so be generous here, for example 30 seconds. Report failure clearly, and point at the log file.

### What restart cannot do

You need to detect these or, at the very least, warn about them. Consider each one:

- **Wrappers.** For `npm run dev`, the process tree looks like `npm` → `sh` → `node`, and only `node` holds the port. Restarting `node` alone replaces the child, with no `npm` above it. It usually works, and the arguments are real, but it is not quite what was first run.
- **Watchers.** `nodemon`, `cargo watch` and similar tools restart servers themselves. If you stop the child, the watcher may restart it, and then yours fails since the port is taken. Detect this by checking if the port is taken again during your wait, and report it instead of starting a second copy.
- **Supervisors.** `systemd` or Docker own that process, and they will fight you. A parent of PID 1 is a clue, although on many systems orphans also get reparented to PID 1, so treat it only as a hint.
- **The terminal link.** When a user started a server in a terminal tab, its logs go to that tab. After your restart, they go to your log file, and that original tab returns to the shell prompt. Users will find this surprising, so tell them.
- **Other users' processes.** You cannot read `environ` for them, so refuse.

**Be honest in the UI.** Before the restart, show the exact command and folder in the confirmation popup. Consider marking this as "experimental" in your README. A restart that says what it cannot do is better than one that sometimes silently does the wrong thing.

### A safer alternative worth considering

Instead of guessing from `/proc`, you could add `portdash run -- npm run dev`, which starts the process itself and always knows its recipe. It is a bigger feature, but a sturdier design. Stage 21 lists it. Do not build it now.

## Your task

1. Write a `RestartPlan` type: program, arguments, folder, environment. Make sure it does not derive `Debug` in a way that prints secrets. A manual `Debug` implementation that hides the environment is a good habit.
2. Build it from a PID, and return a clear error for each reason it cannot work: another user's process, unreadable files, an empty command.
3. Write the restart sequence as a function that sends progress messages through your event channel: stopping, waiting, starting, up, failed.
4. Detect the "something else restarted it" case, and report it rather than starting a duplicate.
5. Add the key and the confirmation popup, displaying the exact command and folder.
6. Add a `restart <port>` CLI command, sharing the same code.
7. Test what you can without real processes: parsing `environ`, and each error case. Test the whole sequence by running it against a real test server in an integration test (see Stage 19).

## Generic example

Starting a program fully detached from your terminal:

```rust
use std::{fs, io, os::unix::process::CommandExt, path::Path, process::{Command, Stdio}};

fn launch_oven_timer(folder: &Path, args: &[String], vars: &[(String, String)], log: &Path)
    -> io::Result<u32>
{
    let log_file = fs::File::create(log)?;
    let child = Command::new("oven-timer")
        .args(args)
        .current_dir(folder)
        .env_clear()
        .envs(vars.iter().map(|(key, value)| (key, value)))
        .stdin(Stdio::null())
        .stdout(log_file.try_clone()?)
        .stderr(log_file)
        .process_group(0)
        .spawn()?;
    Ok(child.id())
}
```

Hiding sensitive fields from `Debug`:

```rust
use std::fmt;

struct Login {
    user: String,
    password: String,
}

impl fmt::Debug for Login {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Login")
            .field("user", &self.user)
            .field("password", &"<hidden>")
            .finish()
    }
}
```

## Verify

**The simple case:**

```sh
cd /tmp
MY_TEST_VAR=hello python3 -m http.server 8765 --bind 127.0.0.1 &
OLD=$!
cargo run -- restart 8765
```

Then confirm that:

```sh
ss -ltnp | grep 8765                          # a new PID
tr '\0' '\n' < /proc/<new-pid>/environ | grep MY_TEST_VAR    # hello
ls -l /proc/<new-pid>/cwd                     # /tmp
curl -s localhost:8765 | head -1              # it serves
```

**Quitting does not take it down:** restart it from the TUI, quit the TUI, and check it is still running.

**A bad server:** restart something that crashes immediately on start, and check you get a clear failure and a pointer to the log.

**A watcher:** this script restarts its server when it dies, and your tool must report it rather than start a duplicate:

```sh
while true; do python3 -m http.server 8768 --bind 127.0.0.1; sleep 1; done &
WATCHER=$!
cargo run -- restart 8768
kill $WATCHER; pkill -f 'http.server 8768'
```

**The real thing:** try it on a real dev server you use, like `vite` or `next dev`. See what actually happens.

## Pitfalls

- **Snapshot after stop.** `/proc/<pid>` is gone by then.
- **Output inherited by the child.** Your TUI screen fills with server logs. Check that all three streams are redirected.
- **A secret in an error message.** Search your code for anywhere the plan is formatted.
- **Deleted working folder.** `current_dir` fails on spawn, and the error is confusing. Check first, and give a clear message.
- **Relative program paths.** A command of `./server` is relative to the old folder, which is why you run from that same folder.
- **Reporting success too early.** `spawn()` succeeding only means the process started. Wait for the port.

## Done when

- [ ] A simple server restarts, with the same folder and environment, on the same port
- [ ] It survives quitting the TUI, and its logs never touch your screen
- [ ] The log path is shown to the user
- [ ] Crashes on start and watchers both give clear messages
- [ ] Secrets never appear in output, logs or `Debug`
- [ ] Limits are documented in your README
- [ ] Committed
