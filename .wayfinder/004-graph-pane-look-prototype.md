---
id: 004
title: What should a graph pane look and feel like?
labels: [wayfinder:prototype]
parent: 000
blocked_by: [001, 003]
assignee: emil.bohleber
status: closed
---

## Question

With the shortlisted UI stack(s), build a rough pane rendering the Factory repo's graph (lanes, ref pills, commit rows, detail panel stub) so the human can judge look, density, scroll feel and resource use — and pick the UI stack.


## Resolution

**UI stack: egui/eframe 0.36 using the OpenGL (`glow`) renderer.** Iced and wgpu are dropped.

Prototype assets (throwaway code): `prototype/` contains `graphcore` (synthetic generator, a git-log loader and a straight-lane layout), `proto-egui` and `proto-iced`. Both prototypes draw the same graph. They were built with Rust 1.98.1 (GNU toolchain) plus WinLibs mingw, because no MSVC build tools are installed.

Measured on Windows (2026-09-24):

| Setup | Memory (RSS) | Idle CPU |
|---|---|---|
| egui on wgpu, empty repo | 434 MB | ~1% |
| egui on wgpu, synthetic 200k commits | 554 MB | 4% |
| egui on OpenGL, empty repo | 151 MB | 0.3% |
| egui on OpenGL, synthetic 200k commits | 224 MB | 0.2% |
| egui on OpenGL, Factory repo | 158 MB | 0.2% |
| iced on wgpu (DX12), synthetic 200k commits | 308 MB | 0.5% |
| iced on its CPU renderer (tiny-skia), synthetic 200k commits | 132 MB | 40% |

- Most of the memory goes to the wgpu backend itself, not to the data.
- Laying out 200k synthetic commits takes about 31 ms, with 18 lanes at most.
- egui on wgpu once idled at 20% CPU with the Factory repo. This was not reproduced or explained.

Pane layout baseline (the human's judgement; colours, fonts and polish stay in the theming fog):

- **Density:** 22 px rows and 14 px lanes are fine.
- **Columns, left to right:**
  - Pills first. Their text may be shortened, and hovering shows the full name.
  - Then the graph column. It can be resized to the right and must never draw into the text. The prototype's graph overlapped the text.
  - Then the message.
  - Author, date and SHA are drawn right-aligned on top of the message, and the message is cut off underneath them.
- **Detail panel:** at the bottom of the pane and can be hidden.
- **Pills:** the styles are fine (filled for local, outline for remote, the synced mark, rectangles for tags). **Only one pill is shown per commit by default**, and every other ref goes under a `+N` dropdown.
  - Which ref gets the pill: HEAD's branch first, then local branches, then remote branches.
  - This refines [How are thousands of tags and remote branches shown without burying the graph?](005-ref-display-policy-grilling.md), which allowed several visible pills.