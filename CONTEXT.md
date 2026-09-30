# groves

A read-only commit-graph viewer. The name: nine groves, each a group of repository graphs.

## Glossary

- **Pane**: one tiled view showing the graph of a single repository, with the HEAD of every worktree of that repository marked on it. A repository is open in at most one pane.
- **Grove**: one of nine numbered layouts of panes; exactly one is visible at a time. _Avoid_: workspace.
- **Ref set**: the refs a pane draws history for; only commits reachable from it are shown.
- **Default ref set**: the ref set a pane starts with: local branches, their upstreams, `origin/HEAD`, all worktree HEADs, and remote branches updated within the staleness window.
- **Stale branch**: a remote branch not updated within the staleness window (default 14 days); excluded from the default ref set.
- **Pill**: the label for a branch on its commit row; filled for local, outline for remote.
- **Synced pill**: a single pill standing for a local branch and its upstream when both point at the same commit.
- **Tag**: shown as a rectangle, never a pill; labels commits but never adds to the ref set.
- **Worktree HEAD marker**: the marker on the commit a worktree's HEAD points at; all worktrees are marked alike, only the main worktree's name is shown by default.
- **Commit info**: a commit's short SHA, author name, title and relative date; hidden by default, revealed on hover or when pinned.
- **Pin**: a commit whose commit info stays visible; clicking a commit toggles its pin. A pane holds any number of pins.
- **Pin set**: all pinned commits of a pane; the detail panel describes the pin set, not a single commit.
- **Age line**: a horizontal line across the graph marking where commits become older than 3h, 1d, 1w or 3w (by committer date).

## Flagged ambiguities

- "Tree" is never used for a pane: it collides with git's worktree and tree object. Say pane.
