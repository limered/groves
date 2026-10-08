//! Ticket 02 red tests: `repo::open` reads parents (and times) so the pane
//! can lay out a real graph, on the linear fixture and on a branch+merge one.
//! Owned by the mentor. Emil makes these pass by changing `crates/repo`.

use groves_core::{CommitSource, Pane};
use groves_testkit::{init_branch_fixture_repo, init_fixture_repo};

#[test]
fn linear_fixture_wires_parents_tip_to_root() {
    let repo = groves_repo::open(&init_fixture_repo()).expect("fixture must open");
    let commits = repo.commits();

    assert_eq!(commits.len(), 3, "the linear fixture has three commits");
    // Newest first, each commit naming its parent — what the graph needs.
    assert!(commits[0].parents.len() <= 1);
    assert_eq!(commits[0].parents, vec![commits[1].id.clone()]);
    assert_eq!(commits[1].parents, vec![commits[2].id.clone()]);
    assert!(commits[2].parents.is_empty(), "the root has no parents");

    // Times run newest-first too: the pane's temporal order matches.
    assert!(
        commits[0].time >= commits[1].time && commits[1].time >= commits[2].time,
        "committer times must be readable and newest-first"
    );

    // And the pane draws the real repo as one lane.
    let pane = Pane::load(&repo);
    assert_eq!(pane.lane_count(), 1);
    assert!(pane.rows().iter().all(|row| row.lane == 0));
}

#[test]
fn merge_fixture_reports_both_parents() {
    let repo = groves_repo::open(&init_branch_fixture_repo()).expect("branch fixture must open");
    let commits = repo.commits();

    let merge = commits
        .iter()
        .find(|c| c.parents.len() == 2)
        .expect("the merge tip must report two parents");
    assert_eq!(merge.title, "Merge side branch");

    // Both parents are commits we also walked.
    let ids: Vec<&str> = commits.iter().map(|c| c.id.as_str()).collect();
    for parent in &merge.parents {
        assert!(
            ids.contains(&parent.as_str()),
            "merge parent {parent} must be among the walked commits"
        );
    }

    // The pane draws the merge with two lanes and an edge per parent.
    let pane = Pane::load(&repo);
    assert_eq!(pane.lane_count(), 2);
    let merge_row = pane
        .rows()
        .iter()
        .position(|row| row.id == merge.id)
        .expect("merge commit must have a row");
    let edge_count = pane
        .edges()
        .iter()
        .filter(|e| e.child_row == merge_row)
        .count();
    assert_eq!(
        edge_count, 2,
        "both merge parents must be visibly connected"
    );
}
