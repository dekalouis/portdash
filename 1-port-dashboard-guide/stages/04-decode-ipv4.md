# Stage 4: Decode IPv4 and find listeners

On a Mac? Also read [this stage's Mac notes](04-decode-ipv4-mac-support.md).

**Goal:** print only listening sockets, with readable addresses, matching what `ss` reports.

## Background

### Why the address looks backwards

Look at `0100007F`. This is `127.0.0.1`, the loopback address, but the digits appear reversed: `01 00 00 7F` is `1 0 0 127`.

The cause is **endianness**, the order a CPU stores the bytes of a number. An IP address has a fixed byte order (127, 0, 0, 1). The kernel treats those four bytes as one 32-bit integer and prints that integer in hex. Your x86 CPU is **little-endian**, meaning it stores the least significant byte first, so the bytes print in reverse.

So you need to:

1. Parse the hex into a `u32`, which recovers the integer the kernel printed.
2. Get that integer's bytes back out in the order the CPU holds them.
3. Build an address from those four bytes.

Rust's integers have three methods for step 2: `to_be_bytes`, `to_le_bytes` and `to_ne_bytes` (big, little and native endian). Think about which one correctly describes "the order this CPU stores it". Two of them give the right result on your machine, but only one is right on every machine.

The standard library has `std::net::Ipv4Addr`, which can be built from `[u8; 4]` and prints correctly. There is no need to format it yourself.

### Socket states

The `st` column is a TCP state. The ones you will see most:

| Hex | State | Meaning |
|---|---|---|
| `01` | ESTABLISHED | An active connection |
| `06` | TIME_WAIT | A closed connection, lingering |
| `0A` | LISTEN | Waiting for connections |

A dashboard of servers only cares about `0A`. Remember it is hex, so `0A` is 10.

### Bind addresses matter

The address a server listens on says who can reach it:

- `127.0.0.1`: only this machine.
- `0.0.0.0`: every network interface, so possibly other machines on your Wi-Fi too.

That difference matters to a developer. Your tool will show it clearly, and later could warn about it.

## Your task

1. Add a function that turns the 8-character hex address into an `Ipv4Addr`, and change your struct to hold the real type instead of a string.
2. Parse a whole file's contents into a list. The input should be `&str`, so you can test it with a string.
3. Keep only LISTEN sockets.
4. Print them as `address:port`, sorted by port.
5. Decide what happens when one line fails to parse. The kernel's format is stable, so a failure means a bug or a format you did not expect. A reasonable answer is to skip that line and warn on stderr. It is worse to lose the entire list over one odd line.
6. Test with these cases:
   - `0100007F` is `127.0.0.1`
   - `3500007F` is `127.0.0.53`
   - `00000000` is `0.0.0.0`
   - A multi-line string with one LISTEN and one ESTABLISHED line returns only the listener

## Generic example

A device that reports its 4-byte ID in little-endian order:

```rust
fn device_id_bytes(hex: &str) -> Result<[u8; 4], std::num::ParseIntError> {
    let raw = u32::from_str_radix(hex, 16)?;
    Ok(raw.to_le_bytes())
}

fn main() {
    // The device prints "DDCCBBAA". Its real ID is AA BB CC DD.
    assert_eq!(device_id_bytes("DDCCBBAA").unwrap(), [0xAA, 0xBB, 0xCC, 0xDD]);
}
```

Filtering and sorting with iterators:

```rust
struct Order {
    id: u32,
    shipped: bool,
}

fn unshipped_ids(orders: &[Order]) -> Vec<u32> {
    let mut ids: Vec<u32> = orders
        .iter()
        .filter(|order| !order.shipped)
        .map(|order| order.id)
        .collect();
    ids.sort();
    ids
}
```

## Verify

Start a server on a known port in a second terminal:

```sh
python3 -m http.server 8765 --bind 127.0.0.1
```

Then compare:

```sh
cargo run
ss -4ltn
```

- Your list must include `127.0.0.1:8765`.
- It must contain the same IPv4 listeners as `ss`. (`-4` means IPv4, `-l` listening, `-t` TCP, `-n` numeric.)
- Stop the server, run again, and it should be gone.

## Pitfalls

- **Comparing state with `10` in the wrong base.** If you kept the state as a string, compare with `"0A"`. If you parsed it as hex, compare with `0x0A`.
- **Doing the byte swap on the string.** It looks simple, but it is more code, and it does not express what is really happening. Use the integer.
- **Seeing `127.0.0.53`.** That is not a bug. It is `systemd-resolved`, the local DNS service.
- **Duplicates between runs.** The table changes constantly. Compare what you expect, not exact output.

## Done when

- [ ] Addresses print in normal form, such as `127.0.0.1`
- [ ] Only listeners are shown, sorted by port
- [ ] Your IPv4 list matches `ss -4ltn`
- [ ] Your test server appears and disappears
- [ ] Committed
