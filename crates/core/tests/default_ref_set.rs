//! Ticket 03 red tests: the pane lays out only history reachable from the
//! default ref set, while preserving fresh branches and excluding stale/tag
//! only history. Tests are owned by the mentor; Emil changes product code.

use groves_core::{GitRef, Pane, RefKind};
use groves_testkit::{GraphCommit, SyntheticSource};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

const DAY: i64 = 24 * 60 * 60;

fn seconds_ago(days: i64) -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after the Unix epoch")
        .as_secs() as i64;
    now - days * DAY
}

fn git_ref(
    name: &str,
    target: &str,
    kind: RefKind,
    tip_time: i64,
    upstream: Option<&str>,
) -> GitRef {
    GitRef {
        name: name.to_string(),
        target: target.to_string(),
        kind,
        tip_time,
        upstream: upstream.map(str::to_string),
    }
}

fn visible_ids(pane: &Pane) -> HashSet<&str> {
    pane.rows().iter().map(|row| row.id.as_str()).collect()
}

fn commit(id: &str, parents: &[&str], time: i64) -> GraphCommit {
    GraphCommit {
        id: id.to_string(),
        parents: parents.iter().map(|parent| (*parent).to_string()).collect(),
        title: id.to_string(),
        time,
        refs: vec![],
    }
}

#[test]
fn stale_remote_and_tag_only_history_do_not_add_rows_or_lanes() {
    let now = seconds_ago(0);
    let source = SyntheticSource::for_graph_with_refs(
        vec![
            commit("m3", &["m1", "s1"], now - 30),
            commit("s1", &["m0"], now - 60),
            commit("m1", &["m0"], now - 90),
            commit("m0", &[], now - 120),
            commit("stale-tip", &["stale-root"], now - 15 * DAY),
            commit("stale-root", &[], now - 16 * DAY),
            commit("tag-tip", &["tag-root"], now - 200),
            commit("tag-root", &[], now - 300),
        ],
        vec![
            git_ref("HEAD", "m3", RefKind::WorktreeHead, now - 30, None),
            git_ref("main", "m3", RefKind::Local, now - 30, Some("origin/main")),
            git_ref(
                "origin/main",
                "m3",
                RefKind::Remote {
                    remote: "origin".to_string(),
                },
                now - 30,
                None,
            ),
            git_ref(
                "origin/HEAD",
                "m3",
                RefKind::Remote {
                    remote: "origin".to_string(),
                },
                now - 30,
                None,
            ),
            git_ref(
                "origin/feature",
                "s1",
                RefKind::Remote {
                    remote: "origin".to_string(),
                },
                now - 60,
                None,
            ),
            git_ref(
                "origin/stale",
                "stale-tip",
                RefKind::Remote {
                    remote: "origin".to_string(),
                },
                now - 15 * DAY,
                None,
            ),
            git_ref("v1", "tag-tip", RefKind::Tag, now - 200, None),
        ],
    );

    let pane = Pane::load(&source);

    assert_eq!(visible_ids(&pane), HashSet::from(["m3", "s1", "m1", "m0"]));
    assert_eq!(
        pane.lane_count(),
        2,
        "a stale-only branch and a tag-only history must not widen the graph"
    );
}

#[test]
fn local_upstream_and_origin_head_are_included_even_when_their_tips_are_old() {
    let old = seconds_ago(30);
    let source = SyntheticSource::for_graph_with_refs(
        vec![
            commit("head-tip", &[], old),
            commit("upstream-tip", &[], old - 1),
            commit("origin-head-tip", &[], old - 2),
            commit("other-worktree-tip", &[], old - 3),
        ],
        vec![
            git_ref("HEAD", "head-tip", RefKind::WorktreeHead, old, None),
            git_ref("main", "head-tip", RefKind::Local, old, Some("origin/main")),
            git_ref(
                "origin/main",
                "upstream-tip",
                RefKind::Remote {
                    remote: "origin".to_string(),
                },
                old - 1,
                None,
            ),
            git_ref(
                "origin/HEAD",
                "origin-head-tip",
                RefKind::Remote {
                    remote: "origin".to_string(),
                },
                old - 2,
                None,
            ),
            git_ref(
                "worktree:secondary",
                "other-worktree-tip",
                RefKind::WorktreeHead,
                old - 3,
                None,
            ),
        ],
    );

    let pane = Pane::load(&source);

    assert_eq!(
        visible_ids(&pane),
        HashSet::from([
            "head-tip",
            "upstream-tip",
            "origin-head-tip",
            "other-worktree-tip",
        ])
    );
}
