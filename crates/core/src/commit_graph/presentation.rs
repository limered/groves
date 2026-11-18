use crate::commit_graph::layout::{Edge, calculate_lane_ids, calculate_row_ids};
use crate::commit_graph::selection::{build_commit_index, select_visible_history};
use crate::{CommitSource, GitRef, GraphOptions, RefKind};
use std::collections::HashMap;

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
        let mut seen: HashMap<usize, bool> = HashMap::new();
        let mut result: Vec<i32> = Vec::new();

        for is_main in [true, false] {
            for (row_index, row) in self.rows.iter().enumerate() {
                let matches = row
                    .head_markers
                    .iter()
                    .any(|marker| marker.is_main == is_main);

                if matches && seen.insert(row_index, true).is_none() {
                    result.push(row_index as i32);
                }
            }
        }
        result
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
    pub fn load(source: &impl CommitSource, options: &GraphOptions) -> Self {
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
        let filtered_commits =
            select_visible_history(all_refs, commits, options.now, options.stale_after_days);
        let filtered_commit_index = build_commit_index(&filtered_commits);

        let row_ids = calculate_row_ids(&filtered_commits, &filtered_commit_index);
        let (placed, lanes) = calculate_lane_ids(
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

        let mut row_by_commit = vec![None; filtered_commits.len()];

        for (row, &commit_index) in row_ids.iter().enumerate() {
            row_by_commit[commit_index] = Some(row);
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
                    let Some(parent_row) = row_by_commit[*p_index] else {
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
}
