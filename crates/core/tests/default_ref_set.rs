//! Ticket 03 red tests: the pane lays out only history reachable from the
//! default ref set, while preserving fresh branches and excluding stale/tag
//! only history. Tests are owned by the mentor; Emil changes product code.

use groves_core::{GitRef, Pane, RefKind};
use groves_testkit::{GraphCommit, SyntheticSource};
use std::collections::{HashMap, HashSet};
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

fn lanes_by_id(pane: &Pane) -> HashMap<String, usize> {
    pane.rows()
        .iter()
        .map(|row| (row.id.clone(), row.lane))
        .collect()
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

#[test]
fn a_merged_stale_branchs_line_is_still_drawn() {
    // The stale branch was merged into main: the merge commit is fresh, the
    // merged commits are old. main reaches them, so they stay drawn —
    // staleness hides a branch's *tip*, never commits other refs reach.
    let now = seconds_ago(0);
    let source = SyntheticSource::for_graph_with_refs(
        vec![
            commit("m4", &["m3", "st1"], now - 30),
            commit("m3", &["m0"], now - 90),
            commit("m0", &[], now - 120),
            commit("st1", &["st0"], now - 15 * DAY),
            commit("st0", &[], now - 16 * DAY),
        ],
        vec![
            git_ref("HEAD", "m4", RefKind::WorktreeHead, now - 30, None),
            git_ref("main", "m4", RefKind::Local, now - 30, Some("origin/main")),
            git_ref(
                "origin/main",
                "m4",
                RefKind::Remote {
                    remote: "origin".to_string(),
                },
                now - 30,
                None,
            ),
            git_ref(
                "origin/stale",
                "st1",
                RefKind::Remote {
                    remote: "origin".to_string(),
                },
                now - 15 * DAY,
                None,
            ),
        ],
    );

    let pane = Pane::load(&source);

    assert_eq!(
        visible_ids(&pane),
        HashSet::from(["m4", "m3", "m0", "st1", "st0"]),
        "main reaches the stale branch through the merge, so its line stays"
    );
    assert_eq!(
        pane.lane_count(),
        2,
        "the merged line hangs off the merge's side lane; it must not widen the graph"
    );
}

#[test]
fn two_merges_of_one_spur_follow_gits_own_columns() {
    // LD41's real shape, pinned to what `git log --graph` itself renders
    // (verified per-row): the shared chain rides the release spur's lane and
    // bends back into HEAD's lane only where the lines physically merge —
    // the develop tip gets the outermost line. Three lanes total.
    let source = SyntheticSource::for_graph_with_refs(
        vec![
            commit("m2", &["s", "rel"], 40), // 66e17c4: master/HEAD merge
            commit("d1", &["c1", "rel"], 39), // 0b0ec49: develop merge tip
            commit("rel", &["c1"], 38), // ad72308: the release spur
            commit("c1", &["s"], 37), // 350424a: top of the shared chain
            commit("s", &["root"], 30), // d954693: deep shared history
            commit("root", &[], 10),
        ],
        vec![
            git_ref("HEAD", "m2", RefKind::WorktreeHead, 40, None),
            git_ref("master", "m2", RefKind::Local, 40, Some("origin/master")),
            git_ref("develop", "d1", RefKind::Local, 39, None),
        ],
    );

    let pane = Pane::load(&source);
    let lanes = lanes_by_id(&pane);

    assert_eq!(pane.lane_count(), 3, "three lines: HEAD chain, spur, develop tip");
    assert_eq!(lanes["m2"], 0, "HEAD's merge owns lane 0");
    assert_eq!(lanes["rel"], 1, "the release spur sits beside HEAD's lane");
    assert_eq!(lanes["d1"], 2, "the develop tip runs its own line");
    // git renders the shared chain on the line that reached it first (the
    // release spur's) and bends back into lane 0 only where the lines
    // physically join at s.
    assert_eq!(lanes["c1"], 1, "the chain rides the line that claimed it first");
    assert_eq!(lanes["s"], 0, "the trunk resumes lane 0 at the join");
    assert_eq!(lanes["root"], 0);

    // Both of the develop merge's parents must be visibly connected.
    let mut targets: Vec<(usize, usize)> = pane
        .edges()
        .iter()
        .filter(|e| e.child_row == 1)
        .map(|e| (e.parent_row, e.parent_lane))
        .collect();
    targets.sort();
    assert_eq!(
        targets,
        vec![(2, lanes["rel"]), (3, lanes["c1"])],
        "the develop merge connects to both the chain and the spur"
    );
}
