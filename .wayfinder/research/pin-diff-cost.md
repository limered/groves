# Pin-set changed-file union: cost

Question: ticket 011. Tags: **[measured]** = timed on this machine, **[estimate]** = worked out from gix design or source knowledge, not timed.

## Test repo

`C:\_source\.ai\BM_AI_Software_Factory` (read-only commands only): 16,723 commits (`--all`), 9,432 tracked files, git 2.55 (Windows). Timings are the median of 5 runs, taken with PowerShell `Measure-Command`.

## (1) Tree-diff cost per commit

Each `git` process costs about **100 ms to start on Windows** [measured: `git --version` median 112 ms, min 93 ms]. So a single-command timing mostly measures process startup. The numbers below subtract that startup cost.

| Case | Files changed | Wall (median) | Net of spawn [measured, approx.] |
|---|---|---|---|
| Normal commit | a few | 126 ms | ~15 ms |
| Large commit | 3,967 | 172 ms | ~60 ms |
| Large commit | 3,906 | 157 ms | ~45 ms |
| Merge, first parent (small) | 3 | 137 ms | ~25 ms |
| Merge, first parent (large) | 4,527 | 158 ms | ~45 ms |
| Merge `--cc` | 0 / 3 | 122–127 ms | ~10–15 ms |
| **50 recent commits, one `--stdin` process, piped through sort -Unique** | 1,192 unique | **190 ms** | **~90 ms total, ≈ 2 ms/commit** |

The batched run gives the best per-commit figure, because the process starts once and the object cache stays warm. Most of the "net" time in single runs is repo open and cold pack or index loading, not the diff itself.

**gix [estimate]:**
- `gix::Tree::changes()` / `gix_diff::tree` is a path-only tree-to-tree walk that skips subtrees whose OIDs match. No blob is read unless rewrite tracking is on.
- Cost grows with the number of *changed subtrees*, not with repo size.
- In a long-lived process with an open `Repository` and an object cache, expect roughly:
  - **0.1–1 ms** for a normal commit
  - **5–20 ms** for a commit touching about 4k files (mostly tree-object decompression plus path allocation)
- That is the same or better than git's in-process cost, which the batched 2 ms/commit average shows.
- Turn rename tracking off (`track_rewrites(None)`). With it on, gix may read blobs for similarity checks, which costs a lot more on large commits.

**Worst case, 50 pins [estimate]:**
- Typical: about 50 × 1 ms plus the union step, **≈ 5–50 ms**
- Pathological (many 4k-file commits, cold cache): **up to ~0.5–1 s**
- The union step is a `HashSet`/`BTreeSet` of paths and costs almost nothing (1,192 unique paths from 50 commits).

## (2) Diff base for merge commits

| Option | Meaning | Cost | Observed |
|---|---|---|---|
| First parent | "What this merge brought into the branch" (the whole merged topic) | 1 tree diff | 3 files and 4,527 files on the two sample merges |
| All parents (`-m`) | Changes relative to each parent, then united; answers "what differs from any parent" | N tree diffs, usually 2× | Same union here (4,527); for a normal merge, the union is at least as large as the first-parent diff |
| Combined (`--cc`) | Only paths that differ from *every* parent, i.e. conflict resolutions and evil merges | N diffs plus intersection; in gix this means building it yourself (no ready-made combined-diff API) | 0 and 3 files |

For a pin set, the union question is "which files did these commits touch?":
- **First parent** fits git-log/GitHub conventions and the users' mental model.
- `--cc` leaves out nearly everything and misleads users.
- All-parents adds noise from the other side's history.

## (3) Caching and background work

- **Cache per commit OID**: map `oid → Arc<[path]>` (per pane, or shared per repo).
- This cannot go stale: a commit's tree never changes.
- It makes toggling pins cheap, because only the newly pinned commit needs a diff and the union is rebuilt from cached sets in microseconds.
- An LRU of a few hundred entries is enough.
- **Run it off the UI thread**:
  - A single large commit (about 5–20 ms estimated) already exceeds the 8 ms frame budget.
  - A cold 50-pin set can take hundreds of ms.
  - So run it on the existing git worker (see 008) and publish the result with a generation counter.
  - The panel shows "computing…" or the partial union.
- **Cancellation**: cooperative. Check the generation or a flag between commits and drop stale results. There is no need to interrupt a single commit's diff.
- **Optional**: when the pinned set is very large, cap the displayed paths (for example the first 5k). Rendering cost could exceed diff cost.
