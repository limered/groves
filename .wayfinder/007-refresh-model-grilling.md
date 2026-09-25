---
id: 007
title: When and how does a pane refresh after fetch?
labels: [wayfinder:grilling]
parent: 000
blocked_by: [002]
assignee: emil.bohleber
status: closed
---

## Question

Decide the refresh model without a file watcher: background fetch interval and scope (all repos vs. visible panes), concurrency limits, what triggers a reload besides fetch (e.g. cheap periodic ref-snapshot poll to catch local commits/checkouts?), how the view preserves scroll position/selection when new commits arrive, and how fetch errors/auth prompts are surfaced.

## Resolution

- **Background fetch**: every 5 min (app-wide setting), first fetch ~10 s after a pane opens. Covers all open panes in all 9 workspaces.
- **Concurrency**: max 2 `git fetch` processes, round-robin queue, never two for the same repo; manual fetch jumps the queue.
- **Local change detection** (no watcher): every 10 s for visible panes, compare a cheap ref snapshot (mtimes of worktree HEAD files, `packed-refs`, `refs/`), full ref read only on change. Also re-check on window focus and when a workspace becomes visible.
- **Keys**: `r` fetches the focused pane, `Shift+R` fetches all open panes; spinner on pane header while fetching.
- **Auth**: background fetch is silent (`GIT_TERMINAL_PROMPT=0`, `GCM_INTERACTIVE=never`); manual `r` allows credential prompts.
- **Errors**: warning badge on pane header (last error on hover) + "last fetched X ago"; no modals/toasts. After 3 consecutive failures the interval doubles (cap 1 h) until success or manual `r`.
- **View on new commits**: if scrolled to top, follow new tips; otherwise keep SHA anchor. Pins whose commits are no longer reachable are dropped silently; detail panel updates.