# Stage 6 on a Mac: Map sockets to processes

## What differs on a Mac, and why

**The join is already done.** `lsof` prints each socket under its process, so there is no `/proc/<pid>/fd` walk and no inode map to build. What remains is the opposite problem: `lsof` gives you one record per process **per socket**, and you need one listener with a list of owners.

**`d` plays the part of the inode.** Forked processes that share a socket show the same `d`. Two sockets on one port, using `SO_REUSEPORT`, show different values:

```
# Forked: one socket, two owners
p56834 f3 d0x84f24cfedc0a0e54 n127.0.0.1:8767
p56857 f3 d0x84f24cfedc0a0e54 n127.0.0.1:8767

# SO_REUSEPORT: two sockets, one process
p55151 f3 d0x62a0271f722ff24b n127.0.0.1:8785
       f4 d0x68567e90bea0bf31 n127.0.0.1:8785
```

So group by socket ID, plus address and port. The value is only stable while the socket lives: I saw the same ID come back for a later, unrelated socket. Never store it between scans as an identity.

**Other users' sockets vanish.** This is the biggest difference from Linux. On Linux, a root-owned port still appears, with an unknown owner. On a Mac without root, it does not appear at all.

On this machine, PostgreSQL runs as uid 502 and listens on 5432:

```sh
ps -axo uid=,comm= | grep postgres | head -1
#   502 /Library/PostgreSQL/17/bin/postgres
/usr/sbin/lsof -nP -iTCP:5432              # no output
```

Connecting to every local port found two ports that accepted connections but were absent from `lsof`: 5432 and 8021. So "a listener with no known owner" is not a state you can see on a Mac without root. `libproc` and `listeners` missed 5432 as well.

The kernel does hand out a socket list without root, through the `net.inet.tcp.pcblist` family of sysctls, which returned about 50 to 70 KB here. I did not parse it. Apple does not document its layout as a public interface, so this guide does not rely on it.

## Your task on a Mac

Replace tasks 2 and 3:

2. Write a function that groups your parsed records into listeners, each with its list of owners.
3. Instead of the inode helper, test the grouping:
   - Two PIDs with the same socket ID give one listener with two owners
   - One PID with two socket IDs on one port gives two listeners
   - One PID on two ports gives two listeners
   - `*:8000` as IPv4 and as IPv6 gives two listeners

Keep `src/process.rs`. Stage 7 fills it in. In task 4, keep the owners type as a list, so that an empty list is still possible on any platform.

## Verify

```sh
python3 -m http.server 8765 --bind 127.0.0.1 &
SERVER=$!
echo "server PID: $SERVER"
cargo run
/usr/sbin/lsof -nP -iTCP:8765 -sTCP:LISTEN -Fp
kill $SERVER
```

Your output must show the same PID. Then the shared socket test, where the original command works unchanged:

```sh
python3 -c '
import socket, os, time
s = socket.socket(); s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("127.0.0.1", 8767)); s.listen(); os.fork(); time.sleep(60)' &
cargo run
```

Result here, one row with two owners:

```
8767   127.0.0.1    56834 +1    python3.13
```

Both processes exit on their own after 60 seconds.

Compare with a port you know another user holds, if you have one. Port 5432 was invisible here, as described above.

## Pitfalls

- **Reporting "nothing is listening" with confidence.** Absent from `lsof` only means absent from what you may see. Stages 9, 10 and 16 deal with this.
- **Thinking `sudo` only adds names.** On a Mac it adds whole rows. With `sudo lsof`, this machine also showed Postgres on `*:5432` and `launchd` (PID 1) on `127.0.0.1:8021` and `[::1]:8021`.
- **Grouping by port alone.** It merges IPv4 and IPv6 sockets.
- **Using the Linux "Things disappear" advice as is.** It still applies, but differently: `lsof` handles a process exiting during its own scan. Your next command, `ps` in Stage 7, may no longer find it.

## Changes to "Done when"

Replace "Each listener shows its PIDs, or a clear unknown", "Permission errors are skipped silently" and "The helper has tests" with:

- [ ] Each listener shows its PIDs
- [ ] The grouping has tests
- [ ] You know which ports on your machine `lsof` hides from you

## Tested on

macOS 27.2 (build 26B123), Apple M1 Max, 2026-11-02. `lsof` 4.91. `cargo run` results came from a throwaway reference build in a temp folder, not from a finished `portdash`, so your wording will differ.

`sudo lsof` was run by the owner of this Mac, and it showed both hidden ports. Not tested: running the tool itself as root. The sysctl socket list was only checked for returning data. An Intel Mac was not tested.
