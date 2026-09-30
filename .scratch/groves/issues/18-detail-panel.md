# 18: Detail panel (synchronous)

**Parent:** spec 015

**What to build:** `d` toggles a bottom detail panel, hidden at start, that describes the **Pin set**. It lists the union of files changed by the pinned commits as a flat list, with A/M/D/R and coloured SHA chips of the pinned commits that touched each file. Merges are diffed against their first parent, with rename tracking off. Below the files come the full messages of the pinned commits, newest first, each headed by its SHA chip, author and date. An empty pin set shows "Click commits to pin them". In this ticket the diffs may still run synchronously; 19 moves them off the UI thread.

**Blocked by:** 07

**Status:** ready-for-agent

**Rust focus:** gix tree diffs, grouping with `BTreeMap`, a changed-path type in `repo` that `core` can consume through the port.

- [ ] Changed-path lists for normal, merge (first parent) and root commits from the fixture (tests)
- [ ] Pin-set union with the correct chips per file (tests via `core::Pane`)
- [ ] Messages newest first (tests)
- [ ] Manual: panel toggles, empty text shows
