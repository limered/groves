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
        let commits = source.commits();

        let mut rows: Vec<Row> = Vec::new();
        let mut placed: Vec<bool> = Vec::new();
        let mut lanes: Vec<usize> = Vec::new();
        let mut row_ids: Vec<usize> = Vec::new();
        let mut child_count: Vec<usize> = Vec::new();
        let edges = Vec::new();

        // index commit-id to index
        let mut commit_index: HashMap<String, usize> = HashMap::new();
        let mut head_index: usize = 0; // fallback if head can't be found
        for i in 0..commits.len() {
            let c = &commits[i];
            commit_index.insert(c.id.clone(), i);
            if c.refs.contains(&"HEAD".to_string()) {
                head_index = i;
            }
            placed.push(false);
            lanes.push(1);
            child_count.push(0);
        }

        // collect children per commit
        for c in &commits {
            for p in &c.parents {
                let Some(p_id) = commit_index.get(p) else {
                    continue;
                };
                child_count[*p_id] += 1;
            }
        }

        // Rows
        let mut queue: Vec<usize> = (0..commits.len())
            .filter(|i| child_count[*i] == 0)
            .collect();

        while queue.len() > 0 {
            queue.sort_by(|a, b| commits[*a].time.cmp(&commits[*b].time));
            let Some(c) = queue.pop() else {
                break;
            };
            row_ids.push(c);
            for p in commits[c].parents.clone() {
                let Some(p_id) = commit_index.get(&p) else {
                    continue;
                };
                child_count[*p_id] -= 1;
                if child_count[*p_id] == 0 {
                    queue.push(*p_id);
                }
            }
        }

        // lanes
        let mut reserved_lanes: Vec<i32> = Vec::new();
        reserved_lanes.push(head_index as i32);
        for id in &row_ids {
            match reserved_lanes.iter().position(|i| i.eq(&((*id) as i32))) {
                None => {
                    lanes[*id] = match reserved_lanes.iter().position(|i| *i == -1) {
                        None => reserved_lanes.len(),
                        Some(i) => i,
                    };
                    placed[*id] = true;
                    for p in &commits[*id].parents {
                        let Some(p_id) = commit_index.get(p) else {
                            continue;
                        };
                        match reserved_lanes.iter().position(|i| *i == -1) {
                            None => reserved_lanes.push(*p_id as i32),
                            Some(i) => {
                                reserved_lanes[i] = *p_id as i32;
                            }
                        }
                    }
                }
                Some(i) => {
                    lanes[*id] = i;
                    placed[*id] = true;
                    let parents = &commits[*id].parents;
                    if parents.is_empty() {
                        continue;
                    }
                    let Some(p_id) = commit_index.get(&parents[0]) else {
                        continue;
                    };
                    reserved_lanes[i] = *p_id as i32;

                    // merge
                    if parents.len() == 2 {
                        let Some(p_id) = commit_index.get(&parents[1]) else {
                            continue;
                        };
                        match reserved_lanes.iter().position(|i| *i == -1) {
                            None => reserved_lanes.push(*p_id as i32),
                            Some(i) => {
                                reserved_lanes[i] = *p_id as i32;
                            }
                        }
                    }

                    // split
                    let Some(second_i) = reserved_lanes.iter().position(|i| i.eq(&((*id) as i32)))
                    else {
                        continue;
                    };
                    reserved_lanes[second_i] = -1;
                }
            }
        }

        // instatces
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
