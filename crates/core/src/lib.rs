pub mod commit_graph;
pub mod repository;

pub use crate::commit_graph::selection::is_stale;
pub use crate::repository::model::{Commit, GitRef, RefKind, RefSnapshot};
pub use crate::repository::CommitSource;
use crate::commit_graph::selection::select_visible_history;

use std::collections::HashMap;
use std::time::SystemTime;

pub const ROW_HEIGHT_PX: f32 = 22.0;
pub const LANE_WIDTH_PX: f32 = 14.0;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeadMarker {
    pub name: String,
    pub is_main: bool,
}

#[derive(Clone, Debug)]
pub struct Row {
    pub id: String,
    pub short: String,
    pub title: String,
    pub lane: usize,
    pub is_merge: bool,
    pub pill: Option<Pill>,
    pub head_markers: Vec<HeadMarker>,
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
    lane_count: usize,
    rows: Vec<Row>,
    edges: Vec<Edge>,
}

impl Pane {
    pub fn empty() -> Self {
        Self {
            lane_count: 0,
            rows: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn head_jump_rows(&self) -> Vec<i32> {
        let mut heads: Vec<(i32, HeadMarker)> = Vec::new();
        for (i, hms) in self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| !row.head_markers.is_empty())
            .map(|(i, row)| (i as i32, row.head_markers.clone()))
        {
            for hm in hms {
                heads.push((i, hm));
            }
        }

        heads.sort_by_key(|a| !a.1.is_main);
        let mut only_rows: Vec<i32> = heads.iter().map(|(i, _)| *i).collect();
        only_rows.dedup();
        only_rows
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }
    pub fn lane_count(&self) -> usize {
        self.lane_count
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
            .find(|r| r.kind == RefKind::WorktreeHead)
        else {
            return Pane::empty();
        };

        let all_refs = source.ref_snapshot().refs();
        let filtered_commits = select_visible_history(all_refs, commits);
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
            commit_to_ref
                .entry(t.clone().to_owned())
                .or_default()
                .push(r.clone());
        }

        let mut lane_count = 0;
        for (child_row, index) in row_ids.iter().enumerate() {
            if placed[*index] {
                let c = &filtered_commits[*index];
                let mut pill: Option<Pill> = None;
                let mut head_markers: Vec<HeadMarker> = Vec::new();
                if let Some(commit_refs) = commit_to_ref.get(&c.id) {
                    pill = Self::build_pill(commit_refs);
                    head_markers = commit_refs
                        .iter()
                        .filter(|r| r.kind == RefKind::WorktreeHead)
                        .map(|r| HeadMarker {
                            name: r.name.clone(),
                            is_main: r == head_ref,
                        })
                        .collect();
                }

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
                rows.push(Row {
                    id: c.id.clone(),
                    short: c.id.get(..7).unwrap_or(&c.id).to_string(),
                    title: c.title.clone(),
                    lane: lanes[*index],
                    is_merge: c.parents.len() > 1,
                    pill,
                    head_markers,
                });
                lane_count = lane_count.max(lanes[*index] + 1);
            }
        }

        Pane {
            lane_count,
            rows,
            edges,
        }
    }

    fn build_pill(commit_refs: &[GitRef]) -> Option<Pill> {
        let mut pill: Option<Pill> = None;
        if !commit_refs.is_empty() {
            // sort name first, then by kind
            let mut filtered: Vec<&GitRef> = commit_refs
                .iter()
                .filter(|r| {
                    r.kind == RefKind::Local || matches!(r.kind, RefKind::Remote { remote: _ })
                })
                .filter(|r| !r.name.ends_with("/HEAD"))
                .collect();

            if filtered.is_empty() {
                return None;
            }

            filtered.sort_by_key(|r| (r.kind.clone(), r.name.clone()));

            // derive style
            let visible_ref = filtered[0];
            let mut style = if visible_ref.kind == RefKind::Local {
                PillStyle::Filled
            } else {
                PillStyle::Outline
            };

            if let Some(upstream) = &visible_ref.upstream
                && let Some(upstream_id) = filtered.iter().position(|r| r.name.eq(upstream))
            {
                style = PillStyle::Synced;
                filtered.remove(upstream_id);
            }

            let plus_count = filtered.len().saturating_sub(1);
            pill = Some(Pill {
                name: visible_ref.name.clone(),
                plus_count,
                style,
            });
        }
        pill
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
}
