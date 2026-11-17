#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Edge {
    pub child_row: usize,
    pub parent_row: usize,
    pub child_lane: usize,
    pub parent_lane: usize,
    pub is_first_parent: bool,
}