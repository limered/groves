pub mod commit_graph;
pub mod repository;

pub use crate::commit_graph::layout::Edge;
pub use crate::commit_graph::presentation::{HeadMarker, Pane, Pill, PillStyle, Row};
pub use crate::commit_graph::selection::is_stale;
pub use crate::repository::CommitSource;
pub use crate::repository::model::{Commit, GitRef, RefKind, RefSnapshot};

pub const ROW_HEIGHT_PX: f32 = 22.0;
pub const LANE_WIDTH_PX: f32 = 14.0;
