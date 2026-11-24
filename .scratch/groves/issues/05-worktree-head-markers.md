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
- **Clock and pixels (added mid-ticket, announced via the AGENTS.md rules):** `Pane::load` now takes `GraphOptions { now, stale_after_days }` — the caller stamps the clock, core owns the window; the 22/14 px constants and `graph_width_px` moved to `app::render` (the renderer owns pixel dimensions). Test call sites were updated in the same commits, and the staleness tests pin that both values come from the options.
- **Label placement:** the marker's name label sits in the existing pill area — trailing right of the pill, at the left margin on rows without a pill (detached heads). Never inside the pill (pill and worktree label are different entities with different click semantics), never a dedicated column (story 25). Label looks secondary to the pill; long names clip and later get the ticket-20 treatment (truncate, full name on hover).

**Rust focus:** `Option` combinators (`map`, `and_then`, `unwrap_or`), gix's worktree API, a small cycling state.

- [x] `repo` lists all worktrees with their HEADs, including detached ones (fixture with 3 worktrees) — `repo/tests/worktrees.rs::snapshot_lists_all_worktree_heads_main_first_with_names`
- [x] A commit reachable only from a secondary worktree's HEAD is drawn — `worktrees.rs::history_reachable_only_from_secondary_worktree_heads_is_drawn`
- [x] Opening a linked worktree path yields the same repository as opening the main one — `worktrees.rs::opening_a_linked_worktree_folder_yields_the_same_repository`
- [x] Jump-to-HEAD order: main first, then the others, then wraps around — `core/tests/head_markers.rs::head_jump_rows_*` + `app/tests/head_jump.rs::jump_cycles_main_then_others_then_wraps`
- [ ] Manual: markers, hover names and click-toggle work

## Manual checks

Build a playground repo with two extra worktrees:

```
git init -q --bare origin.git
git init -q -b main wt-playground && cd wt-playground
echo x > root.txt && git add . && git commit -qm Root
git remote add origin ../origin.git
git push -q -u origin main
git worktree add ../side-checkout -b side
(cd ../side-checkout && echo s > side.txt && git add . && git commit -qm "Side only work")
git push -q origin side
git worktree add --detach ../detached-checkout HEAD
(cd ../detached-checkout && echo d > stray.txt && git add . && git commit -qm "Detached only work")
git fetch -q origin
```

Then `cargo run -p groves-app -- wt-playground`:

1. Every worktree's HEAD commit carries the marker ring: `wt-playground` (main) and `side-checkout` share the top row, `detached-checkout`'s short SHA sits on the stray commit.
2. Only the main worktree's name is visible by default; hovering a marker shows the other names; moving the mouse away hides them again.
3. Clicking a marker toggles its label: a secondary name stays after the click; clicking the main marker hides its name; clicking again restores it.
4. `h` scrolls the main worktree's row into view (centred); pressing it again cycles to the next worktree's row; pressing it repeatedly wraps back to main. In a repo with one worktree, `h` always lands on that row.
5. `cargo run -p groves-app -- ../side-checkout` draws the identical graph as opening `wt-playground` itself.
6. The main pill reads `🔘main`: `main` has an upstream (`origin/main`) that is in sync, so the pill gets the synced style and icon. The `side` pill has no icon — it has no upstream. (This state was missing from the original recipe; the synced style was never exercised by the playground.)
