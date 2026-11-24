# 05c: Detached main worktree shows its short SHA

**Parent:** spec 015 (story 34)

**What to build:** Ticket 05 handles detached *linked* worktrees, but the main worktree path in `repo::open` always uses the repo folder name. When the main worktree's HEAD is detached (common in CI checkouts), its marker must show the target commit's short SHA instead — same rule as any other worktree (story 34: "a detached HEAD shows its SHA, so it's still identifiable").

**Blocked by:** None (05 done)

**Status:** ready-for-agent

**Rust focus:** the same `head.referent_name()` / `short_id()` fallback already used for linked worktrees — extracting the shared head-to-name step instead of duplicating it.

- [ ] A fixture whose main worktree HEAD is detached lists the first `WorktreeHead` ref named by the main HEAD's short SHA, `is_main: true` (test in `crates/repo/tests/worktrees.rs` + one `core/tests/head_markers.rs` assertion)
- [ ] Manual: in a detached-HEAD checkout, the main marker ring carries the short SHA, and `h` still lands on that row first

## Manual checks

```
git init -q -b main detached-main && cd detached-main
echo x > f.txt && git add . && git commit -qm Root
echo y > f.txt && git add . && git commit -qm Tip
git checkout -q --detach HEAD~1
```

Then `cargo run -p groves-app -- detached-main`:

1. The top row's marker ring shows the short SHA of the detached commit, not the folder name.
2. `h` scrolls to that row on the first press.
