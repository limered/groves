---
id: 001
title: Which Rust UI stack can draw a dense graph fast and look GitKraken-nice?
labels: [wayfinder:research]
parent: 000
blocked_by: []
assignee: agent
status: closed
---

## Question

Compare egui/eframe, iced, slint, and a custom wgpu (+ e.g. vello/glyphon) renderer for: virtualized rendering of 200k-row graph with lanes/edges/labels at 60 fps, idle CPU ~0, RAM footprint, Windows + Linux (X11/Wayland) support, custom-drawing APIs, text quality, theming/animation for a polished look, keyboard handling, multi-pane layouts, maturity. Recommend a shortlist of 1–2 for the prototype.

## Resolution

Shortlist: **egui/eframe 0.36 (+ egui_tiles)** first, then **iced 0.14 (canvas/shader + PaneGrid)**.
egui has the strongest custom painting and row virtualization and the least effort. Its weak spots are text quality and its default look. iced has better text (cosmic-text), an animation API and built-in tiling, but its releases have slowed (last one Dec 2025).
Slint looks polished but can't draw the edge-heavy graph canvas well, and it is GPL or commercial. A fully custom wgpu/vello stack costs too much. Keep it only as a fallback for the graph panel alone, inside the chosen framework.
Next: prototype both on a synthetic 200k-commit graph, measure fps, idle CPU and RSS, and have a human judge the look.
Details: [research/ui-framework.md](research/ui-framework.md)
