# gix capabilities research (ticket 002)

Date: 2026-09-24. gix version at time of writing: **0.87.1** (docs.rs, released 2026-08-25).
Labels: **[verified]** = seen in docs/source/local repo; **[estimate]** = inference, not measured (no `cargo` on this machine, so no benchmark was run).

## 1. Local repo facts (BM_AI_Software_Factory, read-only inspection)

- `git version 2.55.0.windows.5`, `credential.helper manager` (GCM), `credential.https://dev.azure.com.usehttppath true`, remote is Azure DevOps HTTPS (`brickmakers.visualstudio.com`). [verified]
- Commit-graph: **split chain** at `objects/info/commit-graphs/` (3 layers, newest from today). No single `objects/info/commit-graph`. [verified]
- `objects/pack/multi-pack-index` present (8 MB, written today) plus 2 `.idx` files (`pack-79bea8…` 8 MB, `pack-8e29…` with `.mtimes` = cruft pack). [verified]
- **161 `.pack` files, only 2 `.idx`.** The idx-less ones are `pack-*.pack`, `loose-*.pack` (from `git maintenance` loose-objects task) and one 171 MB `.tmp-37100-pack-*.pack`. There are also 3 `tmp_pack_*` files. `git count-objects -v` reports them all as **garbage** (`garbage: 162`, `size-garbage: ~4.2 GB`) and warns "no corresponding .idx". `in-pack: 291107`, `size-pack: 208 MiB`. [verified]
  - So git itself does not use these packs: they are leftovers of interrupted/failed repack/maintenance runs. Most likely cause on Windows: another process holding the old pack files open/mmapped, so git cannot delete or rename them (plus interrupted gc). [estimate, but consistent with timestamps and `.tmp-<pid>-pack` naming]
- `objects/info/packs` is stale (lists 5 packs) — only used by dumb-HTTP; ignore it. [verified]
- `packed-refs`: 7,140 lines; loose refs under `refs/`: 5 files. [verified]
- `worktrees/`: `a2m1706-repro` (detached at `c2f30f2…`), `organ-pipe-gila` (`ref: refs/heads/feature/A2M-1707_RAG-MCP-Tool`) + main worktree = 3. [verified]
- `git rev-list --all --count` = 16,612, wall time ~720 ms (cold process incl. startup + 7k ref peel). [verified] — baseline for gix to beat.

## 2. Walking all commits from all refs

API [verified, docs.rs gix 0.87.1 `revision::walk::Platform`]:
```rust
let mut repo = gix::open(path)?;            // or ThreadSafeRepository for worker threads
repo.object_cache_size_if_unset(64 << 20);  // docs: "highly recommended" if objects are looked up after
let tips = repo.references()?.all()?.filter_map(|r| r.ok()?.peel_to_id_in_place().ok().map(|id| id.detach()));
let walk = repo.rev_walk(tips)
    .sorting(Sorting::ByCommitTime(CommitTimeOrder::NewestFirst)) // or BreadthFirst (default)
    .use_commit_graph(true)
    .all()?;
for info in walk { let info = info?; /* info.id, info.parent_ids, info.commit_time; info.object()? for message/author */ }
```
- Sorting options: `BreadthFirst`, `ByCommitTime(NewestFirst|OldestFirst)`, `ByCommitTimeCutoff`. **No git-style `--topo-order`** (docs explicitly say BreadthFirst ≠ topo-order). → The app must compute topological order itself (e.g. collect all commits, then Kahn/DFS with commit-time tie-break). This is needed for graph layout anyway, so plan it in the layout ticket. [verified]
- Commit-graph is used automatically when present (`core.commitGraph`), and load errors fall back to the ODB ("treated as optional cache"). gix-commitgraph reads split chains (`commit-graph-chain`) — matches this repo. It provides parents, commit time, generation number. It does **not** read Bloom filters or corrected generation dates (v2) (crate-status.md). [verified]
- Commit-graph only accelerates parent/time; message/author need the commit object from the pack anyway. For a viewer that shows author + subject for every row, every commit gets decoded once → pack/zlib decode dominates. Docs note that when all commits' details are needed, turning the graph off is at most a micro-optimization.
- Expected speed [estimate, not measured]: gix decodes commits from a well-packed repo at order of ~1–3 µs graph-only, ~5–15 µs with full commit decode. 17k commits → **~50–250 ms**; 200k commits → **~1–3 s** single-threaded cold, less warm. Must run on a background thread; stream rows to the UI. Recommend validating with a throwaway bench before the perf-budget ticket (`cargo` not installed here).
- Threading: `Repository` is `!Sync`; use `ThreadSafeRepository` + `to_thread_local()` per worker. Walk itself is single-threaded; parallelism possible only by decoding commit objects in parallel after collecting ids from the graph.
- Note crate-status still lists "commitgraph support" unchecked for gix-traverse and "Use Commit Graph to speed up certain queries" unchecked for gix — but `Platform::use_commit_graph` exists and gix-revwalk has "commit-graph acceleration". Treat acceleration as partial.

## 3. Refs (loose + packed, 50k)

- `repo.references()?.all()` / `.prefixed("refs/tags/")` etc. merges loose and packed refs, sorted, worktree-aware (private vs common refs). packed-refs is memory-mapped/buffered and binary-searched; handles unsorted/headerless files. [verified, crate-status gix-ref]
- The packed buffer is reloaded automatically when the file's mtime changes. [verified — gix-ref `packed::modifiable` / "auto-refresh" behaviour; also gix-odb "auto-refresh of on-disk state"]
- Cost [estimate]: 7k refs ≈ single-digit ms; 50k ≈ 10–30 ms iterate. **Peeling annotated tags** needs an object read each (6,732 tags here!). packed-refs stores peeled values (`^` lines) when written with `peeled` trait; use `Reference::peel_to_id_in_place` which uses the packed peeled value where available, otherwise one tag-object lookup (~µs). Acceptable.
- No reftable support (Git 3.0 feature) — not an issue for these repos (`extensions` not set).

## 4. Worktrees

[verified, docs/crate-status "worktrees: open a repository with worktrees, read locked state"]
- `repo.worktrees()?` → `Vec<worktree::Proxy>`; each has `id()`, `base()` (path), `is_locked()`, `git_dir()`; `proxy.into_repo_with_possibly_inaccessible_worktree()?` yields a `Repository` whose `head()` / `head_id()` / `head_name()` give that worktree's HEAD. Main worktree: `repo.main_repo()` / `repo.head()`.
- Linked worktrees share the ODB and common refs; opening each as its own `Repository` duplicates caches but reuses mmaps per process [estimate]. Better: open once (common dir), read the per-worktree `HEAD` files via the proxies' repos, and share one commit walk. Since each worktree is its own pane, a pane = (shared repo data, worktree HEAD).
- Missing: "prunable" detection (worktree dir deleted). Handle by checking `base()` exists → show pane as "missing".

## 5. `.pack` without `.idx`

- gix-odb's dynamic store discovers packs **via index files** (`*.idx`) and `multi-pack-index`; a `.pack` without `.idx` is never loaded — same as git. It will not error. [verified by design of gix-odb store (indices are the unit of loading); consistent with git treating them as garbage]
- MIDX is supported (read) [verified crate-status], so this repo's `multi-pack-index` is used.
- `tmp_pack_*` and `.tmp-*` files are ignored by name.
- gix does **not** write idx for them and must not (read-only). Nothing to "tolerate" beyond: don't treat pack count as health signal.
- **Important Windows interaction:** gix memory-maps `.idx`/`.pack` files. On Windows an open mapping prevents deletion/rename, so a long-running viewer holding the repo open can itself cause `git gc`/`git maintenance`/auto-gc (triggered by our own `git fetch`) to fail and leave exactly this kind of garbage. Mitigation for the spec: drop all `Repository`/ODB handles for a repo before spawning `git fetch` (and while idle, if feasible), reopen after; or pass `-c gc.auto=0 -c maintenance.auto=false` to our fetch so we don't trigger gc while holding mmaps (the user's own git/IDE still may). [estimate — mechanism well-known, not reproduced]
- After a fetch adds new packs, gix-odb auto-refreshes on object miss (`RefreshMode::AfterAllIndicesLoaded`, default) — no restart needed; reopening is still the simplest correct approach.

## 6. Cheap change detection between refreshes

No watcher (standing decision). Recommended two-level check [design, based on verified APIs]:
1. **Stat fingerprint** (µs): mtime+size of `packed-refs`, `HEAD` of each worktree (`worktrees/*/HEAD`), `FETCH_HEAD`, and mtime of `refs/` directories (walk loose refs — few files). If unchanged → nothing to do.
2. **Ref snapshot diff** (ms): `BTreeMap<FullName, ObjectId>` of all refs (+ worktree HEADs) vs previous. Diff yields added/removed/moved refs. New tips = moved/added targets.
3. Incremental walk: `rev_walk(new_tips).with_hidden(old_known_tips)` to fetch only new commits (docs: hidden tips make traversal costlier, may traverse all if disjoint). Simpler and robust alternative at our scale: full re-walk (~≤3 s at 200k in background) and replace the model; decide in the refresh-model ticket. Note force-pushed/deleted branches make purely additive updates wrong — commits can disappear.

## 7. Changed-file list for a commit

[verified API: `Repository::diff_tree_to_tree`, `object::tree::diff::Platform`, `Tree::changes()`]
```rust
let commit = repo.find_commit(id)?;
let tree = commit.tree()?;
let parent_tree = commit.parent_ids().next().map(|p| repo.find_commit(p)?.tree()).transpose()?
    .unwrap_or_else(|| repo.empty_tree());
let changes = repo.diff_tree_to_tree(&parent_tree, &tree, gix::diff::Options::default().with_rewrites(None))?;
// each change: Addition/Deletion/Modification(/Rewrite) with location (path), entry mode, ids
```
- Tree-to-tree diff only descends into subtrees whose ids differ → cost ∝ changed directories, not repo size. Typical commit: sub-ms to few ms [estimate]. Huge commits (e.g. 10k-file template updates) can be 10–100 ms; lazily compute on selection, off the UI thread, cache by commit id.
- Rename detection optional (`with_rewrites(Some(..))`) — costs blob reads; recommend off by default or "identity-only" renames (cheap, no blob content compare).
- Merge commits: diff against first parent (GitKraken-like) — policy decision for detail-panel ticket.
- No Bloom filters in gix, irrelevant here (only helps path-filtered log).

## 8. `fetch --all`: shell out vs gix fetch

gix fetch exists (`remote.connect(Direction::Fetch)?.prepare_fetch(..)?.receive(..)`), with credential helpers via `gix-credentials` (launches `git credential-<name>`, i.e. GCM `manager` works if git is installed), http via curl or reqwest feature. [verified crate-status] But gaps [verified crate-status]:
- does **not write `FETCH_HEAD`**, no remote groups (`--all` = iterate remotes yourself), no auto-maintenance and no "auto-explode small packs" → every gix fetch adds a pack, and nothing ever repacks unless git runs → pack pile-up.
- credential helper URL matching after `insteadOf`/redirect not done; `credential.<url>.*` / `usehttppath` normalized URL matching listed as missing in gix-config → **risk with this Azure DevOps setup (`usehttppath true`)**.
- ssh needs external `ssh` anyway; Windows prompts for mingw/cmd not supported; binary grows (curl/reqwest + TLS).

**Recommendation: shell out to `git fetch --all --prune`** (the user's installed git with GCM, proxies, `insteadOf`, maintenance all "just work"). Windows specifics:
- spawn with `CREATE_NO_WINDOW` (`std::os::windows::process::CommandExt::creation_flags(0x08000000)`), set `GIT_TERMINAL_PROMPT=0` (and optionally `GCM_INTERACTIVE=never` for background fetches, so a periodic fetch never pops a GUI login; manual fetch may allow interaction).
- run with `-C <repo>`, capture stderr for error display, timeout + kill.
- consider `-c gc.auto=0` for background fetches only if handles are not dropped (see §5); otherwise let git maintain.
- `git` must be on PATH → startup check, clear error if missing.

## 9. Windows notes (summary)

- mmaps block deletion of packs → drop repo handles around fetch (§5). This is likely a contributor to the garbage packs seen in both repos.
- `core.longpaths true`, `core.ignorecase true` — gix respects these for worktree ops; irrelevant for read-only history, but paths in diffs are bytes (`BStr`); convert lossy for display.
- `core.fscache` is a Git-for-Windows-only setting, ignored by gix (no effect on us).
- gix-discover does not prevent crossing filesystems on Windows (non-issue).
- `safe.directory` not enforced by gix (it restricts untrusted config instead) — repos owned by another user still open, with reduced config trust.

## 10. Verdict

gix is fit for purpose: rev-walk with split commit-graph + MIDX, merged loose/packed refs, worktree enumeration, tree-diff changed-file lists are all present and read-only. Gaps to design around: no topo-order (do it ourselves), no fetch parity (shell out to git), idx-less packs are simply ignored (harmless), and Windows mmap locking (release handles around fetch).

## Sources

- docs.rs gix 0.87.1 `revision::walk::Platform`: https://docs.rs/gix/latest/gix/revision/walk/struct.Platform.html
- docs.rs gix-traverse 0.61.0 `Sorting`: https://docs.rs/gix-traverse/latest/gix_traverse/commit/simple/enum.Sorting.html
- gitoxide crate status (main): https://github.com/GitoxideLabs/gitoxide/blob/main/crate-status.md
- gix API index: https://docs.rs/gix/latest/gix/ (Repository::worktrees, diff_tree_to_tree, references)
- git docs: https://git-scm.com/docs/git-count-objects (garbage), https://git-scm.com/docs/gitformat-commit-graph (split chains), https://git-scm.com/docs/git-maintenance (loose-objects task)
- Local read-only commands on BM_AI_Software_Factory: `git count-objects -v`, directory listings of `objects/pack`, `objects/info`, `worktrees`, `git config --get-regexp`, `git rev-list --all --count`.
