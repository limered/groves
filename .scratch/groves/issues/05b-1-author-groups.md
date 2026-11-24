# 05b-1: Author groups (folding mechanism)

**Parent:** spec 015 (stories 75–80, "Author groups (row folding)")

**What to build:** Linear runs of 3 or more commits by the same author in one lane are folded into one **Author group** row, collapsed by default and drawn as a ring with `×N`. A run counts only if each commit is the only child of the next and that commit is its first parent. A fork, a merge, a pill, a worktree HEAD marker, a tag or an age threshold (3h/1d/1w/3w) ends a group. Clicking a group expands it: a light border wraps its commits, and a **Collapse marker** at the top folds it again. `g` collapses or expands all groups. Lanes never change when a group expands or collapses: folding is a pure step in `core` that runs after the lane layout and maps visible rows to commit ranges. The clicked row stays in place on screen.

**Pin rules** were split out to 05b-2 (they need the pin set from 07). The folding step still takes the pin set as input from the start: `fold` receives `&Pins` — an empty set in this ticket — so 05b-2 plugs the real set in without a signature change. Age thresholds are spec constants, not 06's output; 06 only draws lines at the same boundaries.

**Expanded state:** kept in memory only, keyed by the SHA of the group's oldest commit, and dropped silently for groups that no longer exist.

**Blocked by:** 05

**Status:** ready-for-agent

**Decisions (recorded before code)**

- **2026-11-24 — split from 05b:** the pin interaction rules moved to 05b-2 (blocked by 07) so the mechanism lands before its consumers (06 age lines, 07 pins) without building against features that don't exist yet. The full move of 05b behind 06+07 was rejected: it would have flipped the rework onto 06's line placement and 07's pin semantics, which consume the row model.

**Test setup note:** `testkit::SyntheticSource` has no authors yet. The mentor adds per-commit authors (and committer dates for the age split) before writing the red tests.

**Rust focus:** a pure function over the layout (`fn fold(&Layout, &Expanded, &Pins) -> Rows`), `Range<usize>` and run detection with iterators (`windows`, `chunk_by`), keying state by a stable id (`HashSet<Oid>`), keeping an index mapping visible rows ↔ commits.

- [ ] A same-author linear run of ≥ 3 commits folds into one row. A run of 2 doesn't fold (tests via `core::Pane`).
- [ ] Each break rule ends a group: author change, fork, merge, pill, HEAD marker, tag, age threshold.
- [ ] Lanes are identical whether a group is expanded or collapsed.
- [ ] The expanded state survives a refresh that adds commits on top. A group that disappears drops its state.
- [ ] `g` toggles all groups.
- [ ] Manual: the ring with `×N` shows on collapsed groups, and hover shows `Emil · 5 commits · 2h–6h ago` plus the newest title.
- [ ] Manual: click expands, the light border and collapse marker show, and the clicked row doesn't jump.
