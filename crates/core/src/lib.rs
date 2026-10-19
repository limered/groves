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
        let edges = Vec::new();

        let mut commit_index: HashMap<String, usize> = HashMap::new();
        let commits = source.commits();

        let mut head_index: usize = 0; // fallback if head can't be found
        for i in 0..commits.len() {
            let c = &commits[i];
            commit_index.insert(c.id.clone(), i);
            if c.refs.contains(&"HEAD".to_string()) {
                head_index = i;
            }
            placed.push(false);
            lanes.push(1);
        }

        while !placed[head_index] {
            placed[head_index] = true;
            lanes[head_index] = 0;
            row_ids.push(head_index);
            let parent = commits[head_index].parents.first();
            match parent {
                None => break,
                Some(p) => match commit_index.get(p) {
                    None => break,
                    Some(parent_index) => {
                        head_index = *parent_index;
                    }
                },
            }
        }

        let mut queue: Vec<usize> = Vec::new();
        queue.push(0);

        while queue.len() > 0 {
            queue.sort_by(|a, b| commits[*b].time.cmp(&commits[*a].time));
            let Some(c) = queue.pop() else {
                break;
            };
            let mut parent_ids: Vec<usize> = Vec::new();
            for parent_id in commits[c].parents.clone() {
                let Some(parent_index) = commit_index.get(&parent_id) else {
                    continue;
                };
                parent_ids.push(*parent_index);
            }
            if !placed[c] {
                placed[c] = true;
                lanes[c] = 1;
                if let Some(placed_parent_index) = parent_ids.iter().find(|p| placed[**p]) {
                    let Some(row_index) = row_ids.iter().position(|n| n.eq(placed_parent_index))
                    else {
                        break;
                    };
                    row_ids.insert(row_index, c);
                } else {
                    row_ids.push(c);
                }
            }
            queue.append(&mut parent_ids);
        }

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
