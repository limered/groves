---
id: 015
title: "Spec: groves (commit-graph viewer)"
labels: [ready-for-agent]
source_map: 000
status: open
---

## Problem Statement

I work across several git repositories at once. Each has multiple worktrees, hundreds of remote branches and thousands of tags. Today the Factory repo alone has 16.6k commits and 7,140 refs, and it grows by about 80 commits a day. Existing graph viewers (GitKraken and similar) are heavy, slow on repos this size, bury the graph under tags and stale remote branches, and show one repo per window. I want to see at a glance, for several repos side by side, where every worktree's HEAD sits relative to the branches that matter, and how old things are, without the tool using noticeable CPU or memory while it sits open all day.

## Solution

**groves** is a fast, lightweight, **read-only** git graph viewer for Windows and Linux, written in Rust. One window holds nine **Groves**. Each grove tiles **Panes** automatically in a Hyprland-like dwindle layout and is driven by the keyboard. A pane shows the graph of one repository, with the **Worktree HEAD marker** of every worktree of that repository on it. By default a pane draws only history reachable from the **Default ref set**, so stale remote branches and tags stay out of the way until asked for. The pane stays quiet: graph, **Pills**, HEAD markers and **Age lines**. **Commit info** appears on hover. Clicking commits **Pins** them, and a detail panel describes the whole **Pin set**. Repos fetch in the background every five minutes. The only network operation is `git fetch`, and nothing is ever written to a repository.

## User Stories

### Panes, groves, tiling

1. As a developer, I want to open a repository through the OS folder dialog (`Alt+Enter`), so that I can add a pane without configuring anything.
2. As a developer, I want picking a worktree folder to open its repository, so that I never get two panes for the same repo.
3. As a developer, I want opening an already-open repository to highlight its existing pane, so that each repo lives in at most one pane.
4. As a developer, I want a new pane to split the focused pane in alternating directions (dwindle), so that the layout stays tidy without manual placement.
5. As a developer, I want to resize panes by dragging their edges, so that I can give a busy repo more room.
6. As a developer, I want to move focus with `Alt+h/j/k/l` or `Alt`+arrows, so that I can navigate without the mouse.
7. As a developer, I want to swap panes with `Alt+Shift+h/j/k/l`, so that I can rearrange the layout.
8. As a developer, I want to close the focused pane with `Alt+q`, so that I can drop repos I no longer watch.
9. As a developer, I want to toggle fullscreen for the focused pane with `Alt+f`, so that I can study one graph closely.
10. As a developer, I want nine numbered groves switched with `Alt+1..9`, so that I can group repos by project.
11. As a developer, I want to move the focused pane to another grove with `Alt+Shift+1..9`, so that I can regroup.
12. As a developer, I want a thin status bar that shows the current grove, so that I know where I am.
13. As a developer, I want an `Alt+.` overlay that lists every key, so that I don't have to memorise the keymap.
14. As a developer, I want an empty grove to show a hint ("`Alt+Enter` open repo · `Alt+.` keys"), so that I know what to do.
15. As a developer, I want groves, the pane tree, split ratios, the repo in each pane, window geometry and the last active grove restored at startup, so that my setup survives restarts.
16. As a developer, I want a pane whose repository has disappeared to show a "missing, remove?" placeholder, so that one moved repo doesn't break startup.

### Graph and ref set

17. As a developer, I want only commits reachable from the pane's **Ref set** drawn, with lanes laid out again for that subset, so that irrelevant history doesn't widen the graph.
18. As a developer, I want the default ref set to be local branches, their upstreams, `origin/HEAD`, every worktree HEAD and remote branches updated in the last 14 days, so that the graph shows active work.
19. As a developer, I want `s` to toggle **Stale branches** into the ref set, so that I can look at old remote work when needed.
20. As a developer, I want `f` to open a filter box centred over the greyed-out pane that takes include/exclude globs (e.g. `origin/feature/*, !origin/renovate/*`), so that I can narrow or widen the ref set precisely.
21. As a developer, I want pane keys suspended while I type in the filter box, so that typing a pattern doesn't trigger toggles.
22. As a developer, I want straight, stable lanes with HEAD/main seeded leftmost, so that the graph is readable and doesn't jump.
23. As a developer, I want lanes to stay stable across refreshes and filter changes, so that I don't lose my orientation.
24. As a developer, I want 22 px rows and 14 px lanes, so that the graph is dense but legible.
25. As a developer, I want the graph column at its natural width, draggable wider, and never drawn over text, so that nothing overlaps.

### Pills, tags, HEAD markers

26. As a developer, I want one **Pill** per commit (HEAD's branch first, then local, then remote), with any other refs under `+N`, so that rows stay uncluttered.
27. As a developer, I want pills drawn in their lane's colour, filled for local and outline for remote, so that I can tell them apart at a glance.
28. As a developer, I want a local branch and its upstream on the same commit shown as one **Synced pill**, so that in-sync branches don't take two labels.
29. As a developer, I want long pill text shortened, with the full name on hover, so that pills don't eat the row.
30. As a developer, I want clicking a pill or `+N` to open a dropdown of every branch ref on that commit (local and remote grouped), including refs hidden by the filter, so that nothing is unreachable.
31. As a developer, I want tags hidden by default and toggled with `t`, drawn as rectangles and never adding to the ref set, so that thousands of tags don't bury the graph.
32. As a developer, I want every worktree HEAD shown with the same coloured **Worktree HEAD marker** and included in the ref set, so that I see where every worktree stands.
33. As a developer, I want only the main worktree's name shown by default, other names on hover, and a click on a marker toggling its name, so that labels don't clutter.
34. As a developer, I want a detached HEAD to show its SHA, so that it's still identifiable.
35. As a developer, I want jump-to-HEAD to go to the main worktree first and cycle through the others on repeated presses, so that I can find each worktree quickly.

### Commit info, pins, detail panel

36. As a developer, I want no commit text by default, so that several panes side by side stay calm.
37. As a developer, I want hovering a commit node to fade in its **Commit info** inline (`a1b2c3d  Emil  Fix lane reuse  · 3h ago`), so that I can inspect without clicking.
38. As a developer, I want `m` to peek commit info on all rows, so that I can scan messages when I need to.
39. As a developer, I want clicking a node to toggle a **Pin** that keeps its info visible, with any number of pins, so that I can compare commits.
40. As a developer, I want `Esc` to clear all pins, so that I can start over.
41. As a developer, I want pins to survive fetches and filter changes, with a pruned pinned commit marked "hidden by filter" in the panel, and pins on unreachable commits dropped silently, so that pins behave predictably.
42. As a developer, I want `d` to toggle a bottom detail panel, hidden at start, so that details are there on demand.
43. As a developer, I want the detail panel to list the union of files changed by the pin set (flat, with A/M/D/R and coloured SHA chips of the pinned commits that touched each file), so that I see what a group of commits changed.
44. As a developer, I want merge commits diffed against their first parent, so that merges list meaningful files.
45. As a developer, I want the full messages of pinned commits below the file list, newest first, each headed by SHA chip, author and date, so that I can read context.
46. As a developer, I want an empty pin set to show "Click commits to pin them", so that the panel explains itself.
47. As a developer, I want the panel to update live and never freeze the UI while diffs are computed, so that big commits don't hurt.

### Age

48. As a developer, I want horizontal **Age lines** at 3h, 1d, 1w and 3w (committer date), each drawn once above the first row from the top older than its threshold and labelled at the right edge (`── 1d ago`), so that I can see how fresh work is.
49. As a developer, I want thresholds that no row reaches skipped, and out-of-order rows ignored, so that lines are honest and uncluttered.

### Refresh and fetch

50. As a developer, I want every open pane in every grove fetched in the background every 5 minutes (first fetch about 10 s after the pane opens), so that remotes stay current.
51. As a developer, I want at most 2 fetches at a time, round-robin, never two for the same repo, so that the machine and network stay calm.
52. As a developer, I want `r` to fetch the focused pane and `Shift+R` all panes, jumping the queue and allowing credential prompts, so that I can refresh on demand.
53. As a developer, I want background fetches to be silent (no prompts), so that nothing pops up unexpectedly.
54. As a developer, I want a spinner on the pane header while fetching, a warning badge with the last error on hover, and "last fetched X ago", so that I know the pane's freshness without modals.
55. As a developer, I want the interval to double after 3 consecutive failures (capped at 1 h) until a success or a manual `r`, so that broken remotes don't cause churn.
56. As a developer, I want local ref changes (commits, checkouts in any worktree) detected within about 10 s for visible panes, and again on window focus or grove switch, so that the graph follows my terminal work without a filesystem watcher.
57. As a developer, I want the view to follow new tips when I'm scrolled to the top and otherwise stay anchored to the commit I'm looking at, so that refreshes don't yank me away.
58. As a developer, I want the old graph drawn until the new one is ready, so that refreshes never flash or block.

### Status bar

59. As a developer, I want the status bar to show the focused pane's active toggles (`tags`, `stale`, filter pattern), so that I know why the graph looks the way it does.
60. As a developer, I want tag toggle, stale toggle and filter pattern reset at startup, so that every session starts from the default view.

### Configuration and persistence

61. As a developer, I want a hand-edited `config.toml` with `fetch_interval_minutes`, `stale_after_days` and optional `git_path`, created once with commented defaults and never rewritten, so that my edits and comments survive.
62. As a developer, I want a broken config to fall back to defaults with a warning naming the bad line, so that a typo never stops the app.
63. As a developer, I want a corrupt or incompatible `state.json` backed up as `state.json.bak-<timestamp>` and the app to start with one empty grove and a one-line notice, so that I never lose the file silently.
64. As a developer, I want state written atomically, debounced about 1 s after layout changes and once on exit, so that a crash never corrupts it.
65. As a developer, I want data kept next to the exe in `groves-data/` when writable, otherwise in OS config/state dirs, so that the app is portable.

### Packaging and platform

66. As a developer, I want one portable binary per OS (Windows zip, Linux tarball), with no installer, so that installing means unzipping.
67. As a developer, I want the Linux binary to run on distros with glibc ≥ 2.31 and on both Wayland and X11, so that it works on my Hyprland box and older machines.
68. As a developer, I want the viewer to work fully without git installed, with only fetching disabled and "git not found" shown, so that visualization never depends on git.
69. As a developer, I want a clear error dialog naming the found GL version if OpenGL 3.3 / GLES 3.0 is unavailable, so that I know why it won't start.
70. As a Windows user, I want an embedded icon, version info and per-monitor DPI awareness, so that the app looks right on my displays.

### Performance

71. As a developer, I want a 200k-commit / 50k-ref repo cold-opened in ≤ 1.5 s (≤ 300 ms for today's Factory repo), so that opening is instant in practice.
72. As a developer, I want relayout after a toggle or filter change in ≤ 100 ms, so that toggles feel immediate.
73. As a developer, I want frames in ≤ 8 ms while scrolling, idle CPU below 1%, and RAM ≤ 250 MB base plus ≤ 30 MB per pane, so that the viewer can stay open all day.
74. As a developer, I want the viewer to tolerate `.pack` files without `.idx`, so that repos with failed maintenance still open.

## Implementation Decisions

### Stack

- Rust, egui/eframe 0.36 with the OpenGL (`glow`) renderer. wgpu and iced were rejected: the prototype measured about 224 MB RSS / 0.2% idle CPU on OpenGL against 554 MB / 4% on wgpu at 200k commits.
- Reading repositories: `gix` (0.87 line), read-only. It uses the commit-graph file and multi-pack-index when present, merges loose and packed refs, and `repo.worktrees()` gives each worktree's HEAD.
- Fetching: shell out to system `git fetch --all --prune`, never gix fetch (gix fetch doesn't write FETCH_HEAD, builds up packs and has GCM URL mismatches). On Windows use `CREATE_NO_WINDOW`. Background fetches set `GIT_TERMINAL_PROMPT=0` and `GCM_INTERACTIVE=never`; manual fetches allow prompts. git is found on `PATH` or at `git_path`.
- No async runtime.

### Crates (onion architecture, deep modules, narrow facades)

- **core**: the domain. DAG cache, ref-set computation, graph layout, age lines, pin set. It defines the `CommitSource` port and has no gix or UI dependencies. Its facade is `core::Pane`.
- **repo**: the gix/git adapter that implements `CommitSource`. It also produces the ref snapshot and changed-path lists and runs fetch. Its facade is `repo::open(path)`.
- **app**: egui, tiling, groves, input, persistence, scheduler. It wires everything together.
- **bench**: the performance harness. It drives `core` and `repo` through the same seams, using synthetic sources.

### Threading

- The UI thread (egui) only draws and handles input.
- A shared task pool (rayon-style short tasks) loads, lays out and diffs.
- One scheduler thread owns the fetch timer, the 10 s ref poll and the max-2 `git fetch` queue (round-robin, one fetch per repo at a time, manual fetches jump the queue, 3 failures double the interval up to a 1 h cap).
- Repo handles are opened per task and dropped afterwards. They are never held across a fetch, because on Windows gix's memory maps block git gc.

### Per-pane data model

- **DAG cache**: compact struct-of-arrays (SoA) layout: an OID table, CSR parent indices, committer time and an interned author id. It covers every commit reachable from the *full* ref set. Roughly 10–15 MB per pane at 200k commits.
- **Layout**: a separate `Arc<Layout>` for the *current* ref set, holding row order, lanes and edges (about 12–16 B per commit, plus an edge interval index for viewport rendering). Changing `s`, `t` or the pattern only lays out again; it never reloads.
- **Commit info** (title, author, date) is loaded eagerly for commits in the current ref set, alongside the layout. Commits that are only in the cache stay unloaded.
- **Changed-path lists**: cached per commit OID in an LRU (they never go stale). They are computed on the task pool with a generation counter and cooperative cancellation between commits, diffed against the first parent with rename tracking off, and never computed on the UI thread. The pin-set union is built from the cached sets.

### Layout algorithm

- pvigier-style temporal topological sort (gix has no `--topo-order`, so ordering is ours) plus straight-branch lanes: no shifting, free-slot reuse, a forbidden-column check, and preferred refs (HEAD/main) seeded leftmost.
- Tags never own lanes. No merge-message parsing.
- Always a full relayout. The prototype measured about 31 ms for 200k synthetic commits with 18 lanes at most.
- A `ref → lane` hint map is kept in memory across relayouts for stability (not persisted). Scroll position is anchored by SHA.

### Refresh

- Local change detection: every 10 s for visible panes, plus on window focus and when a grove becomes visible. The check compares a cheap ref snapshot (mtimes of worktree HEAD files, `packed-refs`, `refs/`), and refs are read fully only when it changed.
- On a change (after a fetch or a local change): full reload of the DAG from gix, full relayout with the lane hints, then an atomic swap. The UI keeps drawing the old layout meanwhile.
- After the swap: if scrolled to the top, follow the tips; otherwise keep the SHA anchor. Pins on unreachable commits are dropped; pins on pruned commits are kept and marked.

### Pane display rules

- Pill choice per commit: HEAD's branch, then local, then remote. Local + upstream on the same commit collapse into a synced pill. Everything else goes into the `+N` dropdown, which lists branch refs only (no tags).
- The default ref set as in the glossary. The staleness window comes from `stale_after_days`.
- Age lines as in user stories 48–49.
- Single-key pane bindings `t s f d m r Esc` (plus `Shift+R`). Fixed `Alt` keymap as in user stories 6–13. Nothing is configurable.

### Persistence

- `config.toml` (hand-edited, global): `fetch_interval_minutes = 5`, `stale_after_days = 14`, optional `git_path`. Written once with comments, never rewritten. On a parse error: defaults plus a warning badge naming the line. Everything else is hard-coded (ref poll interval, age thresholds, keymap).
- `state.json` (app-written, serde): `version`, groves, pane tree + split ratios, repo per pane, window position/size/maximized, last active grove. Unknown keys are ignored. Written debounced about 1 s after changes and on exit, atomically (temp file + rename). Corrupt state: back it up with a timestamp and start empty.
- Location: `<exe dir>/groves-data/` if writable, else the `directories` OS dirs (`%APPDATA%\groves\`; `$XDG_CONFIG_HOME/groves/` for config and `$XDG_STATE_HOME/groves/` for state).
- SQLite was rejected (the state is under 10 KB, and a database would add a C dependency, migrations and lose the ability to fix the file by hand).

### Packaging

- One portable binary per OS: a zip on Windows, a tarball on Linux. Linux is built with `cargo zigbuild` against glibc 2.31 (musl was rejected because GL/Wayland are loaded at runtime), with winit's Wayland and X11 both enabled.
- Windows: embedded icon and version resource, per-monitor DPI awareness. No code signing, no shortcut, no updater.
- If GL 3.3 / GLES 3.0 is missing: an error dialog naming the version found.
- Releases are built locally by the owner with one documented script.

### Budgets (target 200k commits / 50k refs)

Cold open ≤ 1.5 s (≤ 300 ms for today's Factory repo), relayout ≤ 100 ms, refresh ≤ the cold-open budget, frame ≤ 8 ms, idle CPU < 1%, RAM ≤ 250 MB base + ≤ 30 MB per pane.

## Testing Decisions

- **What makes a good test:** it drives a facade and checks behaviour a user would notice (which commits and rows appear, lane positions, pill choice, age-line positions, the pin-set union), never internal structure. Tests must survive refactors of the DAG or layout internals.
- **Seam 1 (main): `core::Pane` via the `CommitSource` port.** A seeded synthetic source (configurable commit count, refs, merge rate, branch fan-out, worktrees, dates) feeds the pane. It covers: default ref set and stale window, the `s`/`t`/pattern toggles and pruning, include/exclude globs, lane stability across relayout and refresh, HEAD/main leftmost, age-line placement (including skipped thresholds and out-of-order rows), pin behaviour across refresh and filter (kept, "hidden by filter", dropped when unreachable), pin-set union and first-parent merges, and SHA-anchored scroll against following the tips.
- **Seam 2: `repo::open` against an on-disk fixture repo** built with `git fast-import` from the same seed and cached under `target/`. It covers: reading commits, refs (loose + packed), worktrees and detached HEADs; tolerating packs without `.idx` (a fixture switch); ref-snapshot change detection; changed-path lists. Fetching gets a local bare "remote" fixture, tested with and without git on `PATH`.
- **Seam 3: `app`.** No automated UI tests. Pure functions only: `state.json` and `config.toml` round trips and failure handling (corrupt state backup, bad config falls back to defaults), the portable/OS-dir location choice, dwindle split logic. Visual behaviour is checked by hand.
- **Performance harness** (report-only, no gates, on demand): criterion micro-benches (relayout, DAG build, ref-set computation) and a headless `bench` binary (cold open to first frame, refresh, peak RSS, frame time during a scripted scroll). Idle CPU is measured by hand (60 s, 3 panes open). The reference is the current Windows dev box (Core Ultra 7 165H, 64 GB); results are compared against the budgets.
- **Prior art:** none in-repo (greenfield). The throwaway `prototype/` crate (`graphcore`: synthetic generator, git-log loader, straight-lane layout) shows how the synthetic generator and layout can be shaped and measured.

## Out of Scope

- Any write operation: commit, push, checkout, merge (`git fetch` is the only network or mutating command).
- Code review and diff viewing (file lists only, no hunks).
- Search (message/SHA/author): cut for now.
- Cross-repo relationships (e.g. a consumer repo's template version).
- Filesystem watching of `.git`.
- macOS.
- Update mechanism, installers, code signing.

## Further Notes

Future work (in scope for the product but not for this spec; came from the map's Not yet specified):

- Incremental refresh (walk only new commits) if a full reload misses its budget.
- Filtering from inside the graph (hide or solo a lane from its pill).
- Tag extras: an always-shown tag pattern (e.g. `v*`), and where the tags of a commit are listed.
- Theming and a GitKraken-like visual language beyond the prototype baseline.
- CI (GitHub Actions likely): building and publishing releases, turning the benches into gates, a Linux reference machine.

Scale baseline measured on 2026-09-24: Factory repo 16.6k commits, 7,140 refs (6,732 tags, 398 remote branches), 3 worktrees, about 80 commits/day, 208 MiB of packs. Clarora.AI: 1.5k commits, 236 refs. Many `.pack` files exist without `.idx`.

Full decision trail: the wayfinder map [Commit-graph viewer spec](000-map.md) and its closed tickets. The glossary is in `CONTEXT.md`.
