use core::fmt;

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
        let mut rows: Vec<Row> = Vec::new();
        let mut last_parent = "".to_string();

        for commit in source.commits() {
            let mut row = Row {
                id: commit.id.clone(),
                short: commit.id.get(..7).unwrap_or(&commit.id).to_string(),
                title: commit.title.clone(),
                lane: 0,
            };
            if last_parent.eq(&commit.id) || last_parent.eq("") {
                if commit.parents.len() > 0 {
                    last_parent = commit.parents[0].clone();
                }
            } else {
                row.lane = 1;
            }
            rows.push(row);
        }

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
        match &self.rows.iter().max_by(|a, b| a.lane.cmp(&b.lane)) {
            None => 0,
            Some(i) => i.lane + 1,
        }
    }
    pub fn graph_width_px(&self) -> f32 {
        self.lane_count() as f32 * LANE_WIDTH_PX
    }
}

pub trait CommitSource {
    fn commits(&self) -> Vec<Commit>;
}
