# Stage 21 on a Mac: Where to go next

## What differs on a Mac, and why

The "macOS support" section is already done if you followed the Mac notes. Some other ideas here depend on Linux:

| Idea in the stage | On a Mac |
|---|---|
| `xdg-open` for open in browser | `/usr/bin/open` instead. `xdg-open` is not installed here |
| `/proc/net/udp` for UDP sockets | `lsof -nP -iUDP -F...` has the same field format |
| `pidfd` | Linux only. Your Stage 10 `shortcut:` comment remains the limit. `kqueue` with `EVFILT_PROC` may help, but I did not test it |
| Netlink `sock_diag` | Linux only. The comparable speed-up is `libproc`, at about 5 ms per scan against about 53 ms for `lsof` (Stage 2) |
| Docker awareness | More useful here, since published ports do not behave as on Linux. See Stage 19 |

One extra idea: the hidden ports problem has a possible answer in the `net.inet.tcp.pcblist` sysctls, which returned data without root here (Stage 6). They would show that a port is in use, without saying by whom. Apple does not document the layout publicly, so expect it to break. It is an interesting project, and a poor default.

## Verify

```sh
command -v open       # /usr/bin/open
command -v xdg-open   # no output, exit 1
/usr/sbin/lsof -nP -iUDP -Fpn | head -4
```

## Tested on

macOS 27.2 (build 26B123), Apple M1 Max, 2026-11-02.

Not tested: `kqueue`, parsing the sysctl data, and Docker. `open` was located but not launched, to avoid opening a browser on your screen.
