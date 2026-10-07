# 02: Draw the graph

**Parent:** spec 015

**What to build:** The pane draws a real commit graph: temporal topological order, straight lanes (no shifting, free-slot reuse, forbidden-column check), with HEAD/main seeded leftmost. Edges and nodes are drawn with egui's painter at 22 px rows and 14 px lanes, and only the visible rows are painted. For now the commit text sits to the right of the graph; later tickets make it quiet.

**Blocked by:** 01

**Status:** in-progress

## Manual checks (UI behaviour, checked by hand — no automated UI tests per spec)

- [ ] `cargo run -p groves-app -- /home/limered/code/groves` draws nodes + edges with the painter (22 px rows, 14 px lanes), commit text starts right of the graph column and is never drawn over
- [ ] HEAD's first-parent chain runs straight down lane 0 (leftmost)
- [ ] Longer repo for the scroll-feel check: Factory repo or `/home/limered/Projects/autodev` — scrolling a thousand-plus commits shows no visible stutter (only visible rows are painted)

**Rust focus:** the DAG as a struct of arrays: `u32` row indices into parallel `Vec`s (rapier handles, again), CSR parent lists, borrowing inside loops, `HashMap`, the egui `Painter`, and drawing only the visible rows.

- [ ] Children always appear above their parents; order is deterministic for the same input
- [ ] A linear history uses one lane; a branch and merge reuses the freed lane
- [ ] HEAD's first-parent chain sits in lane 0
- [ ] Lanes never shift sideways along a branch (tests assert lane per commit on synthetic graphs)
- [ ] Graph column at its natural width, never drawn over text
- [ ] Manual: the Factory repo scrolls without visible stutter
