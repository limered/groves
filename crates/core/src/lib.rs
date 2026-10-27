use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

// Emil owns this file — ticket 01 will add `CommitSource`, `Commit`, `Row` and `Pane` here.
pub const ROW_HEIGHT_PX: f32 = 22.0;
pub const LANE_WIDTH_PX: f32 = 14.0;

pub const DAY_SECONDS: i64 = 24 * 60 * 60;

pub fn is_stale(tip_time: i64, now: i64, stale_after_days: i64) -> bool {
    now - tip_time > stale_after_days * DAY_SECONDS
}

#[derive(Clone)]
pub struct Commit {
    pub id: String,
    pub parents: Vec<String>,
    pub time: i64,
    pub title: String,
    pub refs: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum RefKind {
    Local,
    Remote { remote: String },
    Tag,
    WorktreeHead,
}

#[derive(Debug, Clone)]
pub struct GitRef {
    pub name: String,
    pub target: String,
    pub kind: RefKind,
    pub tip_time: i64,
    pub upstream: Option<String>,
}

#[derive(Default)]
pub struct RefSnapshot {
    refs: Vec<GitRef>,
}

impl RefSnapshot {
    pub fn new(refs: Vec<GitRef>) -> Self {
        Self { refs }
    }
    pub fn refs(&self) -> &[GitRef] {
        &self.refs
    }
}

#[derive(Clone, Debug)]
pub struct Row {
    pub id: String,
    pub short: String,
    pub title: String,
    pub lane: usize,
    pub is_merge: bool,
}
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Edge {
    pub child_row: usize,
    pub parent_row: usize,
    pub child_lane: usize,
    pub parent_lane: usize,
    pub is_merge: bool,
}

#[derive(Debug)]
pub struct Pane {
    rows: Vec<Row>,
    edges: Vec<Edge>,
}

impl Pane {
    pub fn load(source: &impl CommitSource) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let commits = source.commits();
        let mut placed: Vec<bool> = Vec::new();
        let mut lanes: Vec<usize> = Vec::new();
        let mut row_ids: Vec<usize> = Vec::new();
        let mut child_count: Vec<usize> = Vec::new();

        let mut rows: Vec<Row> = Vec::new();
        let mut edges = Vec::new();

        // filter refs for the tips-commits
        let mut not_stale_ref_ids: HashSet<String> = source
            .ref_snapshot()
            .refs()
            .iter()
            .filter(|r| is_stale(r.tip_time, now, 14))
            .map(|r| r.target.clone())
            .collect();

        // index commit-id to index
        let mut commit_index: HashMap<String, usize> = HashMap::new();
        for (i, c) in commits.iter().enumerate() {
            commit_index.insert(c.id.clone(), i);
            placed.push(false);
            lanes.push(1);
            child_count.push(0);
        }

        // add only children of the tips into collection
        let mut work_queue: Vec<usize> = not_stale_ref_ids
            .iter()
            .map(|id| commit_index.get(id).unwrap().clone())
            .collect();
        while !work_queue.is_empty() {
            let c_id = work_queue.pop().unwrap();
            let parents = commits[c_id].parents.clone();
            for p in &parents {
                not_stale_ref_ids.insert(p.clone());
                let p_id = commit_index.get(p).unwrap();
                work_queue.push(*p_id);
            }
        }

        // TODO: filter full commits to the actual list

        // collect children per commit
        for c in &commits {
            for p in &c.parents {
                let Some(p_id) = commit_index.get(p) else {
                    continue;
                };
                child_count[*p_id] += 1;
            }
        }

        let mut head_index: usize = 0; // fallback if head can't be found
        // Rows
        let mut queue: Vec<usize> = (0..commits.len())
            .filter(|i| child_count[*i] == 0)
            .collect();

        while !queue.is_empty() {
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
            let mut expected_ids: Vec<usize> = Vec::new();
            for (i, index) in reserved_lanes.iter().enumerate() {
                if index.eq(&((*id) as i32)) {
                    expected_ids.push(i);
                }
            }
            if expected_ids.is_empty() {
                let free_lane = match reserved_lanes.iter().position(|i| *i == -1) {
                    None => reserved_lanes.len(),
                    Some(i) => i,
                };
                lanes[*id] = free_lane;
                placed[*id] = true;
                for p in &commits[*id].parents {
                    let Some(p_id) = commit_index.get(p) else {
                        continue;
                    };
                    if reserved_lanes.contains(&(*p_id as i32)) {
                        continue;
                    }
                    Self::insert_at_free_position(&mut reserved_lanes, p_id);
                }
                continue;
            }
            let Some(smallest_id) = expected_ids.iter().min() else {
                continue;
            };
            lanes[*id] = *smallest_id;
            placed[*id] = true;
            // rmove all expected (split)
            for id in &expected_ids {
                reserved_lanes[*id] = -1;
            }
            let parents = &commits[*id].parents;
            if parents.is_empty() {
                continue;
            }
            // replace deleted position with first parent
            let Some(p_id) = commit_index.get(&parents[0]) else {
                continue;
            };
            reserved_lanes[*smallest_id] = *p_id as i32;

            // add other parents on free slots (merge)
            for parent in parents.iter().skip(1) {
                let Some(p_id) = commit_index.get(parent) else {
                    continue;
                };
                if reserved_lanes.contains(&(*p_id as i32)) {
                    continue;
                }
                Self::insert_at_free_position(&mut reserved_lanes, p_id);
            }
        }

        // instances
        for (i, index) in row_ids.iter().enumerate() {
            if placed[*index] {
                let c = &commits[*index];
                rows.push(Row {
                    id: c.id.clone(),
                    short: c.id.get(..7).unwrap_or(&c.id).to_string(),
                    title: c.title.clone(),
                    lane: lanes[*index],
                    is_merge: c.parents.len() > 1,
                });
                for parent in &c.parents {
                    let Some(p_index) = commit_index.get(parent) else {
                        continue;
                    };
                    let Some(p_row) = row_ids.iter().position(|i| i.eq(p_index)) else {
                        continue;
                    };
                    let edge = Edge {
                        child_lane: lanes[*index],
                        child_row: i,
                        parent_lane: lanes[*p_index],
                        parent_row: p_row,
                        is_merge: c.parents.len() > 1,
                    };
                    edges.push(edge);
                }
            }
        }

        Pane { rows, edges }
    }

    fn insert_at_free_position(reserved_lanes: &mut Vec<i32>, p_id: &usize) {
        match reserved_lanes.iter().position(|i| *i == -1) {
            None => reserved_lanes.push(*p_id as i32),
            Some(i) => {
                reserved_lanes[i] = *p_id as i32;
            }
        }
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
    fn ref_snapshot(&self) -> &RefSnapshot;
}
