# 14: Filter box `f`

**Parent:** spec 015

**What to build:** `f` opens a filter box centred over the greyed-out pane. It takes include/exclude globs, e.g. `origin/feature/*, !origin/renovate/*`. Pane keys are suspended while typing. The pattern narrows or widens the ref set and triggers a relayout. It is shown in the status bar and reset at startup.

**Blocked by:** 13

**Status:** ready-for-agent

**Rust focus:** parsing text into your own types (`FromStr`, error enums), `globset`, focus and keyboard routing in egui.

- [ ] Pattern parsing, including errors for malformed input (tests)
- [ ] Include/exclude semantics on the ref set (tests)
- [ ] Manual: typing `s` or `t` in the box doesn't toggle anything
