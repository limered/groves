//! Pure drawing geometry for a pane: edge routing, widths, coordinates.
//!
//! Ticket 03b: `route_edge` — turning a pane edge span into per-row-gap
//! waypoints — goes here in this module.

use groves_core::Edge;

pub fn route_edge(edge: &Edge) -> Vec<(usize,usize)> {
    vec![(1usize, 1usize)]
}