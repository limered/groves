//! Ticket 02 red tests: `core::Pane` draws a real commit graph — temporal
//! topological rows plus straight lanes with HEAD/main seeded leftmost.
//! Owned by the mentor. Emil makes these pass by changing `crates/core`.
//!
//! Expected facade (mentor's proposal — Emil designs the internals):
//! ```rust,ignore
//! pub struct Commit {
//!     pub id: String,
//!     pub parents: Vec<String>, // ids of parents, empty for a root
//!     pub time: i64,            // committer timestamp, seconds
//!     pub title: String,
//!     pub refs: Vec<String>,    // ref names on this commit, e.g. "HEAD", "main"
//! }
//! pub struct Row {
//!     pub id: String,    // full id of the commit on this row
//!     pub short: String, // 7-char short SHA, as before
//!     pub title: String,
//!     pub lane: usize,   // straight-lane column of this row's node
//! }
//! pub struct Edge {
//!     pub child_row: usize,
//!     pub parent_row: usize,
//!     pub child_lane: usize,
//!     pub parent_lane: usize,
//! }
//! impl Pane {
//!     pub fn rows(&self) -> &[Row];       // as before, now in graph order
//!     pub fn edges(&self) -> &[Edge];
//!     pub fn lane_count(&self) -> usize;  // natural graph width in lanes
//!     pub fn graph_width_px(&self) -> f32; // lane_count * LANE_WIDTH_PX
//! }
//! pub const ROW_HEIGHT_PX: f32 = 22.0;
//! pub const LANE_WIDTH_PX: f32 = 14.0;
//! ```
//!
//! What the tests assert is what a user would notice: which commit sits on
//! which row, which lane each row's node sits in, how wide the graph column
//! is — never internal DAG storage.

use groves_core::{LANE_WIDTH_PX, Pane, ROW_HEIGHT_PX};
use groves_testkit::SyntheticSource;
use std::collections::HashMap;

fn lanes_by_id(pane: &Pane) -> HashMap<String, usize> {
    pane.rows()
        .iter()
        .map(|row| (row.id.clone(), row.lane))
        .collect()
}

fn row_of(pane: &Pane, id: &str) -> usize {
    pane.rows()
        .iter()
        .position(|row| row.id == id)
        .unwrap_or_else(|| panic!("commit {id} must have a row"))
}

#[test]
fn children_appear_above_parents_in_a_deterministic_order() {
    let pane = Pane::load(&SyntheticSource::branch_and_merge());

    // Every parent sits on a lower row than its children.
    assert!(row_of(&pane, "m3") < row_of(&pane, "m1"));
    assert!(row_of(&pane, "m3") < row_of(&pane, "s1"));
    assert!(row_of(&pane, "m1") < row_of(&pane, "m0"));
    assert!(row_of(&pane, "s1") < row_of(&pane, "m0"));

    // Same input, same rows: no HashMap iteration order may leak through.
    let again = Pane::load(&SyntheticSource::branch_and_merge());
    let order: Vec<&str> = pane.rows().iter().map(|r| r.id.as_str()).collect();
    let order_again: Vec<&str> = again.rows().iter().map(|r| r.id.as_str()).collect();
    assert_eq!(order, order_again, "layout must be deterministic");
    assert_eq!(order, vec!["m3", "s1", "m1", "m0"]);
}

#[test]
fn shuffled_input_still_draws_children_above_parents() {
    let ordered = Pane::load(&SyntheticSource::branch_and_merge());
    let shuffled = Pane::load(&SyntheticSource::shuffled_branch_and_merge());

    let order: Vec<&str> = ordered.rows().iter().map(|r| r.id.as_str()).collect();
    let shuffled_order: Vec<&str> = shuffled.rows().iter().map(|r| r.id.as_str()).collect();
    assert_eq!(
        order, shuffled_order,
        "row order follows the graph, not the source listing order"
    );
}

#[test]
fn linear_history_uses_a_single_lane() {
    let pane = Pane::load(&SyntheticSource::linear_three());

    assert_eq!(pane.lane_count(), 1, "a line needs exactly one lane");
    for row in pane.rows() {
        assert_eq!(row.lane, 0, "every node of a line sits in lane 0");
    }
}

#[test]
fn branch_and_merge_reuses_the_freed_lane() {
    let pane = Pane::load(&SyntheticSource::branch_and_merge());
    let lanes = lanes_by_id(&pane);

    assert_eq!(
        pane.lane_count(),
        2,
        "one side branch needs two lanes, not more"
    );
    // The merge tip and the whole main first-parent chain share lane 0 …
    assert_eq!(lanes["m3"], 0);
    assert_eq!(lanes["m1"], 0);
    assert_eq!(lanes["m0"], 0);
    // … while the side commit owns the second lane, freed again by the merge.
    assert_eq!(lanes["s1"], 1);
}

#[test]
fn head_first_parent_chain_owns_lane_zero() {
    // The side tip is newer, so recency alone would seed it leftmost.
    // HEAD/main seeding must win: the user follows HEAD down lane 0.
    let pane = Pane::load(&SyntheticSource::head_newer_side_branch());
    let lanes = lanes_by_id(&pane);

    assert_eq!(lanes["m2"], 0, "HEAD tip sits in lane 0");
    assert_eq!(
        lanes["m1"], 0,
        "HEAD's first-parent chain never leaves lane 0"
    );
    assert_eq!(lanes["m0"], 0);
    assert_eq!(lanes["s1"], 1, "the newer side tip does not steal lane 0");
}

#[test]
fn lanes_never_shift_sideways_along_a_branch() {
    let pane = Pane::load(&SyntheticSource::branch_and_merge());
    let lanes = lanes_by_id(&pane);

    // Each commit's node sits in the lane its branch owns from tip to root:
    // no sideways drift mid-branch.
    for id in ["m3", "m1", "m0"] {
        assert_eq!(lanes[id], 0, "{id} drifts off the main lane");
    }
    assert_eq!(lanes["s1"], 1, "s1 drifts off the side lane");

    // The merge's edges arrive from the lanes the parents actually sit in.
    let merge_row = row_of(&pane, "m3");
    let mut targets: Vec<(usize, usize)> = pane
        .edges()
        .iter()
        .filter(|e| e.child_row == merge_row)
        .map(|e| (e.parent_row, e.parent_lane))
        .collect();
    targets.sort();
    assert_eq!(
        targets,
        vec![(row_of(&pane, "m1"), 0), (row_of(&pane, "s1"), 1)],
        "the merge tip visibly connects to both parents' lanes"
    );
}

#[test]
fn graph_column_is_lanes_wide_and_never_overlaps_text() {
    assert_eq!(ROW_HEIGHT_PX, 22.0, "spec: dense but legible 22 px rows");
    assert_eq!(LANE_WIDTH_PX, 14.0, "spec: 14 px lanes");

    let line = Pane::load(&SyntheticSource::linear_three());
    assert_eq!(line.graph_width_px(), 1.0 * LANE_WIDTH_PX);

    let merged = Pane::load(&SyntheticSource::branch_and_merge());
    assert_eq!(merged.graph_width_px(), 2.0 * LANE_WIDTH_PX);

    // The app offsets commit text past this width; the pane reports the
    // width so text can never sit under the graph.
    assert!(merged.graph_width_px() > line.graph_width_px());
}
