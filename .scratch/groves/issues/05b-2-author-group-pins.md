# 05b-2: Author group pin rules

**Parent:** spec 015 (stories 75–80, the pin-related parts)

**What to build:** The three rules that connect author groups to the **Pin set**:

- Shift+click on a collapsed group expands it and pins all its commits.
- Collapsing (the collapse marker or `g`) unpins every commit inside, so a collapsed group never contains a pin.
- A group that would contain a pin after a refresh or filter change starts expanded.

These need the pin set from 07 and the folding mechanism from 05b-1; `fold` already takes `&Pins`, so this ticket only feeds the real set in and wires the interactions.

**Blocked by:** 07, 05b-1

**Status:** ready-for-agent

**Decisions (recorded before code)**

- **2026-11-24 — split from 05b:** sequenced after 07 because every rule here reads or mutates the pin set, which doesn't exist before it. Splitting keeps 05b-1 free of stubs while preserving the model-first order (folding mechanism before 06/07 consume it).

**Rust focus:** `HashSet` membership as the single source of truth, pure recomputation of "group would contain a pin" from the pin set, interaction wiring in `app`.

- [ ] Shift+click on a collapsed group expands it and pins all its commits (tests via `core::Pane`).
- [ ] Collapsing via the collapse marker or `g` unpins every commit inside; a collapsed group never contains a pin.
- [ ] A group that would contain a pin after a refresh or filter change starts expanded.
- [ ] Manual: shift+click, collapse-unpin and refresh-start-expanded behave as described.
