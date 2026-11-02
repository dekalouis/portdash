# Stage 3 on a Mac: Parse one record

## What differs on a Mac, and why

There are no columns or hex here. `lsof -F` prints one field per line, where the **first character names the field** and the rest is the value. Fields describe a process first, then each of its open sockets:

```
p52733              process ID
R52704              parent process ID
cpython3.13         command name, which may contain spaces
u501                user ID
f3                  file descriptor: a new socket starts here
tIPv4               address family
d0x62a0271f722ff24b socket ID, which Stage 6 uses
n127.0.0.1:8771     address and port
f4                  the same process, a second socket
tIPv6
d0x68567e90bea0bf31
n[::1]:8771
p52734              the next process
...
```

So the parser carries state: remember the latest process fields, remember the socket fields, and emit a socket when you reach `n`. One process can hold many sockets, which is why it is a list and not one struct per process.

The port is plain decimal, so the hex section of the original does not apply. Only the part after the last `:` is the port.

| Original field | Mac source |
|---|---|
| `local_address` | `n`, before the last `:` |
| Port | `n`, after the last `:` |
| `st` | None. `-sTCP:LISTEN` already filtered |
| `uid` | `u` |
| `inode` | `d`, a hex string. Keep it as a `String` |

## Your task on a Mac

1. Define a struct holding: PID, parent PID, name, UID, family, socket ID, the raw address text and the port.
2. Define an error enum, as in the original.
3. Write a function that takes the whole `&str` of output and returns a `Result` with a list of your structs.
4. Test with these cases:
   - A process with one socket gives one record, with port `8771`
   - A process with two sockets gives two records with the same PID
   - Two processes do not leak fields into each other
   - A name with a space, `cLM Studio`, stays whole
   - `n127.0.0.1:ZZZZ` returns an error
   - An empty string returns an empty list, because no listeners is a normal result

## Generic example

A catalog where each line starts with a tag, and later lines belong to the earlier ones:

```rust
#[derive(Debug, PartialEq)]
struct Copy {
    author: String,
    title: String,
    shelf: u32,
}

fn parse_catalog(text: &str) -> Result<Vec<Copy>, String> {
    let mut author = String::new();
    let mut title = String::new();
    let mut copies = Vec::new();

    for line in text.lines() {
        let mut chars = line.chars();
        let Some(tag) = chars.next() else { continue };
        let value = chars.as_str();
        match tag {
            'a' => author = value.to_string(),
            't' => title = value.to_string(),
            's' => {
                let shelf = value.parse().map_err(|_| format!("bad shelf: {value}"))?;
                copies.push(Copy {
                    author: author.clone(),
                    title: title.clone(),
                    shelf,
                });
            }
            _ => {} // Ignore tags you do not use.
        }
    }
    Ok(copies)
}

fn main() {
    let copies =
        parse_catalog("aLe Guin\ntThe Dispossessed\ns4\ns9\naHerbert\ntDune\ns2\n").unwrap();
    assert_eq!(copies.len(), 3);
    assert_eq!(
        copies[1],
        Copy {
            author: "Le Guin".into(),
            title: "The Dispossessed".into(),
            shelf: 9
        }
    );
    assert_eq!(copies[2].author, "Herbert");
    assert!(parse_catalog("aX\ntY\nsZZ\n").is_err());
    println!("ok");
}
```

`chars().next()` and `chars().as_str()` split off the tag without byte-index slicing.

## Verify

```sh
cargo test
cargo clippy
/usr/sbin/lsof -nP -iTCP -sTCP:LISTEN -FpcuRtdn | head -20
```

Check your parsed list against that output. On this Mac the fields arrived in the order `p`, `R`, `c`, `u`, then `f`, `t`, `d`, `n` for each socket, and a test parser built this way read every record correctly.

## Pitfalls

- **Assuming one line per socket.** Process fields appear once, however many sockets follow.
- **Forgetting to reset socket fields at `f`.** A missing `t` would otherwise reuse the previous socket's family.
- **Splitting on `:` from the left.** `[::1]:8772` has several colons. Cut at the last one.
- **Treating `d` as a number.** It is an ID, and comparing it as text is all you need.

## Changes to "Done when"

Replace the second and third items with:

- [ ] The six test cases above pass
- [ ] Every record from your real `lsof` output parses

## Tested on

macOS 27.2 (build 26B123), Apple M1 Max, 2026-11-02. `lsof` 4.91. `cargo run` results came from a throwaway reference build in a temp folder, not from a finished `portdash`, so your wording will differ.

Not tested: an Intel Mac.
