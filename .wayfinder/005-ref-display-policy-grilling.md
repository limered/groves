---
id: 005
title: How are thousands of tags and remote branches shown without burying the graph?
labels: [wayfinder:grilling]
parent: 000
blocked_by: []
assignee: emil.bohleber
status: closed
---

## Question

Decide the ref display policy given ~6.7k tags and ~400 remote branches: default visibility of tags, collapsing multiple refs on one commit, local vs. remote pill styling, branch filter UX (include/exclude patterns, hide merged/stale), how HEAD and each worktree's HEAD are marked, and whether filtering also prunes the commits drawn.


## Resolution

Grilled 2026-09-24.

- **Tags:** hidden by default. The per-pane tag toggle (persisted) shows them. Tags are drawn as **rectangles** (not pills). Tags are labels only and never add history to the drawn set. An always-shown tag pattern (e.g. `v*`) is a later feature.
- **Branch pills:** a pill takes its lane's color; **local = filled, remote = outline**. A local branch and its upstream on the same commit collapse into one pill with a "synced" marker. Any remaining refs show as first pill + `+N`.
- **Ref dropdown:** clicking a pill or `+N` opens a dropdown listing every branch ref on the commit (local/remote grouped), including ones hidden by the filter. **Tags are not listed** in the dropdown.
- **HEAD marking:** this pane's HEAD gets a bold ring on the commit dot and a `HEAD` badge on its pill. HEADs of other worktrees of the same repo get a subtle marker with the worktree name. A detached HEAD shows the SHA instead of a branch name.
- **Filter prunes history:** only commits reachable from the **ref set** are drawn, and lanes are laid out again for that subset. This pane's HEAD is always in the set.
- **Default ref set:** local branches + their upstreams + `origin/HEAD` + all worktree HEADs + remote branches updated in the last N days (default 14, configurable).
- **Stale branches:** remote branches older than N days. A per-pane toggle (`s`, persisted, off at startup) adds all of them.
- **Filter UX:** an on-demand pattern field (include/exclude globs, e.g. `origin/feature/*, !origin/renovate/*`) opened by a keystroke (e.g. `b`) and persisted per pane. **No checklist panel.** Filtering from inside the interactive graph is future work.
