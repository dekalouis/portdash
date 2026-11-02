# Stage 5 on a Mac: Add IPv6

## What differs on a Mac, and why

**One command covers both families.** There is no `/proc/net/tcp6` and nothing to decode or merge: the Stage 2 command already returns IPv4 and IPv6 together. Skip the "tricky part" about byte groups, and skip handling a missing file.

**IPv6 addresses are in brackets:** `[::1]:8772`. Remove the brackets, then let `Ipv6Addr` parse the text.

**Several forms need care.** Each of these came from a socket I bound on this Mac:

| Bound to | `t` | `n` | Note |
|---|---|---|---|
| `::1` | `IPv6` | `[::1]:8772` | The normal case |
| `::` | `IPv6` | `*:8774` | The `*` rule from Stage 4 |
| `::` with `IPV6_V6ONLY` off or on | `IPv6` | `*:8775`, `*:8776` | The two look identical, so `lsof` cannot tell you whether IPv4 clients can connect |
| `::ffff:127.0.0.1` | `IPv6` | `127.0.0.1:8783` | Printed in IPv4 form, unlike Linux |
| `fe80::1%lo0` | `IPv6` | `[fe80:1::1]:8784` | The scope is embedded in the address text |

The last row is a trap. `fe80:1::1` parses as a valid address, but it is not the address that was bound. It is rare for dev servers, so displaying it as it comes is fine, as long as you know.

## Your task on a Mac

Replace tasks 1, 3, 4, 5 and 6:

1. Teach your address function about brackets.
3. Use `IpAddr` throughout. No decoder choice is needed.
4. Sort the combined list by port.
5. Skip this one, since there is no second file.
6. Test with these cases:
   - `[::1]:8766` gives `::1` and port `8766`
   - `*:8000` with family `IPv6` gives `::`
   - `127.0.0.1:8783` with family `IPv6` still parses
   - `[::1]:ZZZZ` returns an error

## Generic example

Cutting at the last separator, then removing optional brackets:

```rust
/// Splits a theatre seat label such as "[Main: Hall]:12" or "Balcony:7".
fn split_seat(label: &str) -> Option<(&str, u32)> {
    // The room name may contain colons, so cut at the last one.
    let (room, seat) = label.rsplit_once(':')?;
    let room = room
        .strip_prefix('[')
        .and_then(|r| r.strip_suffix(']'))
        .unwrap_or(room);
    Some((room, seat.parse().ok()?))
}

fn main() {
    assert_eq!(split_seat("[Main: Hall]:12"), Some(("Main: Hall", 12)));
    assert_eq!(split_seat("Balcony:7"), Some(("Balcony", 7)));
    assert_eq!(split_seat("Balcony:x"), None);
    assert_eq!(split_seat(""), None);
    println!("ok");
}
```

## Verify

```sh
python3 -m http.server 8766 --bind ::1 &
SERVER=$!
cargo run
/usr/sbin/lsof -nP -iTCP -sTCP:LISTEN
kill $SERVER
```

Results here:

```
8766   ::1    56761    python3.13
python3.1 56761 you  3u  IPv6 0x84f24cfedc0a0e54  0t0  TCP [::1]:8766 (LISTEN)
```

The row was gone after `kill`. With a few servers running, `-a -i4` and `-a -i6` returned 12 and 7 sockets here, which is how you can check each family against your list.

## Pitfalls

- **`split(':')` on IPv6.** Use `rsplit_once(':')`, as the example does.
- **Assuming `*` on IPv6 means both families.** It depends on `IPV6_V6ONLY`, which `lsof` does not show. Do not claim more than you know in your UI.
- **Thinking `[::1]` and `127.0.0.1` on one port is a bug.** Here, a Homebrew service held `127.0.0.1:6379` and `[::1]:6379` as two sockets.

## Changes to "Done when"

Replace the second to fourth items with:

- [ ] The merged list matches `lsof -nP -iTCP -sTCP:LISTEN`
- [ ] All four test cases above pass

## Tested on

macOS 27.2 (build 26B123), Apple M1 Max, 2026-11-02. `lsof` 4.91. `cargo run` results came from a throwaway reference build in a temp folder, not from a finished `portdash`, so your wording will differ.

Not tested: an Intel Mac, or a Mac with IPv6 turned off.
