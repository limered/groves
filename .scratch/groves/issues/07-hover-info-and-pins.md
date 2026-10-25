# 07: Hover info, pins, `m`, `Esc`

**Parent:** spec 015

**What to build:** The pane shows no commit text by default. Hovering a node fades in its **Commit info** inline (`a1b2c3d  Emil  Fix lane reuse  · 3h ago`). Clicking a node toggles a **Pin** whose info stays visible, and a pane can hold any number of pins. `m` peeks at the info on all rows, and `Esc` clears all pins. Commit info is loaded eagerly for the current ref set.

**Blocked by:** 02, 05b

**Status:** ready-for-agent

**Rust focus:** where UI state lives and who owns it, `HashSet<Oid>`, egui hit-testing and input (compare with macroquad's input polling), relative-date formatting.

- [ ] The **Pin set** API on `core::Pane`: toggle, clear, query (tests)
- [ ] Relative date formatting (tests with an injected "now")
- [ ] Manual: hover fade, click toggles a pin, `m` peek, `Esc` clears
