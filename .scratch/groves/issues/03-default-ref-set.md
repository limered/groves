# 03: Default ref set

**Parent:** spec 015

**What to build:** The pane draws only history reachable from the **Default ref set**: local branches, their upstreams, `origin/HEAD`, every worktree HEAD, and remote branches updated within the last 14 days (hard-coded until config lands in 11). Lanes are laid out again for that subset. The DAG cache holds every commit reachable from the *full* ref set; the layout covers only the current ref set. After this ticket the app is usable on the Factory repo.

**Blocked by:** 02

**Status:** done

**Rust focus:** enums with data (`RefKind::Local`, `Remote { … }`, `Tag`), iterator chains (`filter`/`map`/`collect`), `HashSet`, time handling, and separating the cache from the layout (`Arc<Layout>`).

**Decision:** The 14-day cutoff is relative to the current time, using each remote ref's tip commit time. A branch updated exactly 14 days ago counts as fresh; staleness starts one second past the window. The boundary lives in a pure `core::is_stale(tip_time, now, stale_after_days)` so tests can pin it without a wall clock. Older remote-only branches stay available through the stale-branch toggle (`s`, issue 13).

- [x] `repo` produces a ref snapshot: loose and packed refs, upstreams, `origin/HEAD`, the main worktree's `HEAD`, each ref's tip time — short names (`main`, `origin/main`; secondary worktrees and detached heads land in 05)
- [x] A remote branch older than 14 days is left out, together with history only it reaches
- [x] Tags never add to the ref set
- [x] Commits reachable only from stale refs don't widen the graph (lane count assertions)
- [x] Manual: the Factory repo shows noticeably fewer lanes than in 02

## Manual checks

- [x] `cargo run -p groves-app -- <Factory-repo>` opens the repo and lays out only the default ref set; the graph has noticeably fewer lanes than in issue 02. *(Confirmed on the Factory repo, 23k commits: smooth scroll, short open time.)*
- [x] Confirm a recent remote branch appears, while a remote branch older than 14 days and a tag-only history do not add rows or lanes. *(Confirmed; the pass also caught two crashes on GitKraken-managed repos — refs like `refs/worktree/gitkraken/undo` that don't peel to commits now skip instead of failing `open()`.)*

## Learned

- gix's `references()` platform unifies loose and packed refs behind one iterator, and `category_and_short_name()` reads git's namespace taxonomy — ref classification is data, not string surgery. Worktree heads are the read of a *slot* (`HEAD`), not data gix hands you.
- Read-only means degrade: skip refs that don't peel or classify instead of failing `open()`. And `-1` sentinels lost to `Option` in the lane slot vec — absent states belong in the type, not in magic values.

## Cleanup under 03 (final pass)

- [x] `map_category_to_ref_kind` returns `Option<RefKind>`; unknown categories (e.g. `refs/bisect/*`, GitKraken's `refs/worktree/*`) skip instead of masquerading as `WorktreeHead`. The `PseudoRef` arm goes too — `references()` never yields HEAD, the worktree loop supplies it separately; worktree-prefixed refs are ticket 05's territory.
- [x] Upstream lookups degrade like the GitKraken fixes do: `.transpose().ok().flatten()` — a broken `branch.*.merge` config costs only the pill link, not the pane. Then drop `RepoError::NoUpstream`.
- [x] Delete the dead `let _refs` line and the never-read `remote` computation in `open()`'s worktree loop (ticket 05 re-adds it if its pills need it).
- [x] **Settled — do NOT add the `contains` guard to the first-parent slot overwrite.** Retraced before you burn an hour: the guard makes the two_merges shape put `c1` in lane 2 (the newer line's claim) instead of lane 1, breaking `two_merges_of_one_spur_follow_gits_own_columns`. The unguarded overwrite is what makes the *leftmost* promise win for a doubly-claimed commit — and that's exactly which line git's own render picks. My earlier "behavior-neutral" note was wrong.
- [x] `cargo fmt` + clippy nits (unused import in the removed diag's wake).
