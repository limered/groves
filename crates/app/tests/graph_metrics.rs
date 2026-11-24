//! Pixel sizes live in the renderer, not in core: core reports the graph in
//! lanes and rows, `render` owns what those mean in pixels ("the renderer
//! owns pixel dimensions"). Moved here from `core/tests/graph.rs` when the
//! constants left core (spec 24: 22 px rows, 14 px lanes).
//! Owned by the mentor; Emil makes these pass by changing `crates/app`.
//!
//! Expected facade:
//! ```rust,ignore
//! // crates/app/src/render.rs
//! pub const ROW_HEIGHT_PX: f32 = 22.0;
//! pub const LANE_WIDTH_PX: f32 = 14.0;
//! pub fn graph_width_px(lane_count: usize) -> f32; // lane_count * LANE_WIDTH_PX
//! ```

use groves_app::render::{LANE_WIDTH_PX, ROW_HEIGHT_PX, graph_width_px};
use groves_core::Pane;
use groves_testkit::{SyntheticSource, TEST_NOW, pane_options};

#[test]
fn spec_pixel_density_is_22_rows_by_14_lanes() {
    assert_eq!(ROW_HEIGHT_PX, 22.0, "spec: dense but legible 22 px rows");
    assert_eq!(LANE_WIDTH_PX, 14.0, "spec: 14 px lanes");
}

#[test]
fn graph_column_is_lanes_wide_and_never_overlaps_text() {
    let line = Pane::load(&SyntheticSource::linear_three(), &pane_options(TEST_NOW));
    assert_eq!(line.lane_count(), 1);
    assert_eq!(graph_width_px(line.lane_count()), 1.0 * LANE_WIDTH_PX);

    let merged = Pane::load(
        &SyntheticSource::branch_and_merge(),
        &pane_options(TEST_NOW),
    );
    assert_eq!(merged.lane_count(), 2);
    assert_eq!(graph_width_px(merged.lane_count()), 2.0 * LANE_WIDTH_PX);

    // The app offsets commit text past this width; the graph column grows
    // with lanes so text can never sit under the graph.
    assert!(
        graph_width_px(merged.lane_count()) > graph_width_px(line.lane_count()),
        "more lanes must mean a wider graph column"
    );
}

#[test]
fn an_empty_graph_column_has_zero_width() {
    assert_eq!(graph_width_px(0), 0.0);

    let source = SyntheticSource::for_graph_with_refs(vec![], vec![]);
    let pane = Pane::load(&source, &pane_options(TEST_NOW));
    assert_eq!(pane.rows().len(), 0);
    assert_eq!(graph_width_px(pane.lane_count()), 0.0);
}
