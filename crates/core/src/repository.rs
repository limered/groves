pub mod model;
use crate::repository::model::{Commit, RefSnapshot};

pub trait CommitSource {
    fn commits(&self) -> Vec<Commit>;
    fn ref_snapshot(&self) -> &RefSnapshot;
}
