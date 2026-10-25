# 02: Draw the graph

**Parent:** spec 015

**What to build:** The pane draws a real commit graph: temporal topological order, straight lanes (no shifting, free-slot reuse, forbidden-column check), with HEAD/main seeded leftmost. Edges and nodes are drawn with egui's painter at 22 px rows and 14 px lanes, and only the visible rows are painted. For now the commit text sits to the right of the graph; later tickets make it quiet.

**Blocked by:** 01

**Status:** done

**Decisions:**
- gix 0.87 → 0.89 (spec says "0.87 line"; 0.89 unifies fallible APIs on one boxed `gix::Error`, so `RepoError` wraps sites, not types — accepted, spec line now reads "0.8x line").
- When an untracked commit's first parent is already reserved in another lane, keep the existing lane and let the new commit's lane continue to the next unreserved parent; revisit after the manual graph check if the resulting merge shape looks odd.
- For now, scan `Pane::edges()` once per repaint and filter by visible-range overlap (O(E)), rather than building the spec's edge interval index. The manual thousand-commit scroll check is smooth; revisit with the 200k/frame-time measurements in ticket 21 and add the index if the budget requires it.

## Manual checks (UI behaviour, checked by hand — no automated UI tests per spec)

- [x] `cargo run -p groves-app -- /home/limered/code/groves` draws nodes + edges with the painter (22 px rows, 14 px lanes), commit text starts right of the graph column and is never drawn over
- [x] HEAD's first-parent chain runs straight down lane 0 (leftmost)
- [x] On `/home/limered/Projects/LD41`, commits reachable from HEAD appear in the same parent-before-child direction/order as `git log HEAD --graph --oneline --date-order`; refs outside HEAD's history are intentionally not included yet (ticket 03)
- [x] Longer repo for the scroll-feel check: Factory repo or `/home/limered/Projects/autodev` — scrolling a thousand-plus commits shows no visible stutter (only visible rows are painted)

**Rust focus:** the DAG as a struct of arrays: `u32` row indices into parallel `Vec`s (rapier handles, again), CSR parent lists, borrowing inside loops, `HashMap`, the egui `Painter`, and drawing only the visible rows.

- [x] Children always appear above their parents; order is deterministic for the same input
- [x] A linear history uses one lane; a branch and merge reuses the freed lane
- [x] HEAD's first-parent chain sits in lane 0
- [x] Lanes never shift sideways along a branch (tests assert lane per commit on synthetic graphs)
- [x] Graph column at its natural width, never drawn over text
- [x] Manual: the Factory repo or the tested 1,000+ commit alternative scrolls without visible stutter

## Learned

- A child-count ready queue gives a deterministic temporal topological order; row and lane indices act like the compact handles Emil knows from Rapier.
- egui `show_rows` uses row height sans spacing, and painter coordinates must be relative to the visible range; visible-range edge filtering is O(E), with the interval index deferred to ticket 21's measurement.
