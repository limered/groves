pub mod layout;
pub mod presentation;
pub mod selection;

use crate::CommitSource;
pub use layout::Edge;
pub use presentation::{HeadMarker, Pane, Pill, PillStyle, Row};

pub struct GraphOptions {
    pub now: i64,
    pub stale_after_days: i64,
}

pub fn build(source: &impl CommitSource, options: &GraphOptions) -> Pane {
    Pane::load(source, options)
}
