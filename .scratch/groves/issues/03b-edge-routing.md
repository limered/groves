# 03b: Route edges per row-gap

**Parent:** spec 015

**What to build:** The pane's edges currently draw as one long span (`child_row` → `parent_row`), so an edge that crosses several rows threads its curve *through* intermediate nodes — on the LD41 repo the develop line (`0b0ec49` → its first parent two rows below) visually fuses with the `66e17c4` → `ad72308` merge curve, and the develop branch reads as invisible. Route edges per row-gap instead: a line keeps its lane through every intermediate gap and bends into the parent's lane only across the final gap — exactly how `git log --graph` draws its `|/` bends, and how the prototype's `graphcore` edge model works (`src` lane at row i → `dst` lane at row i+1).

`core::Pane`'s edge data is unchanged: the renderer derives waypoint geometry from the existing spans. Layout was verified correct against git's own columns (ticket 03's `two_merges_of_one_spur_follow_gits_own_columns`).

**Blocked by:** 03

**Status:** ready-for-agent

**Rust focus:** iterator `map`/`collect` over waypoint slices, destructuring tuples, keeping math in small pure functions (rung-1 territory again: nothing new conceptually, careful geometry instead).

- [ ] Every edge whose parent sits more than one row below renders as: vertical in the child's lane through the intermediate gaps, single bend into the parent's lane across the last gap
- [ ] One-row-gap edges keep their current bend look
- [ ] Manual: on LD41 the develop line reads as one continuous line from `0b0ec49` down past `ad72308`'s row, never fusing with the master merge curve

## Manual checks

- [ ] `cargo run -p groves-app -- /home/limered/Projects/LD41`: the top region matches git's own `git log --graph` — the develop tip's line bends right below the tip, the master merge curve into `ad72308` stays visually separate.
- [ ] Check the 4-column region around "implemented more obstacle stuff" (rows ~59–70): every line stays in its own column with single-gap bends.
- [ ] `git log --graph --oneline` side-by-side for the same repo as reference.

## Evidence from the LD41 hunt ( Nov 2026)

The renderer's long-span drawing was found while validating 03 on LD41 (verified against `git log --graph --oneline --all`, text columns measured per row):

- Rows 1–3: `66e17c4`(lane 0), `0b0ec49`(lane 2), `ad72308`(lane 1) — lanes match git, but the develop line's first-parent edge spans rows 1→3 and threads through `ad72308`'s row, fusing with the master merge curve (`child(row 0, lane 0) → parent(row 2, lane 1)` and `child(row 1, lane 2) → parent(row 3, lane 1)` overlap visually).
- Git's art shows the per-gap bends as `| | |/` and `| |/|` (rows 4–5) and the final trunk join as `|/` at `d954693` (art line 85).
- Lane histogram of the current layout: lane 0: 3 rows, lane 1: 57, lane 2: 8, lane 3: 5 — all four lanes genuinely occupied (git renders `| | * |` in the 28c76e5 region = 4 columns), so the layout itself is git-shaped and stays as ticket 03 delivered it.
