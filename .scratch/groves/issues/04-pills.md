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

## Decision: colour model (agreed with Emil, 2026-11-05)

- **Palette**: Emil's own pick (woody feel), ~10 entries, contrast-checked on the dark background. A `const` table in `crates/app`'s render module — `core` stays colour-free.
- **Single source of truth**: one `lane_colour(lane) -> Color32` used by pills, edges and dots, so the manual check "colours match the lanes" holds by construction.
- **Mapping**: `lane % palette.len()`. Adjacent lanes never share a hue; lane 0 (the trunk) always wears entry 0; wholesale recolouring on re-layout is accepted. Identity colouring (colour follows the branch) is parked in ticket 23 — optional, Emil may pick it up later.
- **Edges**: one colour per edge — the colour of the lane the edge belongs to; **connectors** (the bend where a line splits off or merges in) get the colour of the new branch, i.e. the line the connector itself carries (Emil's phrasing; the renderer picks from `Edge { child_lane, parent_lane, is_first_parent }`). The manual pass judges whether it reads right.
- **Dots**: deliberately open — Emil experiments with look during the manual pass (ring vs solid, maybe even author-hashed fills). Not pinned by ticket 04.
- **Rejected**: colouring lanes/pills by `hash(branch name)` in 04 — collision with the point of lane colour (adjacent lines must differ in dense graphs) and the layout doesn't track branch identity anyway (see ticket 23).

## Decision: pill zone (agreed with Emil, flipped twice)

- ~~Pills render right of the graph, in-flow (prototype + GitHub-commits behaviour).~~ **Changed 2026-11-04 during ticket 04**, after testing against the bm factory repo: with ~17 dense lanes, in-flow pushes pills to the pane edge and long names (`Brickmakers.AIFactory.AgentPlugins/…`) run off; the recorded revisit trigger ("long pills fighting text/hover") fired early — same failure shape, just via lane count instead of ticket 20.
- **Now: GitKraken-style left column.** Fixed zone left of the graph, constant width (cap ≈ 24 chars, ellipsis on longer names), no connector line across the graph — the pill sits on its tip's row, colour = lane colour. Spec 29's shorten + hover maps onto the cap directly. Bonus: short/title columns keep their shared left edge permanently (in-flow would have taken it away again).
- Reasons against per-row in-flow re-confirmed: ragged pill x per row, jitter while scrolling, no natural cap.
- Scope: affects UI render + manual checks only; pill *choice* (core::Pane) is placement-agnostic, red tests unchanged. Ticket 05 (worktree HEAD markers) draws its markers into the same left zone.
- Zone width is a constant, not "widest pill in view" — no reflow while scrolling. Tune the number later if it wastes space on sparse repos.

## Manual checks

- [ ] `cargo run -p groves-app -- <branchy-repo>` (try a dense one like the bm factory repo): decorated rows show one pill in the fixed left column at a stable x; undecorated rows show nothing in the zone; long names ellipsis at the cap and don't push anything around.
- [ ] On a synced tip (e.g. `main` and `origin/main` on the same commit) exactly one pill shows.
- [ ] Filled (local) vs outlined (remote) readable at a glance; the pill colour matches that row's lane colour.
- [ ] A pushed-but-unmerged branch shows an outlined pill on its tip commit.
