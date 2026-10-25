# 03: Default ref set

**Parent:** spec 015

**What to build:** The pane draws only history reachable from the **Default ref set**: local branches, their upstreams, `origin/HEAD`, and remote branches updated within the last 14 days (hard-coded until config lands in 11). Lanes are laid out again for that subset. The DAG cache holds every commit reachable from the *full* ref set; the layout covers only the current ref set. After this ticket the app is usable on the Factory repo.

**Blocked by:** 02

**Status:** in-progress

**Rust focus:** enums with data (`RefKind::Local`, `Remote { … }`, `Tag`), iterator chains (`filter`/`map`/`collect`), `HashSet`, time handling, and separating the cache from the layout (`Arc<Layout>`).

**Decision:** The 14-day cutoff is relative to the current time, using each remote ref's tip commit time. Older remote-only branches stay available through the stale-branch toggle (`s`, issue 13).

- [ ] `repo` produces a ref snapshot: loose and packed refs, upstreams, `origin/HEAD`, each ref's tip time
- [ ] A remote branch older than 14 days is left out, together with history only it reaches
- [ ] Tags never add to the ref set
- [ ] Commits reachable only from stale refs don't widen the graph (lane count assertions)
- [ ] Manual: the Factory repo shows noticeably fewer lanes than in 02

## Manual checks

- [ ] `cargo run -p groves-app -- <Factory-repo>` opens the repo and lays out only the default ref set; the graph has noticeably fewer lanes than in issue 02.
- [ ] Confirm a recent remote branch appears, while a remote branch older than 14 days and a tag-only history do not add rows or lanes.
