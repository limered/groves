# 21: Perf harness

**Parent:** spec 015

**What to build:** A `bench` crate that drives `core` and `repo` through the same seams as the tests. Criterion micro-benches cover relayout, DAG build and ref-set computation. A headless binary measures cold open to first frame, refresh, peak RSS and frame time during a scripted scroll. Results are reported against the budgets (report-only, no gates). The reader tolerates `.pack` files without `.idx` (fixture switch).

**Blocked by:** 13

**Status:** ready-for-agent

**Rust focus:** criterion, `black_box`, measuring memory per platform with `cfg`, reading a profile.

- [ ] `cargo bench -p bench` runs the micro-benches
- [ ] The headless bench prints a budget table for 200k commits / 50k refs
- [ ] The pack-without-idx fixture opens (tests)
- [ ] Manual: idle CPU over 60 s with 3 panes, recorded
