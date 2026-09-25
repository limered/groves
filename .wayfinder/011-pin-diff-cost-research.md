---
id: 011
title: How expensive is the changed-file union for a pin set?
labels: [wayfinder:research]
parent: 000
blocked_by: []
assignee: emil.bohleber
status: closed
---

## Question

For a pin set of 1..50 commits at Factory/200k scale, measure or estimate gix tree-diff cost per commit (normal, large, merge commits), decide the merge-commit diff base (first parent vs. combined), and whether results need caching or cancellation to stay off the 8 ms frame budget.

## Resolution

See [research/pin-diff-cost.md](research/pin-diff-cost.md).

- Measured on the Factory repo (16.7k commits, 9.4k files): 50 commits diffed in one batched run take about 90 ms of work (≈2 ms/commit). A commit touching about 4k files takes about 45–60 ms net per single run. Each git process adds about 100 ms to start on Windows.
- Estimate for gix in-process with rename tracking off: 0.1–1 ms for a normal commit, 5–20 ms for a 4k-file commit. Worst case for 50 pins is up to about 1 s cold.
- Merge commits: diff against the **first parent**. `--cc` shows almost nothing (0–3 files); all parents costs about 2× and adds noise.
- **Cache** each commit's path list by commit OID (never goes stale, LRU). Compute on the **background git worker**, with a generation counter and cooperative cancellation between commits. Build the union from the cached sets. Never compute on the UI thread.
