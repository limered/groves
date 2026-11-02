//! Ticket 03b red tests: `Edge` must say which claim rides the child's own
//! line. Owned by the mentor; Emil makes them pass in `crates/core`.
//!
//! Today an edge only knows `is_merge` — "the child is a merge". That's why
//! the develop tip's first-parent edge on LD41 bent away into the master
//! merge curve's lane and the develop line vanished. git's own art (measured
//! on the fixture shape) routes the first parent and later parents
//! differently, and the renderer can't tell them apart from the four
//! span fields alone, so the edge data names it.

use groves_core::{GitRef, Pane, RefKind};
use groves_testkit::{GraphCommit, SyntheticSource};

fn two_merge_shape() -> Pane {
    let source = SyntheticSource::for_graph_with_refs(
        vec![
            GraphCommit {
                id: "m2".into(),
                parents: vec!["s".into(), "rel".into()],
                title: "m2".into(),
                time: 40,
                refs: vec![],
            },
            GraphCommit {
                id: "d1".into(),
                parents: vec!["c1".into(), "rel".into()],
                title: "d1".into(),
                time: 39,
                refs: vec![],
            },
            GraphCommit {
                id: "rel".into(),
                parents: vec!["c1".into()],
                title: "rel".into(),
                time: 38,
                refs: vec![],
            },
            GraphCommit {
                id: "c1".into(),
                parents: vec!["s".into()],
                title: "c1".into(),
                time: 37,
                refs: vec![],
            },
            GraphCommit {
                id: "s".into(),
                parents: vec!["root".into()],
                title: "s".into(),
                time: 30,
                refs: vec![],
            },
            GraphCommit {
                id: "root".into(),
                parents: vec![],
                title: "root".into(),
                time: 10,
                refs: vec![],
            },
        ],
        vec![
            GitRef {
                name: "HEAD".into(),
                target: "m2".into(),
                kind: RefKind::WorktreeHead,
                tip_time: 40,
                upstream: None,
            },
        ],
    );
    Pane::load(&source)
}

fn edge_of<'a>(pane: &'a Pane, child_row: usize, parent_row: usize) -> &'a groves_core::Edge {
    pane.edges()
        .iter()
        .find(|e| e.child_row == child_row && e.parent_row == parent_row)
        .unwrap_or_else(|| panic!("edge {child_row}→{parent_row} must exist"))
}

#[test]
fn first_parent_edges_flagged_where_the_line_continues() {
    let pane = two_merge_shape();

    // m2's parents are `[s, rel]`: the trunk line continues into s…
    assert!(edge_of(&pane, 0, 4).is_first_parent, "m2→s is m2's first parent");
    // …and the release spur departs as a later claim.
    assert!(
        !edge_of(&pane, 0, 2).is_first_parent,
        "m2→rel is a later claim, its line will bend right below the merge"
    );

    // d1 (a merge!) continues its OWN line into c1 — this is the flag the
    // develop-line fix needs: the child being a merge must not decide this.
    assert!(
        edge_of(&pane, 1, 3).is_first_parent,
        "d1→c1 is d1's first parent even though d1 is a merge"
    );
    assert!(!edge_of(&pane, 1, 2).is_first_parent, "d1→rel is a later claim");

    // Single-parent children always continue their line.
    assert!(edge_of(&pane, 2, 3).is_first_parent, "rel→c1");
    assert!(edge_of(&pane, 3, 4).is_first_parent, "c1→s");
    assert!(edge_of(&pane, 4, 5).is_first_parent, "s→root");
}
