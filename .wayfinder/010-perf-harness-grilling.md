---
id: 010
title: How is the performance harness built and run?
labels: [wayfinder:grilling]
parent: 000
blocked_by: []
assignee: emil.bohleber

status: closed
---

## Question

Given the budgets fixed in 008, decide how they are measured: synthetic 200k-commit/50k-ref repo generation vs. real-repo fixtures, which metrics (cold open, relayout, refresh, frame time, idle CPU, RAM) are automated vs. manual, tooling (criterion, custom bin), and whether it gates CI.

## Resolution

- **Data**: three tiers. (1) Synthetic in-memory `CommitSource` generator (seeded; 200k commits / 50k refs, set merge rate and branch fan-out) for `core`. (2) On-disk fixture repo built by `git fast-import` from the same seed the first time a bench runs, cached under `target/`, with a switch that leaves some packs without `.idx`, for the `repo` adapter and cold open. (3) Real repos (Factory, Clarora) as an optional manual check.
- **Metrics**: criterion micro-benches for relayout, DAG build and ref-set compute. A custom headless `bench` bin for cold open (to first painted frame), refresh and peak RSS, plus frame time from a scripted scroll. Idle CPU is manual (60 s sample with the app idle and 3 panes open).
- **Gating**: none. Benches report only and are compared against the 008 budgets; no CI is chosen yet.
- **Reference machine**: the current Windows dev box (Intel Core Ultra 7 165H 16c/22t, 64 GB, NVMe SSD, Arc iGPU + RTX 2000 Ada). This is a fast machine, so budgets met here are not a minimum-spec guarantee. The Linux box can be added later as a second reference.
- **Cadence**: everything runs on demand for now; it can move into CI later.
