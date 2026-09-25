---
id: 003
title: Which lane-layout algorithm fits a 200k-commit graph?
labels: [wayfinder:research]
parent: 000
blocked_by: []
assignee: agent
status: closed
---

## Question

Survey commit-graph lane assignment approaches (git log --graph, GitKraken/Fork style straight-lane layouts, `git-graph` crate, gitui/lazygit, Sapling/Sublime Merge write-ups): algorithm, complexity, lane stability when new commits arrive on top, visual quality (edge crossings, long lanes), memory per commit. Can a full relayout of 200k commits run < 100 ms, or is incremental layout needed?

## Resolution

- Full relayout, no incremental: estimated ~30–60 ms native for 200k commits (topo sort + O(n·w) lane pass); unmeasured, gate it in the perf harness. Bottleneck is reading commits (use commit-graph file + in-memory DAG cache), not layout.
- Algorithm: pvigier temporal topological sort + straight-branch lanes (no shifting, free-slot reuse, forbidden-column check), preferred refs (HEAD/main) seeded leftmost; tags never own lanes; no merge-message parsing (git-graph's weakness).
- Stability: persisted `ref → lane` hint map across relayouts; scroll anchored by SHA. ~12–16 B/commit SoA + edge interval index for viewport rendering.
- Details: [research/graph-layout.md](research/graph-layout.md)
