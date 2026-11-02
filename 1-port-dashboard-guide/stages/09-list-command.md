# Stage 9: The list command

On a Mac? Also read [this stage's Mac notes](09-list-command-mac-support.md).

**Goal:** a clean, finished `portdash list` command, with an aligned table and JSON output. By the end of this stage your tool is already useful to other people.

## Background

### Pull your code together

So far you have sockets, processes and projects as separate pieces. Now define the one type the rest of the app works with: a `Listener` holding the port, address, UID, owners and project.

Add one function, something like `scan()`, that does everything and returns `Vec<Listener>`. Every later feature, including the whole TUI, calls this. A clean boundary here will make Part 4 far easier, so make the module layout tidy before you continue.

### One row or two?

A server on `0.0.0.0:3000` and `[::]:3000` holds two sockets, with one process. A server with four workers has one socket and four PIDs. You need to choose how to show those. A reasonable rule: one row per socket (it is the honest view), and when several PIDs share a socket, show the first with a `+3` marker. A different rule is fine too, as long as you can explain it.

### Parsing arguments with `clap`

`clap` turns a struct into a command line parser. You describe the arguments as types, and you get `--help`, `--version` and error messages for free. Its `derive` feature enables the macro style shown below:

```sh
cargo add clap --features derive
```

### Output for humans and for scripts

Terminal users want aligned columns. Scripts want a stable format. That is why serious CLI tools have a `--json` flag. With it, someone can write `portdash list --json | jq` in their own script, without you having to predict what they need.

`serde` handles this: derive `Serialize` on a struct, and `serde_json` prints it. Note that `IpAddr` needs serde's support, which it has out of the box. For `PathBuf`, serde works as long as the path is valid UTF-8, and otherwise returns an error. Consider storing display strings in your JSON struct, or use a lossy conversion.

### stdout and stderr

Results go to stdout. Warnings and errors go to stderr. This keeps `portdash list --json | jq` working even when you print a warning.

### Exit codes

`0` means success, and anything else means failure. Scripts rely on it.

## Your task

1. Create the `Listener` type and `scan()` function, and move the display logic out of `main`.
2. Add `clap` with a `list` subcommand and a `--json` flag.
3. Print an aligned table with columns: port, address, PID, process, project.
4. Print JSON with `--json`.
5. Show a friendly message when nothing is listening.
6. Add a footer note if some owners were hidden by permissions, such as `2 ports have unknown owners (try sudo)`.

## Generic example

Subcommands with `clap`:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "Manage a bookshelf")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// List every book
    List {
        /// Print as JSON
        #[arg(long)]
        json: bool,
    },
    /// Remove one book
    Remove { id: u32 },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::List { json }) => println!("listing, json = {json}"),
        Some(Command::Remove { id }) => println!("removing {id}"),
        None => println!("no command given"),
    }
}
```

The `///` comments become the `--help` text. `Option<Command>` makes the subcommand optional, and in Stage 11 running with no command will open the TUI.

Aligned columns using a width computed from the data:

```rust
fn print_rows(rows: &[(String, u32)]) {
    let width = rows.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
    for (name, count) in rows {
        println!("{name:<width$}  {count:>5}");
    }
}
```

Printing JSON:

```rust
use serde::Serialize;

#[derive(Serialize)]
struct BookJson {
    title: String,
    pages: u32,
}

fn print_json(books: &[BookJson]) -> serde_json::Result<()> {
    println!("{}", serde_json::to_string_pretty(books)?);
    Ok(())
}
```

## Verify

```sh
python3 -m http.server 8765 --bind 127.0.0.1 &
cargo run -- list
cargo run -- list --json | python3 -m json.tool
cargo run -- list | head -1
cargo run -- --help
kill %1
```

- The table lines up, including long project names.
- The JSON is valid.
- `| head -1` must not print a panic message. Read the first pitfall.
- With nothing unusual going on, the warnings do not end up in the JSON stream.

## Pitfalls

- **Panic on a closed pipe.** Rust's `println!` panics if the reader goes away: `portdash list | head -1` can produce `failed printing to stdout: Broken pipe`. Use `writeln!` on `io::stdout()`, and treat `ErrorKind::BrokenPipe` as a normal exit.
- **`len()` is bytes, not screen width.** A non-ASCII project name misaligns your columns. The `unicode-width` crate is the proper answer. Note it in a `shortcut:` comment if you decide to leave it for now.
- **JSON changes are breaking changes.** Once people script against it, renaming a field breaks their script. Pick names you can live with.
- **Printing a very long command.** A full command line can be thousands of characters. Leave it out of the table, and include it in JSON.

## Done when

- [ ] `portdash list` prints an aligned table
- [ ] `portdash list --json` prints valid JSON
- [ ] `portdash list | head -1` exits with no panic
- [ ] `--help` reads well
- [ ] `scan()` is the single entry point for the data
- [ ] Committed

**Milestone:** this is a real tool. Try using it for a day, and note what annoys you.
