# 01: Walking skeleton: `groves <path>` lists commits

**Parent:** spec 015 (`.wayfinder/015-commit-graph-viewer-spec.md`)

**What to build:** `cargo run -p groves-app -- <repo path>` opens a window that lists the commits reachable from HEAD, newest first, one row each with short SHA and title. It's a plain text list, not a graph yet. The full path already runs through every layer: `repo::open` reads the repo with gix and exposes it as a `CommitSource`, `core::Pane` turns the source into rows, and the egui/eframe (glow) app draws them.

**Blocked by:** None (can start immediately)

**Status:** in-progress

## Manual checks (UI behaviour, checked by hand — no automated UI tests per spec)

- [ ] `cargo run -p groves-app -- /home/limered/code/groves` opens a window listing this repo's commits, newest first, one row each with short SHA + title
- [ ] Same for the Factory repo; scrolling the list feels smooth
- [ ] `cargo run -p groves-app -- /tmp/not-a-repo` shows the error in the window instead of crashing

**Rust focus:**

**Rust focus:** cargo workspace and edition 2024, your own trait as a port (`CommitSource`), error enums with `thiserror` and `?` across crates, first `cargo test` (integration tests under `tests/`), implementing `eframe::App`.

- [ ] Workspace with crates `core`, `repo` and `app` (plus `testkit`, which the mentor writes); dependencies point inward
- [ ] `core` defines `CommitSource` and has no gix or egui dependency
- [ ] `repo::open(path)` returns an error for a path that isn't a repo; the app shows the error instead of crashing
- [ ] `core::Pane` built from a synthetic source lists rows in the expected order (red tests from the mentor)
- [ ] `repo::open` against the fixture repo yields the fixture's commits (red tests from the mentor)
- [ ] Manual: the app opens this repo and the Factory repo and scrolls smoothly
