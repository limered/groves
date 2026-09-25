---
id: 006
title: How do panes, tiling and workspaces behave?
labels: [wayfinder:grilling]
parent: 000
blocked_by: []
assignee: emil.bohleber
status: closed
---

## Question

Define the pane/tiling model: what a pane binds to (repo, repo+worktree, or repo with worktree switcher), layout algorithm (dwindle/master), keybindings (focus, swap, resize, fullscreen, close, open repo), fast repo switching (picker? recent list?), multiple workspaces/tabs, what is persisted and restored at startup, and how a pane for the same repo in two worktrees shares data.

## Resolution

- **Pane = one repository**, all worktree HEADs marked on its graph. A repo is open in at most one pane; opening an already-open repo highlights its pane.
- **Worktree HEAD markers**: coloured marker on the commit dot; worktree name on hover; click toggles name visibility.
- **Jump to HEAD** (repeat): main worktree first, each further press cycles to the next worktree HEAD.
- **Layout**: dwindle only (new pane splits the focused one, alternating direction). Resize by mouse-dragging pane edges; no resize keys.
- **Workspaces**: 9 numbered, one visible, indicator in a thin status bar.
- **Open repo**: OS folder dialog only (no recent list, no scan roots). A worktree folder resolves to its repository.
- **Keymap** (fixed, not configurable): `Alt+hjkl`/arrows focus, `Alt+Shift+hjkl` swap, `Alt+Enter` open, `Alt+q` close, `Alt+f` fullscreen, `Alt+1..9` workspace, `Alt+Shift+1..9` move pane to workspace, `Alt+.` keymap overlay (layout-independent choice).
- **Persisted & restored**: workspaces, pane tree + split ratios, repo per pane. Per-pane view state not persisted. Missing repo at startup -> "missing, remove?" placeholder.
- **Empty workspace**: centred hint "`Alt+Enter` open repo · `Alt+.` keys".
- Spawned: [What does a pane show by default?](009-pane-default-display-grilling.md).
