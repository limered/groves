//! Ticket 03 red test: the staleness boundary of the default ref set.
//! A pure predicate takes `now` as a parameter, so the exact boundary
//! can be pinned deterministically — no wall clock in the assertion.

use groves_core::is_stale;

const DAY: i64 = 24 * 60 * 60;

#[test]
fn the_staleness_boundary_is_inclusive() {
    let now = 1_790_000_000;
    let stale_after_days = 14;

    assert!(
        !is_stale(now - 14 * DAY, now, stale_after_days),
        "a remote branch updated exactly 14 days ago is still fresh"
    );
    assert!(
        is_stale(now - 14 * DAY - 1, now, stale_after_days),
        "one second past the window the branch is stale"
    );
    assert!(
        !is_stale(now, now, stale_after_days),
        "a branch updated this second is fresh"
    );
}
