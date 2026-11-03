use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

// Emil owns this file — ticket 01 will add `CommitSource`, `Commit`, `Row` and `Pane` here.
pub const ROW_HEIGHT_PX: f32 = 22.0;
pub const LANE_WIDTH_PX: f32 = 14.0;

pub const DAY_SECONDS: i64 = 24 * 60 * 60;

pub fn is_stale(tip_time: i64, now: i64, stale_after_days: i64) -> bool {
    now - tip_time > stale_after_days * DAY_SECONDS
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pill {
    pub name: String,
    pub style: PillStyle,
    pub plus_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PillStyle {
    Filled,
    Outline,
    Synced,
}

#[derive(Clone, Debug)]
pub struct Commit {
    pub id: String,
    pub parents: Vec<String>,
    pub time: i64,
    pub title: String,
    pub refs: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
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
    pub pill: Option<Pill>,
}
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Edge {
    pub child_row: usize,
    pub parent_row: usize,
    pub child_lane: usize,
    pub parent_lane: usize,
    pub is_first_parent: bool,
}

#[derive(Debug)]
pub struct Pane {
    rows: Vec<Row>,
    edges: Vec<Edge>,
}

impl Pane {
    pub fn empty() -> Self {
        Self {
            rows: Vec::new(),
            edges: Vec::new(),
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
    pub fn load(source: &impl CommitSource) -> Self {
        let commits = source.commits();
        let mut rows: Vec<Row> = Vec::new();
        let mut edges = Vec::new();

        let Some(head_ref) = source
            .ref_snapshot()
            .refs()
            .iter()
            .find(|r| r.name == "HEAD")
        else {
            return Pane::empty();
        };

        let all_refs = source.ref_snapshot().refs();
        let filtered_commits = Self::filter_commits(all_refs, commits);
        let filtered_commit_index = Pane::build_commit_index(&filtered_commits);

        let row_ids = Self::calculate_row_ids(&filtered_commits, &filtered_commit_index);
        let (placed, lanes) = Self::calculate_lane_ids(
            &head_ref,
            &filtered_commits,
            &filtered_commit_index,
            &row_ids,
        );

        let mut commit_to_ref: HashMap<String, Vec<GitRef>> = HashMap::new();
        for r in all_refs {
            let t = r.target.clone();
            commit_to_ref.entry(t.clone().to_owned()).or_default().push(r.clone());
        }

        for (child_row, index) in row_ids.iter().enumerate() {
            if placed[*index] {
                let c = &filtered_commits[*index];
                let commit_refs = commit_to_ref.entry(c.id.clone()).or_default();
                let mut pill: Option<Pill> = None;
                if !commit_refs.is_empty() {
                    let len = commit_refs.len().saturating_sub(1);
                    let visible_ref = commit_refs[0].name.clone();
                    let pill_type = PillStyle::Filled;
                    pill = Some(Pill{
                        name: visible_ref,
                        plus_count: len,
                        style: pill_type,
                    });
                }
                
                rows.push(Row {
                    id: c.id.clone(),
                    short: c.id.get(..7).unwrap_or(&c.id).to_string(),
                    title: c.title.clone(),
                    lane: lanes[*index],
                    is_merge: c.parents.len() > 1,
                    pill,
                });
                for (parent_index, parent) in c.parents.iter().enumerate() {
                    let Some(p_index) = filtered_commit_index.get(parent) else {
                        continue;
                    };
                    let Some(parent_row) = row_ids
                        .iter()
                        .position(|inner_row_index| inner_row_index.eq(p_index))
                    else {
                        continue;
                    };
                    let edge = Edge {
                        child_lane: lanes[*index],
                        child_row,
                        parent_lane: lanes[*p_index],
                        parent_row,
                        is_first_parent: parent_index == 0,
                    };
                    edges.push(edge);
                }
            }
        }

        Pane { rows, edges }
    }

    fn calculate_lane_ids(
        head_ref: &&GitRef,
        filtered_commits: &[Commit],
        filtered_commit_index: &HashMap<String, usize>,
        row_ids: &Vec<usize>,
    ) -> (Vec<bool>, Vec<usize>) {
        let head_index = match filtered_commit_index.get(&head_ref.target) {
            Some(i) => *i,
            None => 0,
        };
        let mut placed: Vec<bool> = vec![false; filtered_commits.len()];
        let mut lanes: Vec<usize> = vec![0; filtered_commits.len()];
        let mut reserved_lanes: Vec<Option<usize>> = Vec::new();
        reserved_lanes.push(Some(head_index));
        for id in row_ids {
            let mut expected_ids: Vec<usize> = Vec::new();
            for (i, index) in reserved_lanes.iter().enumerate() {
                if index.eq(&Some(*id)) {
                    expected_ids.push(i);
                }
            }
            if expected_ids.is_empty() {
                let free_lane = match reserved_lanes.iter().position(|i| i.is_none()) {
                    None => reserved_lanes.len(),
                    Some(i) => i,
                };
                lanes[*id] = free_lane;
                placed[*id] = true;
                for p in &filtered_commits[*id].parents {
                    let Some(p_id) = filtered_commit_index.get(p) else {
                        continue;
                    };
                    if reserved_lanes.contains(&Some(*p_id)) {
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
            // move all expected (split)
            for id in &expected_ids {
                reserved_lanes[*id] = None;
            }
            let parents = &filtered_commits[*id].parents;
            if parents.is_empty() {
                continue;
            }
            // replace deleted position with first parent
            let Some(p_id) = filtered_commit_index.get(&parents[0]) else {
                continue;
            };
            reserved_lanes[*smallest_id] = Some(*p_id);

            // add other parents on free slots (merge)
            for parent in parents.iter().skip(1) {
                let Some(p_id) = filtered_commit_index.get(parent) else {
                    continue;
                };
                if reserved_lanes.contains(&Some(*p_id)) {
                    continue;
                }
                Self::insert_at_free_position(&mut reserved_lanes, p_id);
            }
        }
        (placed, lanes)
    }

    fn calculate_row_ids(
        filtered_commits: &Vec<Commit>,
        filtered_commit_index: &HashMap<String, usize>,
    ) -> Vec<usize> {
        let mut row_ids: Vec<usize> = Vec::new();
        let mut child_count: Vec<usize> = vec![0; filtered_commits.len()];

        for c in filtered_commits {
            for p in &c.parents {
                let Some(p_id) = filtered_commit_index.get(p) else {
                    continue;
                };
                child_count[*p_id] += 1;
            }
        }

        let mut queue: Vec<usize> = (0..filtered_commits.len())
            .filter(|i| child_count[*i] == 0)
            .collect();

        while !queue.is_empty() {
            queue.sort_by(|a, b| filtered_commits[*a].time.cmp(&filtered_commits[*b].time));
            let Some(c) = queue.pop() else {
                break;
            };
            row_ids.push(c);
            for p in filtered_commits[c].parents.clone() {
                let Some(p_id) = filtered_commit_index.get(&p) else {
                    continue;
                };
                child_count[*p_id] -= 1;
                if child_count[*p_id] == 0 {
                    queue.push(*p_id);
                }
            }
        }
        row_ids
    }

    fn insert_at_free_position(reserved_lanes: &mut Vec<Option<usize>>, p_id: &usize) {
        match reserved_lanes.iter().position(|i| i.is_none()) {
            None => reserved_lanes.push(Some(*p_id)),
            Some(i) => {
                reserved_lanes[i] = Some(*p_id);
            }
        }
    }

    fn stamp_now() -> i64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    fn build_commit_index(commits: &[Commit]) -> HashMap<String, usize> {
        commits
            .iter()
            .enumerate()
            .map(|(i, c)| (c.id.clone(), i))
            .collect()
    }

    fn should_keep(branch: &GitRef, all_refs: &[GitRef], now: i64) -> bool {
        if branch.kind == RefKind::Tag {
            return false;
        }
        if branch.kind == RefKind::WorktreeHead || branch.kind == RefKind::Local {
            return true;
        }
        // only remotes left
        if branch.name.contains("HEAD") {
            return true;
        }
        // only remotes without HEAD
        if all_refs
            .iter()
            .find(|r| match &r.upstream {
                Some(upstream) => upstream.eq(&branch.name),
                None => false,
            })
            .is_some()
        {
            return true;
        }
        // Only remotes without head and no local branch
        !is_stale(branch.tip_time, now, 14)
    }

    fn filter_commits(refs: &[GitRef], commits: Vec<Commit>) -> Vec<Commit> {
        let now = Self::stamp_now();
        let commit_index: HashMap<String, usize> = Pane::build_commit_index(&commits);

        let mut not_stale_ref_ids: HashSet<String> = refs
            .iter()
            .filter(|r| Pane::should_keep(r, refs, now))
            .map(|r| r.target.clone())
            .collect();

        let mut work_queue: Vec<usize> = not_stale_ref_ids
            .iter()
            .map(|id| *commit_index.get(id).unwrap())
            .collect();
        while let Some(c_id) = work_queue.pop() {
            let parents = commits[c_id].parents.clone();
            for p in &parents {
                let p_id = commit_index.get(p).unwrap();
                if not_stale_ref_ids.insert(p.clone()) {
                    work_queue.push(*p_id);
                }
            }
        }
        commits
            .into_iter()
            .filter(|c| not_stale_ref_ids.contains(&c.id))
            .collect()
    }
}

pub trait CommitSource {
    fn commits(&self) -> Vec<Commit>;
    fn ref_snapshot(&self) -> &RefSnapshot;
}
