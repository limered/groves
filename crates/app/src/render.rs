//! Pure drawing geometry for a pane: edge routing, widths, coordinates.
//!
//! Ticket 03b: `route_edge` — turning a pane edge span into per-row-gap
//! waypoints — goes here in this module.

use groves_core::Edge;

pub fn route_edge(edge: &Edge) -> Vec<(usize, usize)> {
    if edge.parent_lane == edge.child_lane {
        return vec![
            (edge.child_row, edge.child_lane),
            (edge.parent_row, edge.parent_lane),
        ];
    }
    let row_diff = edge.parent_row.saturating_sub(edge.child_row);
    let mut points: Vec<(usize, usize)> = vec![(edge.child_row, edge.child_lane)];
    if edge.is_first_parent && row_diff > 1 {
        points.push((edge.parent_row - 1, edge.child_lane));
    } else if row_diff > 1 {
        points.push((edge.child_row + 1, edge.parent_lane));
    }

    points.push((edge.parent_row, edge.parent_lane));
    points
}
