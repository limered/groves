//! Ticket 03b red tests: edges route per row-gap instead of one long span.
//! Owned by the mentor; Emil makes them pass in `crates/app`.
//!
//! A long-span edge threads its curve through intermediate commit rows (on
//! LD41 the develop line visually fused with the master merge curve). The
//! fix: `route_edge` returns the waypoints (row, lane) the drawn line passes
//! through, top to bottom — one waypoint per row-gap where the line turns,
//! so the renderer draws each straight run and each single-gap bend. This
//! matches `git log --graph`'s `|/` art measured on the real repo.

use groves_app::render::route_edge;
use groves_core::{Edge, GitRef, RefKind};

fn edge(
    child_row: usize,
    child_lane: usize,
    parent_row: usize,
    parent_lane: usize,
    is_first_parent: bool,
) -> Edge {
    Edge {
        child_row,
        parent_row,
        child_lane,
        parent_lane,
        is_first_parent,
    }
}

#[test]
fn a_vertical_edge_is_one_straight_run() {
    // Like the shared trunk on LD41: master's tip to its deep first parent,
    // never leaving lane 0.
    let route = route_edge(&edge(0, 0, 4, 0, true));
    assert_eq!(route, vec![(0, 0), (4, 0)]);
}

#[test]
fn first_parent_edge_keeps_lane_through_intermediate_gaps() {
    // The develop line's fix: ride the child's own lane past every
    // intermediate commit row, bend into the parent's lane only across the
    // final gap. Corners, not cells: the ride collapses to one pair, the
    // bend keeps a one-gap pair so the curve can't thread through the
    // middle rows.
    let route = route_edge(&edge(0, 0, 3, 2, true));
    assert_eq!(route, vec![(0, 0), (2, 0), (3, 2)]);
}

#[test]
fn one_row_gap_edges_keep_their_single_bend() {
    // A bend across the only gap it has — today's look, first or later
    // parent. This is the shape the develop tip's merge curve keeps (it must
    // still bend right below the tip).
    assert_eq!(route_edge(&edge(0, 0, 1, 2, true)), vec![(0, 0), (1, 2)]);
    assert_eq!(route_edge(&edge(0, 2, 1, 1, false)), vec![(0, 2), (1, 1)]);
}

#[test]
fn later_parent_edge_bends_right_below_the_child() {
    // A merge's second parent may sit several rows below the merge (the
    // master merge curve into `ad72308`). git draws that departure
    // immediately below the merge tip, then the line rides the parent's
    // lane down to the commit: bend across the first gap, straight after.
    assert_eq!(
        route_edge(&edge(0, 0, 3, 1, false)),
        vec![(0, 0), (1, 1), (3, 1)]
    );
}

#[test]
fn the_two_merges_pane_routes_every_edge_per_row_gap() {
    // LD41's real top shape, already pinned to git's own lanes in core's
    // `two_merges_of_one_spur_follow_gits_own_columns`. Rows 0..5 run
    // m2(0) d1(2) rel(1) c1(1) s(0) root(0). Every edge's route follows
    // from those spans — the develop line's first-parent route is the one
    // this ticket is about.
    use groves_core::Pane;
    use groves_testkit::{GraphCommit, SyntheticSource};

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
            // Mirror ticket 03's fixture: HEAD, master, develop. Without
            // `develop` the default-ref-set filter correctly drops d1 —
            // nothing else reaches the develop tip — and this shape stops
            // being LD41's top region. The filter isn't broken; my first
            // copy of this fixture was.
            GitRef {
                name: "HEAD".into(),
                target: "m2".into(),
                kind: RefKind::WorktreeHead,
                tip_time: 40,
                upstream: None,
            },
            GitRef {
                name: "master".into(),
                target: "m2".into(),
                kind: RefKind::Local,
                tip_time: 40,
                upstream: None,
            },
            GitRef {
                name: "develop".into(),
                target: "d1".into(),
                kind: RefKind::Local,
                tip_time: 39,
                upstream: None,
            },
        ],
    );
    let pane = Pane::load(&source);
    assert_eq!(pane.rows().len(), 6, "the whole shape is on the pane");

    let lane_of = |id: &str, _row: usize| -> usize {
        pane.rows()
            .iter()
            .find(|row| row.id == id)
            .expect("row exists")
            .lane
    };
    for (id, lane, row) in [
        ("m2", 0usize, 0usize),
        ("d1", 2, 1),
        ("rel", 1, 2),
        ("c1", 1, 3),
        ("s", 0, 4),
        ("root", 0, 5),
    ] {
        assert_eq!(lane_of(id, row), lane, "{id} sits in lane {lane}");
    }

    let route_of = |child_row: usize, parent_row: usize| -> Vec<(usize, usize)> {
        let e = pane
            .edges()
            .iter()
            .find(|e| e.child_row == child_row && e.parent_row == parent_row)
            .unwrap_or_else(|| panic!("edge {child_row}→{parent_row} must exist"));
        route_edge(e)
    };

    // m2→s: the trunk rides lane 0 all the way down.
    assert_eq!(route_of(0, 4), vec![(0, 0), (4, 0)]);
    // m2→rel: the master merge curve bends below the merge tip, then rides
    // rel's lane — visually separate from the develop line (lane 2).
    assert_eq!(route_of(0, 2), vec![(0, 0), (1, 1), (2, 1)]);
    // d1→c1: THE develop line — its own lane 2 through rel's row, bend into
    // lane 1 only across the last gap. Never fusing with m2→rel.
    assert_eq!(route_of(1, 3), vec![(1, 2), (2, 2), (3, 1)]);
    // d1→rel: single gap, current bend look.
    assert_eq!(route_of(1, 2), vec![(1, 2), (2, 1)]);
    // rel→c1 and s→root: verticals.
    assert_eq!(route_of(2, 3), vec![(2, 1), (3, 1)]);
    assert_eq!(route_of(4, 5), vec![(4, 0), (5, 0)]);
    // c1→s: the trunk join, one gap, current bend look.
    assert_eq!(route_of(3, 4), vec![(3, 1), (4, 0)]);
}
