# 03b: Route edges per row-gap

**Parent:** spec 015

**What to build:** The pane's edges currently draw as one long span (`child_row` → `parent_row`), so an edge that crosses several rows threads its curve *through* intermediate nodes — on the LD41 repo the develop line (`0b0ec49` → its first parent two rows below) visually fuses with the `66e17c4` → `ad72308` merge curve, and the develop branch reads as invisible. Route edges per row-gap instead: a line keeps its lane through every intermediate gap and bends into the parent's lane only across the final gap — exactly how `git log --graph` draws its `|/` bends, and how the prototype's `graphcore` edge model works (`src` lane at row i → `dst` lane at row i+1).

`core::Pane`'s edge data is unchanged: the renderer derives waypoint geometry from the existing spans. Layout was verified correct against git's own columns (ticket 03's `two_merges_of_one_spur_follow_gits_own_columns`).

**Blocked by:** 03

**Status:** done

**Rust focus:** iterator `map`/`collect` over waypoint slices, destructuring tuples, keeping math in small pure functions (rung-1 territory again: nothing new conceptually, careful geometry instead).

- [x] Every edge whose parent sits more than one row below renders so its line never threads through an intermediate commit row — measured against git's own art (`/tmp/opencode/gshape`, the fixture shape rebuilt in git, Jul–Nov 2026 hunt):
  - **First-parent edges** (the child's line continuing): vertical in the child's lane through every intermediate gap, single bend into the parent's lane across the **last** gap — the develop line's fix
  - **Later-parent edges** (a merge's second+ parents): single bend into the parent's lane across the **first** gap right below the child, then straight down the parent's lane — git draws the master merge curve departing `66e17c4` immediately (`|\` on the art line below the tip), never sprouting from the trunk next to `0b0ec49`
- [x] One-row-gap edges keep their current bend look (first or later parent) — *(superseded by the Decision amendment below: all bends now draw the new corner shape)*
- [x] Manual: on LD41 the develop line reads as one continuous line from `0b0ec49` down past `ad72308`'s row, never fusing with the master merge curve

**Decision amendment (at close, 03 Nov 2026):** the bend *shape* was repainted during review: every bend — single- or multi-gap, first or later parent — now draws one constant quarter-curve plus a straight run (vertical → corner → flat chase into the dot for first parents; flat leave → corner → vertical for later parents), so curve geometry never varies with lane distance. Supersedes the "current bend look" criterion above; Emil confirmed on LD41 that the top region still reads as git's own art with the new shape.

**Decision (recorded before code):** `is_merge` on `Edge` cannot route edges — it says the *child* is a merge, and `d1` (contract "0b0ec49") is precisely a merge whose *first*-parent edge must keep its own lane. The traffic rule lives in parent position, so `Edge.is_merge` becomes `Edge.is_first_parent` (set true only for the child's `parents[0]` edge; same size, one field swapped). This edits the ticket's "core data unchanged" promise on purpose: renderer-visible distinction, verified `two_merges…` lanes untouched. `route_edge` is a pure function in `app` (the renderer derives waypoints `(row, lane)` from spans; nothing else changes); `crates/app` gains a lib.rs + `render` module shim so integration tests can call it — the binary stays `main.rs`.

## Manual checks

- [x] `cargo run -p groves-app -- /home/limered/Projects/LD41`: the top region matches git's own `git log --graph` — the develop tip's line bends right below the tip, the master merge curve into `ad72308` stays visually separate.
- [x] Check the 4-column region around "implemented more obstacle stuff" (rows ~59–70): every line stays in its own column with single-gap bends.
- [x] `git log --graph --oneline` side-by-side for the same repo as reference.

## Learned

- Rust shadowing is legal and silent: a new inner binding simply replaces an outer name — the `enumerate` rename made `child_row: i` read the parent ordinal, scrambling every edge until the composite test named the broken row pair. Indices that mean different things get different words; names carry the meaning, the compiler doesn't.
- Parent order is DAG data, not visual order: `Edge.is_first_parent` carries the routing intent (own-lane ride vs bend-at-child), `route_edge` turns spans into corner waypoints so bends are always one gap, and the draw loop's two branches are exactly "straight run" and "one-gap corner".

## Resource trail

- Git-art forensics: `/tmp/opencode/b2` (the fixture shape in git), `/tmp/opencode/gshape`, `/tmp/opencode/c3` (plain develop tip — no staged columns). Reference runs: `git log --graph --oneline --all` on LD41 and the fixture; diagonal glyphs sit between lane columns (odd half-cell positions).
- The corner contract is load-bearing in `main.rs`'s bend branches (all offsets relative to `end_y`): re-verify it when ticket 17's `Arc<Layout>` replaces edge production.

## Evidence from the LD41 hunt (Nov 2026)

The renderer's long-span drawing was found while validating 03 on LD41 (verified against `git log --graph --oneline --all`, text columns measured per row):

- Rows 1–3: `66e17c4`(lane 0), `0b0ec49`(lane 2), `ad72308`(lane 1) — lanes match git, but the develop line's first-parent edge spans rows 1→3 and threads through `ad72308`'s row, fusing with the master merge curve (`child(row 0, lane 0) → parent(row 2, lane 1)` and `child(row 1, lane 2) → parent(row 3, lane 1)` overlap visually).
- Git's art shows the per-gap bends as `| | |/` and `| |/|` (rows 4–5) and the final trunk join as `|/` at `d954693` (art line 85).
- Lane histogram of the current layout: lane 0: 3 rows, lane 1: 57, lane 2: 8, lane 3: 5 — all four lanes genuinely occupied (git renders `| | * |` in the 28c76e5 region = 4 columns), so the layout itself is git-shaped and stays as ticket 03 delivered it.
