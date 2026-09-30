# 15: Local change detection

**Parent:** spec 015

**What to build:** A scheduler thread polls visible panes every 10 s, and also checks on window focus and when a grove becomes visible. Each check compares a cheap ref snapshot (mtimes of worktree HEAD files, `packed-refs` and `refs/`). When it changed, the pane does a full reload and relayout with a `ref → lane` hint map, then an atomic swap. After the swap: if scrolled to the top, follow the tips; otherwise stay anchored to the commit by SHA. Pins on unreachable commits are dropped; pins on commits pruned by the filter are kept and marked "hidden by filter".

**Blocked by:** 08

**Status:** ready-for-agent

**Rust focus:** a long-lived thread with a timer loop (your game loop again), `Instant`, carrying state (lane hints, pins, anchor) across reloads.

- [ ] Snapshot change detection against the fixture (commit, checkout in a worktree) (tests)
- [ ] Lanes stay stable across a refresh that adds commits (tests)
- [ ] Follow tips vs SHA anchor (tests)
- [ ] Pin behaviour: kept, hidden by filter, dropped (tests)
- [ ] Manual: a commit made in a terminal appears within about 10 s
