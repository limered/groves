# Research: Rust UI stack for the commit-graph viewer

Ticket: [001](../001-ui-framework-research.md) · Date: 2026-09-24

## What was checked

- **Checked this session (GitHub release pages, 2026-09-24):**
  - egui/eframe **0.36.2** (released 2026-09-08). 0.34 moved API from `Context` to `Ui`. 0.35 added inspection, `egui_mcp`, classes (style hooks) and better IME. 0.36 syncs window decorations with the app theme. MSRV 1.95.
  - iced **0.14.0** (released 2025-12-07). Still the latest release, so nothing has shipped in about 9 months. 0.14 added reactive rendering, an animation API, a `table` widget, `sensor`, primitive culling in column/row, `canvas::Cache::draw_with_bounds`, x11/wayland feature flags, a `shader::Pipeline` trait and comet devtools. `PaneGrid` already existed.
  - Slint **1.18.1** (released 2026-09-21). Ships about monthly with patch releases. Licensing is GPLv3, royalty-free, or commercial.
  - vello **0.10.0** (released 2026-08-14). Still 0.x with no API-stability promise. The sparse-strips "Vello CPU/Hybrid" line is at 0.2.0, and Glifo (text) is at 0.3.0.
- **Not re-checked this session (from general knowledge, verify during the prototype):**
  - Current glyphon and cosmic-text versions.
  - Slint's Skia/FemtoVG renderer details.
  - Whether Slint's `Path` element is fast enough for thousands of edges.

## Comparison

| Criterion | egui / eframe | iced | Slint | Custom wgpu + vello/glyphon + winit |
|---|---|---|---|---|
| Rendering model | Immediate mode. The whole UI is rebuilt each frame, drawn with a tessellated mesh on wgpu or glow | Elm-style retained mode. Reactive redraw since 0.14. wgpu, with tiny-skia fallback | Declarative `.slint` DSL, compiled. Renderers: FemtoVG, Skia, software | Whatever you build: a retained scene, redrawn on demand |
| 200k-row virtualized graph at 60 fps | Good. `ScrollArea::show_rows` / `show_viewport` only draws visible rows. `Painter` handles lines, beziers and text. A few thousand shapes per frame is cheap | Good if you use `canvas` with `Cache` (per-viewport geometry) or a custom `shader` widget. The built-in `scrollable` is not virtualized, so you drive it yourself | Weakest fit. `ListView` virtualizes rows, but edges crossing rows need `Path` or pre-rendered images. There is no low-level painter; the escape hatch is rendering to a texture yourself | Best ceiling. Only the visible rows are instanced, and edges can be a GPU line/bezier pipeline or a vello scene |
| Idle CPU | Near 0. eframe repaints only on input or `request_repaint`. Hover/animation causes short bursts. Every repaint rebuilds the whole UI | Near 0 since 0.14's reactive rendering | Near 0, property-driven redraw | Near 0 if the event loop uses `ControlFlow::Wait` (your responsibility) |
| RAM / binary | Small: roughly 30–60 MB RSS with wgpu, less with glow (estimate) | Moderate. wgpu + cosmic-text is a similar ballpark (estimate) | Small. Designed for embedded; the software/FemtoVG renderer is lean | Depends on you. wgpu alone is the largest part |
| Windows + Linux (X11/Wayland) | Yes, through winit. Mature | Yes. winit, with x11/wayland feature flags | Yes. winit or Qt backends | Yes, through winit |
| Custom-drawing API | Excellent: `Painter`, `Shape`, `Mesh`, plus a wgpu paint callback | Good: `canvas` (path API, caches) and `shader` widgets for raw wgpu | Limited: `Path` and images, or an underlay/texture via the wgpu interop feature | Complete, but you write all of it |
| Text quality | Acceptable but weakest. Its own atlas rasterizer with limited shaping/hinting and no complex shaping. 0.34.2 fixed layout issues. Fine for a Latin UI; slightly softer than native | Good: cosmic-text shaping and fallback. `text::Shaping::Auto` since 0.14 | Good: Skia or FemtoVG with system fonts | Best possible: glyphon/cosmic-text or parley+vello. Needs integration work |
| Theming / animation for a polished look | Theming via `Style`/`Visuals` and classes since 0.35. Tends to look "egui-ish" unless heavily restyled. Animation limited to `animate_value_*` helpers | Palette/styling per widget, plus a first-class animation API since 0.14. Makes a modern look easier | Strongest for polish: designer-friendly DSL, states/transitions, Fluent/Material/Cupertino styles, live preview | Unlimited, at full cost |
| Keyboard handling | Easy: global input state queried every frame; focus handling OK | Subscriptions and `keyboard` events. Focus operations (`is_focused`, `unfocus`) exist but are verbose | `FocusScope`/key handlers in the DSL. Global shortcuts are awkward | Raw winit events; you build shortcuts and focus |
| Multi-pane / tiling | `egui_tiles` (Rerun) or `egui_dock` give drag-to-tile with serializable layouts. Hyprland-style auto-layout would be custom logic on top | `PaneGrid` is built in: split, resize, drag, maximize. The closest to Hyprland tiling out of the box | No tiling widget; nested layouts you write in the DSL | Build it yourself (it's a tree of rects, which is fairly easy) |
| Maturity / momentum | High. Frequent releases (0.33→0.36 in 2026), Rerun-sponsored, large user base. Breaking changes each minor | Medium-high: popular, 0.x. Release cadence is slow (0.14 is from Dec 2025) | High: 1.x with stable API, commercial company behind it, monthly releases. GPL/commercial licensing | Parts are mature (wgpu, winit). vello is still 0.x/unstable. The app shell is yours to maintain |
| Dev effort for this app | Lowest | Low-medium | Medium. The graph canvas fights the framework | Highest (weeks spent on basics: widgets, text input, scrolling) |

## Assessment

- **The graph canvas is the deciding factor.** Every option handles about 100 visible rows × (dot + lane lines + label) easily. What matters is how directly you can draw them. egui (`Painter` + `show_rows`) and iced (`canvas`/`shader` + manual virtualization) both do it natively. Slint does not.
- **Polish vs. effort:**
  - egui is the fastest route to a working, dense, keyboard-first tool. Its text and "look" are the known weak spots, and reaching a GitKraken feel means custom visuals.
  - iced has better text (cosmic-text), a real animation API, and `PaneGrid` for tiling. The risk is its slow release cadence.
- **Slint** is the most polished for standard UI, but its custom-drawing gap is exactly where this app lives. The GPL or a commercial license is also a factor.
- **Custom wgpu** only makes sense if the prototypes show neither framework can reach the target look or performance. It could also be used inside either framework for the graph panel only (egui paint callback, iced `shader` widget), which is a good fallback.

## Recommended shortlist

1. **egui/eframe + egui_tiles.** Lowest risk and fastest prototype. Very good custom painting and virtualization, idle-friendly, mature. Watch text rendering and how far restyling can go.
2. **iced 0.14.** Better text, animation and built-in `PaneGrid`, so more likely to look polished. Watch the manual canvas virtualization and the slow release cadence.

Prototype both on the same synthetic 200k-commit lane layout. Measure scroll fps, idle CPU and RSS, and let a human judge how it looks. If the graph panel falls short in either one, move just that panel to a wgpu custom primitive rather than changing frameworks.

## Sources

- egui releases: https://github.com/emilk/egui/releases (0.36.2, 2026-09-08)
- iced releases: https://github.com/iced-rs/iced/releases (0.14.0, 2025-12-07; feature list)
- Slint releases: https://github.com/slint-ui/slint/releases (1.18.1, 2026-09-21)
- vello releases: https://github.com/linebender/vello/releases (0.10.0, 2026-08-14; sparse strips 0.2.0, Glifo 0.3.0)
- Not fetched this session, recommended reading: https://github.com/rerun-io/egui_tiles, https://github.com/grovesNL/glyphon, https://github.com/pop-os/cosmic-text, https://docs.rs/iced/latest/iced/widget/pane_grid/
- RAM figures are rough estimates from general knowledge, not measurements. The prototype benchmark should measure them.
