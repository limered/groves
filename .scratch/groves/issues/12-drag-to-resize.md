# 12: Drag to resize

**Parent:** spec 015

**What to build:** Pane edges can be dragged to change split ratios, and the graph column can be dragged wider than its natural width. Neither ever draws over text. The ratios are persisted through the pane tree.

**Blocked by:** 09

**Status:** ready-for-agent

**Rust focus:** geometry helpers, egui drag interaction (`Sense::drag`), clamping, mapping a screen position back to a tree node.

- [ ] Ratio update from a drag delta, with clamping (tests)
- [ ] Manual: resizing feels smooth; the graph column never overlaps text
