use crate::{Commit, CommitSource, GitRef};

pub mod layout;
pub mod presentation;
pub mod selection;

pub use layout::Edge;
pub use presentation::{HeadMarker, Pane, Pill, PillStyle, Row};

pub struct GraphOptions {
    pub now: i64,
    pub stale_after_days: i64,
}

pub fn build(
    source: &impl CommitSource,
    options: &GraphOptions, // todo: add to load, so pane does not need to read now
) -> Pane {
    Pane::load(source)
}
