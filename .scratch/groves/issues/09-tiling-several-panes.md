# 09: Tiling: several panes

**Parent:** spec 015

**What to build:** `Alt+Enter` opens the OS folder dialog and adds a pane by splitting the focused one in alternating directions (dwindle). Focus moves with `Alt+h/j/k/l` or `Alt`+arrows, `Alt+q` closes the focused pane, and `Alt+f` toggles fullscreen for it. Opening a repository that is already open highlights its existing pane instead of opening a second one.

**Blocked by:** 08

**Status:** ready-for-agent

**Rust focus:** a recursive `enum` tree with `Box` (`Node::Split { dir, ratio, a, b } | Node::Leaf(PaneId)`), walking and mutating it through `&mut`, `Option::take`, and removing a node while keeping the tree valid.

- [ ] Dwindle split logic as pure functions: insert, remove, find a neighbour in a direction (tests)
- [ ] Closing a pane collapses its parent split
- [ ] Duplicate-repo detection by canonical repository path (tests)
- [ ] Manual: open 4 repos, navigate, close, fullscreen
