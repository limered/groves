//! Ticket 05 red tests: rows carry Worktree HEAD markers (main worktree
//! first, named after its worktree), the first `WorktreeHead` ref seeds
//! lane 0, and the pane exposes the jump-to-HEAD row order (main first,
//! then the others, duplicate rows skipped).
//! Owned by the mentor. Emil makes these pass by changing `crates/core`.

use groves_core::{GitRef, HeadMarker, Pane, RefKind};
use groves_testkit::{GraphCommit, SyntheticSource, TEST_NOW, pane_options};

fn git_ref(name: &str, target: &str, tip_time: i64) -> GitRef {
    GitRef {
        name: name.to_string(),
        target: target.to_string(),
        kind: RefKind::WorktreeHead,
        tip_time,
        upstream: None,
    }
}

fn commit(id: &str, parents: &[&str], time: i64) -> GraphCommit {
    GraphCommit {
        id: id.to_string(),
        parents: parents.iter().map(|parent| (*parent).to_string()).collect(),
        title: id.to_string(),
        time,
        refs: vec![],
    }
}

/// Main worktree `Factory` and secondary `wt-side` share the tip; a
/// detached-style head `c0ffee1` sits one commit down. Rows end up
/// newest-first: m3 (0), m1 (1), m0 (2).
fn tip_shared_source() -> SyntheticSource {
    SyntheticSource::for_graph_with_refs(
        vec![
            commit("m3", &["m1"], 300),
            commit("m1", &["m0"], 200),
            commit("m0", &[], 100),
        ],
        vec![
            git_ref("Factory", "m3", 300),
            git_ref("wt-side", "m3", 300),
            git_ref("c0ffee1", "m1", 200),
        ],
    )
}

#[test]
fn rows_carry_head_markers_named_by_their_worktree() {
    let pane = Pane::load(&tip_shared_source(), &pane_options(TEST_NOW));

    let row = |id: &str| {
        pane.rows()
            .iter()
            .find(|row| row.id == id)
            .unwrap_or_else(|| panic!("row {id} must exist"))
    };

    assert_eq!(
        row("m3").head_markers,
        vec![
            HeadMarker {
                name: "Factory".to_string(),
                is_main: true,
            },
            HeadMarker {
                name: "wt-side".to_string(),
                is_main: false,
            },
        ],
        "both worktrees on the tip: main first, then the secondary"
    );
    assert_eq!(
        row("m1").head_markers,
        vec![HeadMarker {
            name: "c0ffee1".to_string(),
            is_main: false,
        }],
        "the detached-style head marks its own row, never main"
    );
    assert!(
        row("m0").head_markers.is_empty(),
        "rows without a worktree head carry no markers"
    );
}

#[test]
fn first_worktree_head_seeds_lane0_and_is_main() {
    // The FIRST WorktreeHead ref is the main worktree even when another
    // worktree's tip is newer: main's chain owns lane 0, like HEAD did.
    let source = SyntheticSource::for_graph_with_refs(
        vec![
            commit("m3", &["m0"], 400),
            commit("s1", &["m0"], 300),
            commit("m0", &[], 100),
        ],
        vec![git_ref("wt-a", "s1", 300), git_ref("wt-b", "m3", 400)],
    );

    let pane = Pane::load(&source, &pane_options(TEST_NOW));
    let lanes: std::collections::HashMap<String, usize> = pane
        .rows()
        .iter()
        .map(|row| (row.id.clone(), row.lane))
        .collect();

    assert_eq!(lanes["s1"], 0, "the main worktree's chain owns lane 0");
    assert_eq!(
        lanes["m3"], 1,
        "the newer secondary worktree hangs beside it"
    );

    let s1_row = pane
        .rows()
        .iter()
        .find(|row| row.id == "s1")
        .expect("s1 has a row");
    assert_eq!(
        s1_row.head_markers,
        vec![HeadMarker {
            name: "wt-a".to_string(),
            is_main: true,
        }]
    );
    let m3_row = pane
        .rows()
        .iter()
        .find(|row| row.id == "m3")
        .expect("m3 has a row");
    assert_eq!(
        m3_row.head_markers,
        vec![HeadMarker {
            name: "wt-b".to_string(),
            is_main: false,
        }]
    );
}

#[test]
fn head_jump_rows_go_main_first_then_the_others_deduped() {
    let pane = Pane::load(&tip_shared_source(), &pane_options(TEST_NOW));

    assert_eq!(
        pane.head_jump_rows(),
        vec![0, 1],
        "main's row first, the shared secondary row skipped, then c0ffee1's row"
    );
}

#[test]
fn head_jump_rows_follow_worktree_order_not_row_order() {
    // Here the main worktree's head (s1) sits BELOW the secondary's newer
    // tip (m3): jump order must still start at main's row.
    let source = SyntheticSource::for_graph_with_refs(
        vec![
            commit("m3", &["m0"], 400),
            commit("s1", &["m0"], 300),
            commit("m0", &[], 100),
        ],
        vec![git_ref("wt-a", "s1", 300), git_ref("wt-b", "m3", 400)],
    );

    let pane = Pane::load(&source, &pane_options(TEST_NOW));
    assert_eq!(
        pane.head_jump_rows(),
        vec![1, 0],
        "main's row (1) first, then the secondary worktree's row (0)"
    );
}
