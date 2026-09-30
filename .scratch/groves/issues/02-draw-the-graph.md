# 02: Draw the graph

**Parent:** spec 015

**What to build:** The pane draws a real commit graph: temporal topological order, straight lanes (no shifting, free-slot reuse, forbidden-column check), with HEAD/main seeded leftmost. Edges and nodes are drawn with egui's painter at 22 px rows and 14 px lanes, and only the visible rows are painted. For now the commit text sits to the right of the graph; later tickets make it quiet.

**Blocked by:** 01

**Status:** ready-for-agent

**Rust focus:** the DAG as a struct of arrays: `u32` row indices into parallel `Vec`s (rapier handles, again), CSR parent lists, borrowing inside loops, `HashMap`, the egui `Painter`, and drawing only the visible rows.

- [ ] Children always appear above their parents; order is deterministic for the same input
- [ ] A linear history uses one lane; a branch and merge reuses the freed lane
- [ ] HEAD's first-parent chain sits in lane 0
- [ ] Lanes never shift sideways along a branch (tests assert lane per commit on synthetic graphs)
- [ ] Graph column at its natural width, never drawn over text
- [ ] Manual: the Factory repo scrolls without visible stutter
