# 05: Worktree HEAD markers

**Parent:** spec 015

**What to build:** Every worktree of the repository gets the same **Worktree HEAD marker** on its HEAD commit, and every worktree HEAD joins the ref set. By default only the main worktree's name is shown. Other names show on hover, and clicking a marker toggles its name. A detached HEAD shows its short SHA. Jump-to-HEAD scrolls to the main worktree first and cycles through the others on repeated presses. Opening a worktree folder opens its repository.

**Blocked by:** 03

**Status:** ready-for-agent

**Rust focus:** `Option` combinators (`map`, `and_then`, `unwrap_or`), gix's worktree API, a small cycling state.

- [ ] `repo` lists all worktrees with their HEADs, including detached ones (fixture with 3 worktrees)
- [ ] A commit reachable only from a secondary worktree's HEAD is drawn
- [ ] Opening a linked worktree path yields the same repository as opening the main one
- [ ] Jump-to-HEAD order: main first, then the others, then wraps around
- [ ] Manual: markers, hover names and click-toggle work
