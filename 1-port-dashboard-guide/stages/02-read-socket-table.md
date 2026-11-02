# Stage 2: Read the raw socket table

On a Mac? Also read [this stage's Mac notes](02-read-socket-table-mac-support.md).

**Goal:** your program prints the kernel's raw list of TCP sockets, with proper error handling.

## Background

### What is a port, and what is listening?

A server asks the operating system for a port number. If it gets it, the program is **listening**, which means waiting for connections. Only one program can normally listen on a given port and address, which is why you see "port 3000 already in use". Your tool answers the question that follows: *who has it?*

### `/proc` is a window into the kernel

`/proc` looks like a folder, but nothing in it is stored on disk. When you read a file there, the kernel generates the content on the spot. Try it:

```sh
cat /proc/uptime
cat /proc/net/tcp
```

`/proc/net/tcp` lists the machine's IPv4 TCP sockets. This is how tools like `ss` and `netstat` can work, and you do not need any special library to read it: it is plain text.

```
  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 3500007F:0035 00000000:0000 0A 00000000:00000000 00:00000000 00000000   975        0 15411 1 ...
   1: 0100007F:C0DE 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 492943 1 ...
```

It looks cryptic, but the next stages decode it. For now, just read it.

### Errors in Rust

Reading a file can fail, so `std::fs::read_to_string` returns a `Result`. You could call `.unwrap()`, which crashes on failure. Instead, let the error travel upward with the `?` operator, so that one place decides how to report it. `main` itself can return a `Result`.

### Keeping it testable

Separate *getting* the data from *using* it. A function that takes a `&str` of file contents can be tested with any string. A function that opens a real file depends on your machine's state.

## Your task

1. Create a module `src/sockets.rs` and declare it in `main.rs` with `mod sockets;`.
2. Write a function there that reads `/proc/net/tcp` and returns its contents, or an error.
3. Print each line except the header, with a line count at the end.
4. If reading fails, print a clear message to stderr and exit with a non-zero status. Do not panic.

## Generic example

Reading a file, skipping its first line, and passing errors up:

```rust
use std::{fs, io};

fn read_entries(path: &str) -> io::Result<Vec<String>> {
    let contents = fs::read_to_string(path)?;
    Ok(contents.lines().skip(1).map(str::to_string).collect())
}

fn main() {
    match read_entries("guest-book.txt") {
        Ok(entries) => println!("{} guests", entries.len()),
        Err(err) => {
            eprintln!("could not read the guest book: {err}");
            std::process::exit(1);
        }
    }
}
```

## Verify

```sh
cargo run
```

Your output should match the kernel's. This compares the line counts:

```sh
tail -n +2 /proc/net/tcp | wc -l
```

The count may differ by a few between runs, because sockets open and close all the time.

To test the error path, temporarily point your code at a file that does not exist. You should get your message, not a panic, and `echo $?` should print a non-zero number.

## Pitfalls

- **Expecting a file size.** `ls -l /proc/net/tcp` reports size 0, because the content does not exist until it is read. Reading the whole file works fine.
- **Assuming the data stays still.** Each read is a new snapshot. Never assume two reads will match.
- **Reading the file twice** to count lines and then print them. Read once, and use what you got.

## Done when

- [ ] Sockets are read in `src/sockets.rs`
- [ ] Your output has every socket line without the header
- [ ] A read failure gives a clear message and a non-zero exit, not a panic
- [ ] Committed
