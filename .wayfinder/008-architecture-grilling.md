---
id: 008
title: What is the core architecture and data model?
labels: [wayfinder:grilling]
parent: 000
blocked_by: [001, 002, 003, 007]
assignee: emil.bohleber
status: closed
---

## Question

Decide threading/process model (UI thread vs. per-repo loader workers), in-memory commit/graph representation (one repo per pane, no cross-pane sharing needed), crate/module boundaries, memory and CPU budgets, and how data flows from gix -> layout -> renderer on load and on refresh.

## Resolution

- **Threading**: UI thread (egui) + shared task pool (rayon-style short tasks: load, layout, diff) + one scheduler thread owning the 5-min fetch timer, 10 s ref poll and the max-2 `git fetch` queue. No async runtime.
- **Repo handles**: opened per task, dropped after; never held across a fetch (Windows mmap rule from 002).
- **Per-pane model**: compact SoA DAG cache (OID table, CSR parent indices, committer time, interned author id) for all commits reachable from the full ref set; separate `Arc<Layout>` (row order, lanes, edges) for the current ref set, so `s`/`t`/pattern changes only relayout. ~10-15 MB/pane at 200k commits.
- **Commit info**: title/author/date loaded eagerly for commits in the current ref set, alongside layout; cache-only commits stay unloaded. Search indexing decided later.
- **Refresh (MVP)**: on ref-snapshot change, full reload of DAG from gix, full relayout with lane-hint map, atomic swap; UI keeps drawing old layout meanwhile. Incremental walk deferred.
- **Crates (onion, deep modules, narrow facades)**: `core` (domain: DAG, ref set, layout; defines `CommitSource` port; no gix/UI deps; facade `core::Pane`), `repo` (gix/git adapter implementing `CommitSource`; facade `repo::open(path)`), `app` (egui, tiling, input; wires it together), `bench` (harness, uses the same seam with synthetic sources).
- **Budgets** (target 200k commits / 50k refs; hard gates for the harness): cold open ≤ 1.5 s (≤ 300 ms today's Factory); relayout ≤ 100 ms; refresh ≤ cold-open budget (full reload); frame ≤ 8 ms; idle CPU < 1%; RAM ≤ 250 MB base + ≤ 30 MB/pane.