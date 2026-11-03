# 04: Pills

**Parent:** spec 015

**What to build:** Each commit row that has branches shows one **Pill**, chosen in order: HEAD's branch, then local, then remote. The pill is drawn in its lane's colour, filled for local and outlined for remote. A local branch and its upstream on the same commit show as one **Synced pill**. Any other branch refs on the commit show as `+N`.

**Blocked by:** 03

**Status:** in-progress

**Rust focus:** implementing `Ord` and sorting by key, `match` with guards, borrowing `&str` from owned data (your first real lifetimes), colour tables.

- [ ] Pill choice follows HEAD branch > local > remote (tests via `core::Pane`)
- [ ] Local + upstream on the same commit collapse into one synced pill
- [ ] `+N` counts the remaining branch refs (tags excluded)
- [ ] Manual: filled vs outline is clearly readable, colours match the lanes

## Decisions (mentor, recorded at red)

- **Decorations come from the full ref snapshot**, not the current ref set: a stale branch whose tip is still visible (history passes through it) keeps its label. Pills stay stable when `s` or filters change the ref set.
- **`origin/HEAD` is an alias**, never a pill, never in `+N` — otherwise every synced tip carries a pointless `+1`.
- **Secondary worktree heads** are markers, not pills (ticket 05 draws them); they don't pill or count.
- **Tags** never pill and never count (spec 26/31).
- **Ties inside a tier** resolve alphabetically. Several locals on one commit share the tier, so alphabetical covers them too; refined in 05 once the snapshot carries HEAD's referent name.
- "HEAD's branch" means **the local branch standing on the HEAD commit** (the referent name isn't in the snapshot yet), which beats locals standing elsewhere.

## Decision: pill zone (agreed with Emil, before UI code)

- Pills render **right of the graph, in-flow** (prototype + GitHub-commits behaviour): `x = left + graph_w`, pills flow left-to-right in a row, `short`/`title` follow right after that row's pills. — No fixed width tax, nothing can overlap the graph or each other; text columns lose their shared x, which is temporary until ticket 07 makes commit text hover-only and the pill flow becomes the row's always-on label. → **Revisit condition:** if long pills start fighting the text (ticket 20) or the hover layout (07), switch to the GitKraken-style left column — fixed zone `= widest pill, capped`, clipping + `+N` badge, lanes stay aligned. Discussed round two against Emil's GitKraken screenshot (left `BRANCH / TAG` column); he chose in-flow for "pills at the tips" and zero wasted left space.

## Manual checks

- [ ] `cargo run -p groves-app -- <branchy-repo>`: decorated rows show one pill right of the graph; undecorated rows draw nothing in the pill zone; nothing overlaps the short/title columns.
- [ ] On a synced tip (e.g. `main` and `origin/main` on the same commit) exactly one pill shows.
- [ ] Filled (local) vs outlined (remote) readable at a glance; the pill colour matches that row's lane colour.
- [ ] A pushed-but-unmerged branch shows an outlined pill on its tip commit.
