---
id: 009
title: What does a pane show by default?
labels: [wayfinder:grilling]
parent: 000
blocked_by: []
assignee: emil.bohleber

status: closed
---

## Question

Rework what a pane shows by default, now that one pane shows a whole repository with all worktree HEADs: which columns/fields are visible, which markers (worktree HEADs, pills, tags) and labels are on or off by default, what toggles exist and how they are reached, and how this refines the prototype's layout (004) and the ref display policy (005).

## Resolution

Grilled 2026-09-25. Supersedes parts of 004 (always-visible message/author/date/SHA; single-commit detail panel), 005 (single "this pane's HEAD"; persisted toggles/filter) and 006 (worktree names).

- **Visible by default:** graph, worktree HEAD markers, branch pills (one per commit + `+N`), main worktree's name label, age lines. **No commit text by default** - too noisy with several panes open.
- **HEADs:** all worktree HEADs equal, same coloured marker, no `HEAD` badge, all in the ref set. Only the main worktree's name shows by default; clicking a marker toggles its name.
- **Commit info** = short SHA, author name, title, relative date. Hovering a commit node fades it in, **inline** in the row right of the graph (`a1b2c3d  Emil  Fix lane reuse  · 3h ago`). Clicking the node toggles a **pin** (info stays visible); pins accumulate.
- **Graph column:** natural width, draggable wider; rest of the row is free for commit info.
- **Age lines:** horizontal lines at 3h, 1d, 1w, >3w; drawn above the first row (from the top) whose committer date is older than the threshold, out-of-order rows ignored; each drawn once, label at right edge (`── 1d ago`); thresholds no row reaches are skipped.
- **Detail panel** (`d` toggles; hidden at start): describes the **pin set**. File list = union of changed files, flat, each with change kind (A/M/D/R) and coloured SHA chips of the pinned commits that touched it. Below: full messages of pinned commits, newest first (committer date), each headed by its SHA chip, author, date. Updates live. Empty pin set -> hint "Click commits to pin them".
- **Pins:** survive fetch and filter changes; a pruned pinned commit stays in the panel marked "hidden by filter". `Esc` clears all pins.
- **Pane keys** (single keys, no `Alt`, listed in `Alt+.` overlay): `t` tags, `s` stale branches, `f` filter, `d` detail panel, `m` peek commit info on all rows, `r` fetch now, `Esc` clear pins.
- **Filter box:** opens centred over the pane with the pane greyed out; while typing, pane keys are suspended.
- **Status bar** shows active toggles of the focused pane (`tags`, `stale`, filter pattern).
- **Nothing persisted:** tag toggle, stale toggle and filter pattern all reset at startup (reverts 005's persistence).
