use std::collections::HashMap;

// Emil owns this file — ticket 01 will add `CommitSource`, `Commit`, `Row` and `Pane` here.
pub const ROW_HEIGHT_PX: f32 = 22.0;
pub const LANE_WIDTH_PX: f32 = 14.0;

#[derive(Clone)]
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
        let mut placed: Vec<bool> = Vec::new();
        let mut lanes: Vec<usize> = Vec::new();
        let mut row_ids: Vec<usize> = Vec::new();
        let mut occupancy: Vec<bool> = Vec::new();
        let edges = Vec::new();

        let mut commit_index: HashMap<String, usize> = HashMap::new();
        let commits = source.commits();

        let mut head_id: usize = 0; // fallback if head can't be found
        for i in 0..commits.len() {
            let c = &commits[i];
            commit_index.insert(c.id.clone(), i);
            if c.refs.contains(&"HEAD".to_string()) {
                head_id = i;
            }
            placed.push(false);
            lanes.push(1);
        }

        while !placed[head_id] {
            placed[head_id] = true;
            lanes[head_id] = 0;
            row_ids.push(head_id);
            let parent = commits[head_id].parents.first();
            match parent {
                None => break,
                Some(p) => match commit_index.get(p) {
                    None => break,
                    Some(parent) => {
                        head_id = *parent;
                    }
                },
            }
        }
        occupancy.push(true);

        // for i in 0..commits.len() {
        //     if placed[i] {
        //         continue;
        //     }
        //     let c = &commits[i];
        //     println!("{:?}", c.parents);
        // }

        for i in 0..row_ids.len() {
            if placed[row_ids[i]] {
                let c = &commits[row_ids[i]];
                rows.push(Row {
                    id: c.id.clone(),
                    short: c.id.get(..7).unwrap_or(&c.id).to_string(),
                    title: c.title.clone(),
                    lane: lanes[row_ids[i]],
                });
            }
        }

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
