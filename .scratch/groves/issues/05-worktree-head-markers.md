# 05: Worktree HEAD markers

**Parent:** spec 015

**What to build:** Every worktree of the repository gets the same **Worktree HEAD marker** on its HEAD commit, and every worktree HEAD joins the ref set. By default only the main worktree's name is shown. Other names show on hover, and clicking a marker toggles its name. A detached HEAD shows its short SHA. Jump-to-HEAD scrolls to the main worktree first and cycles through the others on repeated presses. Opening a worktree folder opens its repository.

**Blocked by:** 03

**Status:** in-progress

## Decisions (recorded before code)

- **Marker names:** a worktree's label is its git worktree name — the main worktree is named after its repo folder (e.g. `Factory`), linked worktrees after their git worktree id (e.g. `a2m1706-repro`). A worktree whose HEAD is detached shows the target commit's short SHA instead (spec story 34). Chooses 006's "worktree name on hover" over 005's branch-name wording (009 supersedes 005 here); avoids duplicating the branch pill on the same row.
- **Snapshot shape:** worktree HEADs stay `RefKind::WorktreeHead` refs whose `name` is the label above. The **first** `WorktreeHead` ref in the snapshot is the main worktree (gix's `worktrees_including_main()` already yields main first; `repo::open` keeps that order). This renames the main head ref from `"HEAD"` — `repo/tests/ref_snapshot.rs` is updated accordingly as an announced contract change.
- **Jump key:** plain `h` (spec's single-key list has no jump key; `h` is free, plain `h` ≠ `Alt+h` pane focus).
- **New facades:** `core::HeadMarker { name: String, is_main: bool }` on `Row.head_markers: Vec<HeadMarker>`; `core::Pane::head_jump_rows() -> Vec<usize>` (row indices in worktree order: main first, then the others, duplicate rows skipped); `app::head_jump::next_head_jump(cursor: Option<usize>, targets: &[usize]) -> Option<(usize, usize)>` (next cursor index + row to scroll to, wraps around).

**Rust focus:** `Option` combinators (`map`, `and_then`, `unwrap_or`), gix's worktree API, a small cycling state.

- [ ] `repo` lists all worktrees with their HEADs, including detached ones (fixture with 3 worktrees)
- [ ] A commit reachable only from a secondary worktree's HEAD is drawn
- [ ] Opening a linked worktree path yields the same repository as opening the main one
- [ ] Jump-to-HEAD order: main first, then the others, then wraps around
- [ ] Manual: markers, hover names and click-toggle work

## Manual checks

Build a playground repo with two extra worktrees:

```
git init -q -b main wt-playground && cd wt-playground
echo x > root.txt && git add . && git commit -qm Root
git worktree add ../side-checkout -b side
(cd ../side-checkout && echo s > side.txt && git add . && git commit -qm "Side only work")
git worktree add --detach ../detached-checkout HEAD
(cd ../detached-checkout && echo d > stray.txt && git add . && git commit -qm "Detached only work")
```

Then `cargo run -p groves-app -- wt-playground`:

1. Every worktree's HEAD commit carries the marker ring: `wt-playground` (main) and `side-checkout` share the top row, `detached-checkout`'s short SHA sits on the stray commit.
2. Only the main worktree's name is visible by default; hovering a marker shows the other names; moving the mouse away hides them again.
3. Clicking a marker toggles its label: a secondary name stays after the click; clicking the main marker hides its name; clicking again restores it.
4. `h` scrolls the main worktree's row into view (centred); pressing it again cycles to the next worktree's row; pressing it repeatedly wraps back to main. In a repo with one worktree, `h` always lands on that row.
5. `cargo run -p groves-app -- ../side-checkout` draws the identical graph as opening `wt-playground` itself.
