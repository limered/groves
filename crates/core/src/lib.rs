// Emil owns this file — ticket 01 will add `CommitSource`, `Commit`, `Row` and `Pane` here.

pub const ROW_HEIGHT_PX: f32 = 22.0;
pub const LANE_WIDTH_PX: f32 = 14.0;

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Commit {
    pub id: String,
    pub parents: Vec<String>,
    pub time: i64,
    pub title: String,
    pub refs: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Row {
    pub id: String,
    pub short: String,
    pub title: String,
    pub lane: usize,
}
#[derive(Debug)]
pub struct Edge {
    pub child_row: usize,
    pub parent_row: usize,
    pub child_lane: usize,
    pub parent_lane: usize,
}

#[derive(Debug)]
pub struct Pane {
    rows: Vec<Row>,
    edges: Vec<Edge>,
}

impl Pane {
    pub fn load(source: &impl CommitSource) -> Self {
        let mut sorted_rows = source.commits();
        sorted_rows.sort_by(|a, b| a.time.cmp(&b.time));

        let rows = sorted_rows
            .into_iter()
            .map(|c| Row {
                id: c.id.clone(),
                short: c.id.get(..7).unwrap_or(&c.id).to_string(),
                title: c.title,
                lane: 0,
            })
            .collect();

        
        let edges = Vec::new();
        Pane { rows, edges }
    }
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }
    pub fn lane_count(&self) -> usize {
        0
    }
    pub fn graph_width_px(&self) -> f32 {
        self.lane_count() as f32 * LANE_WIDTH_PX
    }
}

pub trait CommitSource {
    fn commits(&self) -> Vec<Commit>;
}
