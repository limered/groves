//! Ticket 01 red tests: `core::Pane` turns a `CommitSource` into the rows the
//! app draws — newest first, one row per commit, short SHA plus title.
//! Owned by the mentor. Emil makes these pass by changing `crates/core`.

use groves_core::Pane;
use groves_testkit::SyntheticSource;

#[test]
fn pane_lists_rows_newest_first_with_short_sha_and_title() {
    let source = SyntheticSource::three();
    let pane = Pane::load(&source);

    let rows = pane.rows();
    assert_eq!(rows.len(), 3, "one row per commit");

    // Newest first, matching the source order.
    assert_eq!(rows[0].title, "Fix lane reuse");
    assert_eq!(rows[1].title, "Add filter box");
    assert_eq!(rows[2].title, "Initial commit");

    // What the user notices per row: short SHA plus title.
    assert_eq!(rows[0].short, "c3c3c3c");
    assert_eq!(rows[1].short, "b2b2b2b");
    assert_eq!(rows[2].short, "a1a1a1a");
}

#[test]
fn pane_is_empty_without_commits() {
    let source = SyntheticSource::new(vec![]);
    let pane = Pane::load(&source);
    assert!(pane.rows().is_empty());
}
