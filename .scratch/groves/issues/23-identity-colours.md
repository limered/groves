# 23 (OPTIONAL): Identity colours

**Parent:** spec 015

**Optional:** Emil's call. Not on the critical path — the frontier ignores this until he explicitly picks it up.

**Status:** parked

**Blocked by:** 04

**What to build:** Colour follows **branch identity** instead of lane position: a lane's line wears a colour derived from its owning branch, so `main` keeps its hue across re-layouts and a branch is recognisable at a glance anywhere in the graph. Colour is picked from the same woody palette by a stable function of the branch name (e.g. `hash(short_name) % palette.len()`), and pills inherit the colour of their branch's line — GitKraken's "the release spur is always orange" feel.

**Rust focus:** `std::hash` (`DefaultHasher` over `&str`), carrying state through the layout (line identity across lane reuse), palette tables, and reading git's decoration semantics again for *which* line a pill belongs to.

**Why parked:** In 04 the layout has no notion of "which branch this lane belongs to" — lane slots are reused across unrelated branches after merges, so line colour by name needs **line identity threading through the layout**. That's core-layout surgery with settled invariants (ticket 03's cleanup), and for two adjacent lines a name-hash can still pick the same palette entry — the collision risk sits exactly in the dense graphs where lane colours matter most. `lane % N` avoids both problems.

- [ ] Layout tracks a line identity per lane through fork, merge and lane reuse
- [ ] Colour pick from branch name is consistent across re-layouts (same graph → same colours, different graph → colours per name, not per spawn order)
- [ ] Adjacent visible lanes never share a hue (probe strategy on collision, or another documented tie-break)
- [ ] Spec 27 re-read: "pills drawn in their lane's colour" under identity colouring — amend or record the new reading before code
- [ ] Manual: side-by-side against `lane % N` on a dense repo (the bm factory repo, ~17 lanes); get lost once in each mode, be honest about which one you recover from faster
