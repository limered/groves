# 04: Pills

**Parent:** spec 015

**What to build:** Each commit row that has branches shows one **Pill**, chosen in order: HEAD's branch, then local, then remote. The pill is drawn in its lane's colour, filled for local and outlined for remote. A local branch and its upstream on the same commit show as one **Synced pill**. Any other branch refs on the commit show as `+N`.

**Blocked by:** 03

**Status:** ready-for-agent

**Rust focus:** implementing `Ord` and sorting by key, `match` with guards, borrowing `&str` from owned data (your first real lifetimes), colour tables.

- [ ] Pill choice follows HEAD branch > local > remote (tests via `core::Pane`)
- [ ] Local + upstream on the same commit collapse into one synced pill
- [ ] `+N` counts the remaining branch refs (tags excluded)
- [ ] Manual: filled vs outline is clearly readable, colours match the lanes
