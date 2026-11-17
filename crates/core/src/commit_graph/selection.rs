use crate::commit_graph::Pane;
use crate::{Commit, GitRef, RefKind};
use std::collections::{HashMap, HashSet};

pub const DAY_SECONDS: i64 = 24 * 60 * 60;

pub fn is_stale(tip_time: i64, now: i64, stale_after_days: i64) -> bool {
    now - tip_time > stale_after_days * DAY_SECONDS
}

pub fn should_keep_ref(
    branch: &GitRef,
    all_refs: &[GitRef],
    now: i64,
    stale_after_days: i64,
) -> bool {
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
    !is_stale(branch.tip_time, now, stale_after_days)
}

pub fn select_visible_history(
    refs: &[GitRef],
    commits: Vec<Commit>,
    now: i64,
    stale_after_days: i64,
) -> Vec<Commit> {
    let commit_index: HashMap<String, usize> = Pane::build_commit_index(&commits);

    let mut not_stale_ref_ids: HashSet<String> = refs
        .iter()
        .filter(|r| should_keep_ref(r, refs, now, stale_after_days))
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
