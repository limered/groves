# 13: `s`/`t` toggles and status bar

**Parent:** spec 015

**What to build:** `s` toggles **Stale branches** into the ref set, and `t` toggles tags, which are drawn as rectangles and never add to the ref set. Both only relayout, from the DAG cache; neither reloads the repo. Relayout takes ≤ 100 ms. The status bar shows the focused pane's active toggles. Toggles reset at startup.

**Blocked by:** 04, 08

**Status:** ready-for-agent

**Rust focus:** reusing the cache while rebuilding the layout, a first criterion micro-bench for relayout, flag sets.

- [ ] `s` brings stale branches and their history back; toggling again removes them (tests)
- [ ] `t` shows tags without changing rows or lanes (tests)
- [ ] Relayout at 200k synthetic commits is measured and reported against the 100 ms budget
- [ ] Manual: status bar shows `tags`/`stale`
