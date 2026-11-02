# Stage 3: Parse one line

On a Mac? Also read [this stage's Mac notes](03-parse-one-line-mac-support.md).

**Goal:** turn one line of `/proc/net/tcp` into a typed Rust struct, proven by tests.

This is the smallest real part of the tool. You do not touch files in this stage, only one string going in and one struct coming out.

## Background

### The fields

Splitting a line on whitespace gives you these fields, starting at index 0:

| Index | Name | Example | Meaning |
|---|---|---|---|
| 0 | `sl` | `1:` | Row number. Not useful |
| 1 | `local_address` | `0100007F:C0DE` | Address and port on this machine |
| 2 | `rem_address` | `00000000:0000` | The remote end. Zero for listeners |
| 3 | `st` | `0A` | Connection state, in hex |
| 7 | `uid` | `1000` | The user who owns it, in decimal |
| 9 | `inode` | `492943` | The socket's unique ID, in decimal |

Careful: `tx_queue rx_queue` looks like two columns in the header, but it is one field in the data (`00000000:00000000`). `tr tm->when` works the same way. So do not count columns from the header, use the table above.

Why you need these fields:

- **Port and address** are what you display.
- **State** tells you who is listening (next stage).
- **Inode** connects a socket to its process (Stage 6).
- **UID** tells you the owner even when you cannot see the process.

### Hexadecimal

The port in `0100007F:C0DE` is `C0DE`, which is hex. Hex digits go 0-9 then A-F, so `C0DE` is 49374 in decimal. Rust parses this for you with `u16::from_str_radix("C0DE", 16)`. A port always fits in a `u16`.

Leave the address (`0100007F`) as a string for now. Stage 4 decodes it.

### Modeling with types

Parsing is where bad data gets turned into a type you can trust. After this function, the rest of the code handles a real `u16`, not a string that might be a port.

Make your own error type, an `enum` listing what can go wrong. That produces better messages than `.unwrap()`, and tests can check for specific failures.

`Option` has `.ok_or(...)`, which turns "field was missing" into an error. `Result` has `.map_err(...)`, which translates one error type into yours. Together with `?` they keep parsing code short.

## Your task

1. Define a struct holding the local address (raw string for now), port, state, UID and inode.
2. Define an error enum for your parser.
3. Write a function that takes `&str` and returns `Result<YourStruct, YourError>`.
4. Write tests first, or right alongside it:
   - A real line parses to the right values. Use the lines in Stage 2. Port `C0DE` must become `49374`, and port `0035` must become `53`.
   - A line with too few fields returns an error.
   - A line with a bad port (`0100007F:ZZZZ`) returns an error.
   - An empty string returns an error.

## Generic example

Parsing a line like `kitchen 1F` from a temperature sensor:

```rust
#[derive(Debug, PartialEq)]
struct Reading {
    sensor: String,
    value: u32,
}

#[derive(Debug, PartialEq)]
enum ParseError {
    MissingField(&'static str),
    BadNumber(String),
}

fn parse_reading(line: &str) -> Result<Reading, ParseError> {
    let mut fields = line.split_whitespace();
    let sensor = fields.next().ok_or(ParseError::MissingField("sensor"))?;
    let raw = fields.next().ok_or(ParseError::MissingField("value"))?;
    let value = u32::from_str_radix(raw, 16)
        .map_err(|_| ParseError::BadNumber(raw.to_string()))?;

    Ok(Reading { sensor: sensor.to_string(), value })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_good_line() {
        let reading = parse_reading("kitchen 1F").unwrap();
        assert_eq!(reading.value, 31);
    }

    #[test]
    fn rejects_a_short_line() {
        assert_eq!(parse_reading("kitchen"), Err(ParseError::MissingField("value")));
    }
}
```

Useful for the address field: `str::split_once(':')` splits a string at the first colon. `.nth(n)` on an iterator can jump ahead, but it consumes what it passes, which trips people up. Collecting the fields into a `Vec<&str>` first is simpler here.

## Verify

```sh
cargo test
cargo clippy
```

Then call your parser on every line from Stage 2 and print the structs. Every real line should parse. If one fails, the bug is in your parser, not the file.

## Pitfalls

- **Splitting on `' '`.** The columns are padded with a varying number of spaces, so this creates empty fields. `split_whitespace` handles it.
- **Mixed number bases.** The port and state are hex, but UID and inode are decimal. Inode `492943` parsed as hex is silently the wrong number, and you would not notice until Stage 6.
- **Inode size.** Use `u64`. A `u32` is likely to work on your machine, then fail on someone else's.
- **Assuming a fixed field count.** Lines have extra fields at the end. Check for the ones you need, and ignore the rest.

## Done when

- [ ] A struct and an error enum exist
- [ ] All four test cases pass
- [ ] Every line of your real `/proc/net/tcp` parses
- [ ] Committed
