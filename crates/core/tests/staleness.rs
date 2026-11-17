//! Ticket 03 red test: the staleness boundary of the default ref set.
//! A pure predicate takes `now` as a parameter, so the exact boundary
//! can be pinned deterministically — no wall clock in the assertion.
//!
//! Refactor additions: the pane-level window and clock. `Pane::load` takes
//! `GraphOptions { now, stale_after_days }` — the caller stamps the clock and
//! config owns the window, so these tests pin that both values actually come
//! from the options, not from a constant or the wall clock.

use groves_core::{GraphOptions, Pane, is_stale};
use groves_testkit::{GraphCommit, SyntheticSource, TEST_NOW};

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

#[test]
fn the_stale_window_comes_from_the_options_not_a_constant() {
    // A remote branch updated 20 days ago: past the default 14-day window,
    // inside a 30-day one. Same ref, same `now` — only the option differs.
    let source = SyntheticSource::for_graph(vec![
        GraphCommit {
            id: "tip".to_string(),
            parents: vec!["root".to_string()],
            title: "Old remote tip".to_string(),
            time: TEST_NOW - 20 * DAY,
            refs: vec!["origin/feature".to_string()],
        },
        GraphCommit {
            id: "root".to_string(),
            parents: vec![],
            title: "Old remote root".to_string(),
            time: TEST_NOW - 20 * DAY - 1,
            refs: vec![],
        },
    ]);

    let default_window = Pane::load(
        &source,
        &GraphOptions {
            now: TEST_NOW,
            stale_after_days: 14,
        },
    );
    assert!(
        default_window.rows().is_empty(),
        "20 days is past the default 14-day window: the branch is out"
    );

    let wide_window = Pane::load(
        &source,
        &GraphOptions {
            now: TEST_NOW,
            stale_after_days: 30,
        },
    );
    assert_eq!(
        wide_window.rows().len(),
        2,
        "within a 30-day window the same remote branch is drawn"
    );
}

#[test]
fn the_pane_believes_the_now_it_is_given_not_the_wall_clock() {
    // The remote tip was updated exactly when the caller says "now" —
    // whatever the real clock says, the pane must trust the options.
    // (An implementation that stamps the wall clock itself sees a ~weeks-old
    // tip here and drops the ref; trusting `options.now` keeps it.)
    let source = SyntheticSource::for_graph(vec![
        GraphCommit {
            id: "tip".to_string(),
            parents: vec!["root".to_string()],
            title: "Just updated".to_string(),
            time: TEST_NOW,
            refs: vec!["origin/feature".to_string()],
        },
        GraphCommit {
            id: "root".to_string(),
            parents: vec![],
            title: "Old remote root".to_string(),
            time: TEST_NOW - 1,
            refs: vec![],
        },
    ]);

    let pane = Pane::load(
        &source,
        &GraphOptions {
            now: TEST_NOW,
            stale_after_days: 14,
        },
    );
    assert_eq!(
        pane.rows().len(),
        2,
        "the caller's now is the truth: a tip updated this second is fresh"
    );
}
