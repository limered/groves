# 19: Detail panel off the UI thread

**Parent:** spec 015

**What to build:** Changed-path lists are computed on a shared task pool and never on the UI thread. They are cached per commit OID in an LRU (they never go stale). A generation counter plus cooperative cancellation between commits means changing the pin set abandons outdated work. The panel updates live, and a 4k-file commit never freezes the UI.

**Blocked by:** 18, 08

**Status:** ready-for-agent

**Rust focus:** rayon, `AtomicU64` generations, cooperative cancellation, the `lru` crate, `Mutex` vs message passing.

- [ ] Cached results are reused (tests)
- [ ] A newer generation cancels an older one; stale results are discarded (tests)
- [ ] Manual: pin a very large commit and keep scrolling smoothly
