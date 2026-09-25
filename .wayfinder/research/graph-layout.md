# Research: lane-layout algorithm for a 200k-commit graph (ticket 003)

Status: done 2026-09-24. Confidence markers: **[verified]** = read in source/write-up during this research; **[recalled]** = prior knowledge, not re-checked this session.

## Problem framing

Layout = two steps:
1. **Row order** (y): a topological order, normally newest-first.
2. **Lane assignment** (x): pick a column for each commit plus routing for its edges.

Constraint for this project: 200k commits, 50k refs, one commit per row next to a commit list (GitKraken-like), refresh after `fetch` adds ~80 commits/day on top.

## Approaches surveyed

### 1. `git log --graph` (git `graph.c`) — streaming, curved lanes  **[verified]**
- Source: https://github.com/git/git/blob/master/graph.c
- Algorithm: walks commits in revision order; keeps `columns[]` (one per active parent pointer). Per commit, `graph_update_columns` builds `new_columns` from current columns + commit's interesting parents (dedup via linear scan `graph_find_new_column_by_commit`), and a `mapping[]` of where each line must move. Lanes are **compacted leftwards** ("we never have to move branches to the right"), drawn over extra COLLAPSING rows.
- Complexity: O(n · w) time (w = active lanes, linear scans), O(w) memory — no per-commit state; it's a renderer, not a layout store.
- Stability: poor for a GUI — any compaction shifts every lane to the right of it; lanes are not tied to branches. Needs extra non-commit rows (breaks one-row-per-commit).
- Visual: compact, many diagonal shifts; long histories get "braided". Has `graph_max_lanes` truncation.

### 2. Curved-branch "active list" (Git Extensions, SmartGit, gitk-ish, lazygit/gitui)  **[verified: pvigier; recalled: lazygit/gitui]**
- Source: pvigier write-up (below), "curved_branches" pseudo-code; lazygit `pkg/gui/presentation/graph` uses a similar pipe-per-row model with columns reused/compacted; gitui has no real graph (list only, as of last knowledge).
- Algorithm: iterate rows; commit replaces one of its branch children (first-parent child) in the active list, else inserted; other branch children removed → list shifts.
- Complexity: O(n · w). Memory O(n) for (lane) + O(w).
- Stability: low — removals shift lanes; a new commit on top can change lanes of everything below it that shares the active list.

### 3. Straight-branch layout (GitKraken-like; pvigier/gitamine)  **[verified]**
- Source: https://pvigier.github.io/2019/05/06/commit-graph-drawing-algorithms.html (master thesis chapter, gitamine https://github.com/pvigier/gitamine)
- Row order: **temporal topological sort** — DFS over children, roots visited in committer-date order newest→oldest. Always topological, deterministic, equals date order when dates are sane. O(n log n + m).
- Lane assignment ("straight_branches"): active list B, removals set `nil` (no shifting), new commits take a free `nil` slot or append. A commit continues a branch child's column unless that column is in the forbidden set J(c) = columns occupied between the row of its highest merge child and its own row (prevents merge edges crossing nodes/lanes). Lemma reduces J(c) to one range query from min(mergeChild.row).
- Complexity: ~O(n · w) with the "list of J_i" method; interval-tree variant O((n+m) log n + k). Benchmark in the write-up: **react repo (~10k commits in 2019) = 274 ms** (JS/Electron, list method), 482 ms interval tree.
- Stability: good within a lane (no shifting), but full relayout still re-derives slot reuse top-down, so inserting commits at the top can re-assign which freed slot older branches reuse.
- Visual: branches perfectly vertical, merge edges as bends; width can grow (append-only for obstacles) — needs slot reuse to stay narrow.
- Memory: per commit (row u32, lane u16/u32, color u8) + edges; render cost solved by interval tree over edge row-spans → 0.58 ms per visible frame.

### 4. `git-graph` crate (mlange-42 → git-bahn)  **[repo verified; algorithm recalled]**
- Source: https://github.com/git-bahn/git-graph (lib used by git-igitt)
- Algorithm: first assigns commits to **branches** (walk first-parent chains from each ref, branch-model TOML gives order/priority, merge-commit summaries parsed to name merged branches), then assigns each branch a column, reusing columns whose occupied row-interval doesn't overlap (interval packing by branch ordering groups).
- Complexity: O(n + refs · chain) for tracing, column packing O(b log b) over b branches. Built on git2 and loads everything eagerly.
- Stability: high per branch identity (column = f(branch, model)), but depends on merge-message parsing (fragile; README: "summaries of merge commits should not be modified", no octopus support).
- Relevance: branch-first idea is the best route to GitKraken look; the message parsing and 50k-ref fan-out need care (tags must not create branches).

### 5. Sapling `renderdag` / Sublime Merge  **[recalled]**
- Sapling (`eden/scm/lib/renderdag`): streaming column renderer like git's but with a fixed "column" vector, `Ancestor`/`Parent` edges and ASCII/box renderers; O(n · w). Designed for `sl smartlog` (small, pruned DAG), not a 200k scroll view.
- Sublime Merge: closed source; public statements only (fast, native). Not usable as reference beyond "native + virtualized is fast enough".

## Can a full relayout of 200k commits run < 100 ms?

Estimate (native Rust, data already in memory as dense arrays indexed by u32):
- Temporal topo sort: 200k nodes + ~220k edges, one sort of u64 timestamps + iterative DFS → **~5–15 ms**.
- Straight-lane assignment: per commit O(w) scan of active list; with w ≈ 20–60 active lanes typical, 200k · 50 ≈ 10M simple ops → **~10–30 ms**. Pathological w (hundreds of stale remote branches spanning history) → use a bitset of occupied columns per range (or the J_i list) to stay O(n · w/64).
- Edge interval index build: O(m log m) → **~10 ms**.
- pvigier's 274 ms for ~10k commits is JS with object graphs; native with SoA layout is plausibly 50–100× faster per commit, matching the estimate. **Unmeasured — must be confirmed by the benchmark harness.**

Main risk isn't CPU but **input**: walking 200k commits from packs via `gix` (parse parents + times) costs far more than layout (hundreds of ms cold). Use the commit-graph file (`.git/objects/info/commit-graph`) when present — it gives parents, generation numbers, commit times without decompression — and cache the parsed DAG in memory between refreshes.

## Stability when new commits arrive on top

Full relayout is deterministic, but a pure top-down greedy lets new top rows change slot reuse for older rows (visual "jump" after fetch). Mitigations, cheapest first:
1. **Anchor order bottom-up**: assign lanes from oldest to newest (or keep a persisted `branch → lane` hint map from the previous layout and prefer it). New commits then only affect the top.
2. **Branch-first lanes** (git-graph style): lane = f(ref identity), so new commits on existing branches never move others.
3. UI: keep scroll anchored to the selected commit's SHA, not row index.

## Recommendation

- **Full relayout, no incremental algorithm.** At 200k commits the layout pass should be ~30–60 ms native, well under 100 ms, and it runs off the UI thread after a fetch (~80 new commits/day don't justify incremental complexity or its bug surface).
- **Algorithm**: pvigier temporal topological sort + straight-branch lane assignment (free-slot reuse, forbidden-column check), with first-parent chains from preferred refs (HEAD/main/develop first) seeded into leftmost lanes — borrowing git-graph's branch-first priority but **without** merge-message parsing. Tags never own lanes.
- **Stability**: carry a `branch-tip-ref → lane` hint map across relayouts; prefer the hinted lane when not forbidden.
- **Data layout**: SoA arrays keyed by u32 commit index: `row u32, lane u16, color u8, parents offset` (~12–16 B/commit → ~3 MB at 200k) + edge interval index for viewport queries (render only visible rows/edges).
- **Gate**: add a benchmark (synthetic 200k DAG + real Factory repo ×10) to the perf harness ticket; if layout > 100 ms, fall back to "incremental top-prefix": relayout only rows above the old top, reusing the previous active-lane state at the boundary (possible because the old layout's lane state at any row is reconstructible).

## Sources
- git `graph.c`: https://github.com/git/git/blob/master/graph.c
- P. Vigier, "Commit Graph Drawing Algorithms" (2019): https://pvigier.github.io/2019/05/06/commit-graph-drawing-algorithms.html
- gitamine: https://github.com/pvigier/gitamine
- git-graph: https://github.com/git-bahn/git-graph
- Sapling renderdag (recalled): https://github.com/facebook/sapling/tree/main/eden/scm/lib/renderdag
- lazygit graph (recalled): https://github.com/jesseduffield/lazygit/tree/master/pkg/gui/presentation/graph
