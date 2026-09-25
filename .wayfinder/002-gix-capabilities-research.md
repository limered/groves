---
id: 002
title: Can gix read commits, refs and worktrees fast and robustly enough?
labels: [wayfinder:research]
parent: 000
blocked_by: []
assignee: agent
status: closed
---

## Question

For `gix` (gitoxide), establish: API + expected speed for walking all commits reachable from all refs (commit-graph file usage, topo/date order) at 17k and 200k commits; reading loose + packed refs (50k refs); enumerating worktrees and their HEADs; behavior with `.pack` files missing `.idx` (as found in BM_AI_Software_Factory); cheap change detection between refreshes (ref snapshot diffing); computing a commit's changed-file list; whether `fetch --all` should shell out to `git` or use gix's fetch (auth/credential helpers on Windows). Note Windows specifics.

## Resolution

gix 0.87 is fit: `rev_walk` uses the split commit-graph + multi-pack-index automatically, refs merge loose + packed (7k now, 50k fine), `repo.worktrees()` gives each worktree's HEAD, and `diff_tree_to_tree` against the first parent gives changed files cheaply, computed only when a commit is selected. Speed is estimated, not measured: ~0.1 s for 17k commits, ~1–3 s for 200k, on a background thread.
Design around these gaps: gix has no `--topo-order`, so we order commits ourselves. Packs without `.idx` are unused garbage from failed git maintenance runs; git and gix both ignore them. On Windows, gix's memory maps block git gc, so drop repo handles before fetching.
Change detection: stat fingerprint first, then diff a ref snapshot. `fetch --all`: shell out to `git`, not gix fetch. gix fetch doesn't write FETCH_HEAD, builds up packs and doesn't fully match URLs for GCM/`usehttppath`. Use `CREATE_NO_WINDOW` and `GIT_TERMINAL_PROMPT=0`.
Details: [research/gix-capabilities.md](research/gix-capabilities.md)
