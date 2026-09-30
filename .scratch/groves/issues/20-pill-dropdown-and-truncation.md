# 20: Pill dropdown and truncation

**Parent:** spec 015

**What to build:** Long pill text is shortened, with the full name on hover. Clicking a pill or `+N` opens a dropdown of every branch ref on that commit, local and remote grouped, including refs hidden by the filter. Tags are not listed.

**Blocked by:** 04

**Status:** ready-for-agent

**Rust focus:** Unicode-safe truncation (`char_indices`, not byte slicing), egui popups.

- [ ] Truncation never splits a character and keeps a recognisable suffix (tests)
- [ ] The dropdown's ref list: grouping and inclusion of filtered refs (tests via `core::Pane`)
- [ ] Manual: hover and dropdown work
