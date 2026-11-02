# Stage 15 on a Mac: Kill from the dashboard

The modes, popup and background stop are the same. One of the Verify steps cannot happen.

## What differs on a Mac, and why

The original asks you to try a port owned by another user. Without root, that row does not exist on a Mac (Stage 6), so there is nothing to select, and no popup to open. Task 6's "rows with no known owner" stays worth writing for shared code and for Linux, but you cannot reach it here through the real scan.

## Your task on a Mac

Keep task 6, and test it by building an `App` with a listener that has an empty owners list, since you cannot produce one for real.

## Verify

The other situations behaved as described. Results from the reference tool here:

| Step | Result |
|---|---|
| Kill key, then `Esc` | Server stayed up |
| Kill key, then `y`, on the polite server | Status showed `Graceful`, and the server stopped |
| Kill key, then `y`, on the stubborn server | `Stopping...`, keys still worked during the grace period, then `Escalated` |
| Popup open, server stopped elsewhere, then `y` | `already gone`, with no crash |
| Port owned by another user | **No row to select**: 5432 was not listed |

## Pitfalls

- **Telling users a port is free because they cannot find it.** In the TUI they see only an absent row, so keep the note about hidden ports from Stage 9 on screen.

## Changes to "Done when"

Add:

- [ ] The no-known-owner path has a test that does not depend on a real scan

## Tested on

macOS 27.2 (build 26B123), Apple M1 Max, 2026-11-02. ratatui 0.30.2. I drove the reference TUI through tmux, and the status wording above is its own.

Not tested: a very small terminal with this reference, although a separate 10 by 3 `TestBackend` test of the popup helper passed. Running as root, Terminal.app and an Intel Mac were not tested.
