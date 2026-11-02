# Stage 5: Add IPv6

On a Mac? Also read [this stage's Mac notes](05-add-ipv6-mac-support.md).

**Goal:** show IPv6 listeners too, in one combined list that matches `ss -ltn`.

Do not skip this. Many dev servers bind to `::1` or `::`. If you read IPv4 only, those servers would be missing and your tool would be wrong without anyone noticing.

## Background

### IPv6 addresses

An IPv6 address is 16 bytes rather than 4, written as hex groups. Two you will see all the time:

- `::1` is IPv6's loopback, like `127.0.0.1`.
- `::` is "all interfaces", like `0.0.0.0`.

`std::net::Ipv6Addr` can be built from `[u8; 16]`, and it handles the short `::` form when printing.

### The file format

`/proc/net/tcp6` has the same layout as `/proc/net/tcp`, with a longer address:

```
0: 00000000000000000000000001000000:1538 000...000:0000 0A ...
```

### The tricky part: byte order within groups

An obvious guess would be that the 32 hex characters are one big number in reverse, or that the bytes are simply in order. Neither is right.

The kernel treats the 16 bytes as **four 32-bit integers**, and prints each one separately, using the same process you decoded in Stage 4. So you need to:

1. Split the 32 characters into four groups of 8.
2. Decode each group the same way as IPv4, which gives 4 bytes.
3. Concatenate the four results into 16 bytes.

Check it by hand with `::1`. In real byte order it is fifteen zero bytes, then `01`. The final 4-byte group is `00 00 00 01`, and reversing that gives `01 00 00 00`. That matches the end of the string above: `...01000000`.

You already wrote the function for one group in Stage 4. Reuse it.

### Representing either kind of address

`std::net::IpAddr` is an enum in the standard library with `V4` and `V6` variants. Your struct can hold an `IpAddr`, and the rest of your code does not need to care which kind it is. Sorting and printing just work.

### IPv4-mapped addresses

You may see something like `0000000000000000FFFF00000100007F`. This is `::ffff:127.0.0.1`, an IPv4 address written in IPv6 form. You can decode it the same way, and it prints correctly without special handling.

## Your task

1. Add a decoder for 32-character addresses, reusing your IPv4 group logic.
2. Change your struct's address to `IpAddr`.
3. Have your parser pick the right decoder. Think about how: the length of the hex string tells you, without an extra parameter.
4. Read both files and merge them into one list, sorted by port.
5. Handle a missing `/proc/net/tcp6`. A system with IPv6 turned off may not have it, and that should not be fatal.
6. Test with these cases:
   - `00000000000000000000000001000000` is `::1`
   - `00000000000000000000000000000000` is `::`
   - `0000000000000000FFFF00000100007F` is `::ffff:127.0.0.1`
   - A port of `1538` is `5432`, and a port of `18EB` is `6379`

## Generic example

Walking through a string in fixed-size pieces and joining the result:

```rust
fn decode_groups(hex: &str) -> Option<Vec<u8>> {
    if !hex.is_ascii() || hex.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::new();
    for pair in hex.as_bytes().chunks(2) {
        let text = std::str::from_utf8(pair).ok()?;
        bytes.push(u8::from_str_radix(text, 16).ok()?);
    }
    Some(bytes)
}

fn main() {
    assert_eq!(decode_groups("0AFF10"), Some(vec![0x0A, 0xFF, 0x10]));
    assert_eq!(decode_groups("0AF"), None);
}
```

Converting a `Vec` or slice to a fixed-size array uses `try_into()`, which fails if the length is wrong:

```rust
fn first_four(bytes: &[u8]) -> Option<[u8; 4]> {
    bytes.get(..4)?.try_into().ok()
}
```

## Verify

```sh
python3 -m http.server 8766 --bind ::1
```

Compare your output with:

```sh
ss -ltn
```

- Your list must include `[::1]:8766`, or however you format it.
- It should have the same listeners as `ss`.
- If you run Postgres, Redis or similar, those likely appear now even if they were missing before.

Formatting tip: IPv6 addresses with ports are normally written with brackets (`[::1]:8766`), because the address itself contains colons. `std::net::SocketAddr` does this for you when printed.

## Pitfalls

- **Slicing strings by byte index.** `&s[0..8]` panics on a non-ASCII string or a wrong length. The kernel's output is ASCII, but validate the length first, so a surprise gives you an error and not a crash.
- **Reversing all 32 characters at once.** This is the most common bug here, and it gives you an address that looks plausible. Your `::1` test catches it.
- **Treating a missing file as fatal.** Check `err.kind() == std::io::ErrorKind::NotFound` and carry on with an empty list. Other errors should still be reported.
- **Thinking a port is listed twice by mistake.** A server may really bind IPv4 and IPv6 separately. Those are two sockets. Stage 9 decides how to display them.

## Done when

- [ ] IPv6 listeners are displayed, and `::1` prints correctly
- [ ] The merged list matches `ss -ltn`
- [ ] All four test cases pass
- [ ] A missing `tcp6` file does not crash the program
- [ ] Committed
