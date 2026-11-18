use std::collections::HashMap;
use crate::{Commit, GitRef};

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Edge {
    pub child_row: usize,
    pub parent_row: usize,
    pub child_lane: usize,
    pub parent_lane: usize,
    pub is_first_parent: bool,
}

pub(super) fn calculate_lane_ids(
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
                insert_at_free_position(&mut reserved_lanes, p_id);
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
            insert_at_free_position(&mut reserved_lanes, p_id);
        }
    }
    (placed, lanes)
}

pub(super) fn calculate_row_ids(
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
