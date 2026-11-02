# Stage 4 on a Mac: Addresses and listeners

## What differs on a Mac, and why

**No byte swapping.** `lsof` already prints `127.0.0.1`, so the endianness lesson does not apply to this tool on a Mac. It is still worth reading, and Apple Silicon is little-endian too, but you will not call `to_ne_bytes` here. `Ipv4Addr` parses the text with `str::parse`.

**No state column.** `-sTCP:LISTEN` asks `lsof` to filter, so there is nothing to compare with `0A`.

**The wildcard is a star.** A server on all interfaces does not show as `0.0.0.0`:

```
tIPv4
n*:8773
```

The address alone cannot tell you whether that means `0.0.0.0` or `::`, because `lsof` prints `*` for both. That is why the command in Stage 2 asks for `t`:

| `t` | `n` | Bind address |
|---|---|---|
| `IPv4` | `*:8773` | `0.0.0.0` |
| `IPv6` | `*:8774` | `::` |

This matters for the same reason as in the original: a `*` row may be reachable from your Wi-Fi.

**You will see Mac services.** Here, `ControlCenter` listened on `*:5000` and `*:7000`. These are not a bug. Port 5000 is a common dev server default, so "address already in use" on it is a classic Mac surprise, and your tool now explains it.

## Your task on a Mac

Replace tasks 1, 3 and 6:

1. Add a function that turns the address text and the family into an `IpAddr`, including the `*` rule.
3. Keep this step only as a check: every row you print came from a LISTEN-filtered scan.
6. Test with these cases:
   - `127.0.0.1` is `127.0.0.1`
   - `*` with family `IPv4` is `0.0.0.0`
   - `not-an-address` returns an error

Tasks 2, 4 and 5 stay the same. Skip the `3500007F` case, since `127.0.0.53` does not appear on a Mac.

## Verify

Start a server, then compare with `lsof`, which replaces `ss`:

```sh
python3 -m http.server 8765 --bind 127.0.0.1 &
SERVER=$!
cargo run
/usr/sbin/lsof -nP -iTCP -sTCP:LISTEN -a -i4
kill $SERVER
```

Results here:

```
8765   127.0.0.1    56760    python3.13
python3.1 56760 you  3u  IPv4 0x62a0271f722ff24b  0t0  TCP 127.0.0.1:8765 (LISTEN)
```

After `kill`, the row was gone. `lsof` shortens the name to `python3.1` in this table view. The `-F` form gave the full `python3.13`.

## Pitfalls

- **Dropping `t` from the field list.** `*:8000` becomes a guess, and your list says `0.0.0.0` for an IPv6 server.
- **An IPv6-family socket that prints an IPv4 address.** A socket bound to `::ffff:127.0.0.1` printed `t` as `IPv6` and `n` as `127.0.0.1:8783`. Parse the text you got, and do not assume the family matches its shape.
- **Comparing to `ss`.** It does not exist here.

## Changes to "Done when"

Replace "Your IPv4 list matches `ss -4ltn`" with:

- [ ] Your IPv4 list matches `lsof -nP -iTCP -sTCP:LISTEN -a -i4`
- [ ] `*` with an IPv4 family prints as `0.0.0.0`

## Tested on

macOS 27.2 (build 26B123), Apple M1 Max, 2026-11-02. `lsof` 4.91. `cargo run` results came from a throwaway reference build in a temp folder, not from a finished `portdash`, so your wording will differ.

Not tested: an Intel Mac.
