//! Ticket 04 red tests: one Pill per decorated row — HEAD's branch first,
//! then local, then remote; a local branch and its upstream on the same
//! commit collapse into one **Synced pill**; anything else on the commit
//! shows as `+N`. Before any product code exists this fails for the right
//! reason: `Pill`/`PillStyle` and `Row::pill` don't exist yet.
//!
//! Owned by the mentor. Emil makes these pass by changing `crates/core`.
//!
//! Decisions encoded here (agreed in-ticket, see `.scratch/groves/issues/04-pills.md`):
//! - Decorations come from the **full ref snapshot**: a stale remote branch
//!   history passes *through* still decorates its visible tip. Pills must
//!   not flicker when the ref set changes (`s`, filters).
//! - `origin/HEAD` is an alias for the main remote branch — it never takes
//!   the pill and never is counted in `+N` (no `+1` noise on every synced tip).
//! - Secondary **worktree head** refs are markers, not pills (ticket 05 draws them).
//! - **Tags** never pill and never count (spec 26/31).
//! - Ties inside one tier are broken alphabetically. Several locals on one
//!   commit share the tier; the alphabetical rule is refined in ticket 05
//!   when the snapshot carries HEAD's referent name.

use groves_core::{GitRef, Pane, Pill, PillStyle, RefKind};
use groves_testkit::{GraphCommit, SyntheticSource};
use std::time::{SystemTime, UNIX_EPOCH};

const DAY: i64 = 24 * 60 * 60;

fn seconds_ago(days: i64) -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after the Unix epoch")
        .as_secs() as i64;
    now - days * DAY
}

/// Fresh-enough remote ref (inside the 14-day window) so `Pane::load` keeps it.
fn remote_ref(name: &str, target: &str, tip_time: i64) -> GitRef {
    git_ref(
        name,
        target,
        RefKind::Remote {
            remote: "origin".to_string(),
        },
        tip_time,
        None,
    )
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

fn commit(id: &str, parents: &[&str], time: i64) -> GraphCommit {
    GraphCommit {
        id: id.to_string(),
        parents: parents.iter().map(|parent| (*parent).to_string()).collect(),
        title: id.to_string(),
        time,
        refs: vec![],
    }
}

fn row_of(pane: &Pane, id: &str) -> usize {
    pane.rows()
        .iter()
        .position(|row| row.id == id)
        .unwrap_or_else(|| panic!("commit {id} must have a row"))
}

/// The pill a user sees on the row of commit `id`.
fn pill_of<'a>(pane: &'a groves_core::Pane, id: &str) -> Option<&'a Pill> {
    pane.rows()[row_of(pane, id)].pill.as_ref()
}

#[test]
fn local_and_upstream_on_one_commit_collapse_into_a_synced_pill() {
    let now = seconds_ago(0);
    let source = SyntheticSource::for_graph_with_refs(
        vec![
            commit("tip", &["root"], now - 60),
            commit("root", &[], now - 300),
        ],
        vec![
            git_ref("HEAD", "tip", RefKind::WorktreeHead, now - 60, None),
            git_ref("main", "tip", RefKind::Local, now - 60, Some("origin/main")),
            remote_ref("origin/main", "tip", now - 60),
            remote_ref("origin/feature", "tip", now - 60),
        ],
    );

    let pane = Pane::load(&source);

    assert_eq!(
        pill_of(&pane, "tip"),
        Some(&Pill {
            name: "main".to_string(),
            style: PillStyle::Synced,
            plus_count: 1,
        }),
        "main + origin/main on one commit are one Synced pill; origin/feature stays under +N"
    );
    assert_eq!(pill_of(&pane, "root"), None, "root has no refs at all");
}

#[test]
fn local_branch_beats_remote_and_the_rest_is_plus_n() {
    let now = seconds_ago(0);
    let source = SyntheticSource::for_graph_with_refs(
        vec![commit("tip", &[], now - 60)],
        vec![
            git_ref("HEAD", "tip", RefKind::WorktreeHead, now - 60, None),
            git_ref("aaa", "tip", RefKind::Local, now - 60, None),
            remote_ref("origin/zeta", "tip", now - 60),
            remote_ref("origin/mid", "tip", now - 60),
        ],
    );

    let pane = Pane::load(&source);

    assert_eq!(
        pill_of(&pane, "tip"),
        Some(&Pill {
            name: "aaa".to_string(),
            style: PillStyle::Filled,
            plus_count: 2,
        }),
        "the local branch takes the pill, both remotes remain under +N"
    );
}

#[test]
fn remote_only_commit_takes_an_outline_pill() {
    let now = seconds_ago(0);
    let source = SyntheticSource::for_graph_with_refs(
        vec![commit("tip", &[], now - 60)],
        vec![
            git_ref("HEAD", "tip", RefKind::WorktreeHead, now - 60, None),
            remote_ref("origin/feature", "tip", now - 60),
        ],
    );

    let pane = Pane::load(&source);

    assert_eq!(
        pill_of(&pane, "tip"),
        Some(&Pill {
            name: "origin/feature".to_string(),
            style: PillStyle::Outline,
            plus_count: 0,
        }),
        "with no local branch the remote decorates, outlined"
    );
}

#[test]
fn origin_head_alias_never_pills_or_counts() {
    let now = seconds_ago(0);
    let source = SyntheticSource::for_graph_with_refs(
        vec![commit("tip", &[], now - 60)],
        vec![
            git_ref("HEAD", "tip", RefKind::WorktreeHead, now - 60, None),
            git_ref("main", "tip", RefKind::Local, now - 60, Some("origin/main")),
            remote_ref("origin/main", "tip", now - 60),
            remote_ref("origin/HEAD", "tip", now - 60),
        ],
    );

    let pane = Pane::load(&source);

    assert_eq!(
        pill_of(&pane, "tip"),
        Some(&Pill {
            name: "main".to_string(),
            style: PillStyle::Synced,
            plus_count: 0,
        }),
        "origin/HEAD is an alias: no pill of its own, no +N noise"
    );
}

#[test]
fn tags_never_pill_or_count() {
    let now = seconds_ago(0);
    let source = SyntheticSource::for_graph_with_refs(
        vec![
            commit("tip", &["root"], now - 60),
            commit("root", &[], now - 300),
        ],
        vec![
            git_ref("HEAD", "tip", RefKind::WorktreeHead, now - 60, None),
            git_ref("main", "tip", RefKind::Local, now - 60, None),
            git_ref("v1", "tip", RefKind::Tag, now - 60, None),
            git_ref("v2", "root", RefKind::Tag, now - 300, None),
        ],
    );

    let pane = Pane::load(&source);

    assert_eq!(
        pill_of(&pane, "tip"),
        Some(&Pill {
            name: "main".to_string(),
            style: PillStyle::Filled,
            plus_count: 0,
        }),
        "v1 is a tag: it doesn't decorate and doesn't count"
    );
    assert_eq!(pill_of(&pane, "root"), None, "a tag alone shows nothing");
}

#[test]
fn diverged_upstream_stays_unsynced() {
    let now = seconds_ago(0);
    let source = SyntheticSource::for_graph_with_refs(
        vec![
            commit("tip", &["root"], now - 60),
            commit("root", &[], now - 300),
        ],
        vec![
            git_ref("HEAD", "tip", RefKind::WorktreeHead, now - 60, None),
            git_ref("main", "tip", RefKind::Local, now - 60, Some("origin/main")),
            remote_ref("origin/main", "root", now - 300),
        ],
    );

    let pane = Pane::load(&source);

    assert_eq!(
        pill_of(&pane, "tip"),
        Some(&Pill {
            name: "main".to_string(),
            style: PillStyle::Filled,
            plus_count: 0,
        }),
        "origin/main sits on another commit: main is a plain filled pill here"
    );
    assert_eq!(
        pill_of(&pane, "root"),
        Some(&Pill {
            name: "origin/main".to_string(),
            style: PillStyle::Outline,
            plus_count: 0,
        }),
        "the upstream decorates the commit where it actually stands"
    );
}

#[test]
fn upstream_not_in_snapshot_stays_a_plain_local_pill() {
    let now = seconds_ago(0);
    let source = SyntheticSource::for_graph_with_refs(
        vec![commit("tip", &[], now - 60)],
        vec![
            git_ref("HEAD", "tip", RefKind::WorktreeHead, now - 60, None),
            git_ref(
                "main",
                "tip",
                RefKind::Local,
                now - 60,
                Some("origin/missing"),
            ),
        ],
    );

    let pane = Pane::load(&source);

    assert_eq!(
        pill_of(&pane, "tip"),
        Some(&Pill {
            name: "main".to_string(),
            style: PillStyle::Filled,
            plus_count: 0,
        }),
        "the configured upstream ref doesn't exist: no collapse, no crash"
    );
}

#[test]
fn worktree_head_refs_never_pill_or_count() {
    let now = seconds_ago(0);
    let source = SyntheticSource::for_graph_with_refs(
        vec![commit("tip", &[], now - 60)],
        vec![
            git_ref("HEAD", "tip", RefKind::WorktreeHead, now - 60, None),
            git_ref(
                "worktree:secondary",
                "tip",
                RefKind::WorktreeHead,
                now - 60,
                None,
            ),
        ],
    );

    let pane = Pane::load(&source);

    assert_eq!(
        pill_of(&pane, "tip"),
        None,
        "worktree heads are markers (ticket 05), not pills"
    );
}

#[test]
fn refs_in_the_same_tier_are_chosen_alphabetically() {
    let now = seconds_ago(0);
    let source = SyntheticSource::for_graph_with_refs(
        vec![commit("tip", &[], now - 60)],
        vec![
            git_ref("HEAD", "tip", RefKind::WorktreeHead, now - 60, None),
            git_ref("zulu", "tip", RefKind::Local, now - 60, None),
            git_ref("alpha", "tip", RefKind::Local, now - 60, None),
            remote_ref("origin/zeta", "tip", now - 60),
            remote_ref("origin/alpha", "tip", now - 60),
        ],
    );

    let pane = Pane::load(&source);

    assert_eq!(
        pill_of(&pane, "tip"),
        Some(&Pill {
            name: "alpha".to_string(),
            style: PillStyle::Filled,
            plus_count: 3,
        }),
        "locals and remotes tie within their tiers; the winner is alphabetical, the rest is +N"
    );
}

#[test]
fn stale_remote_branch_still_decorates_its_visible_tip() {
    // Ticket 03's shape: the fresh head merges an old branch's line, so the
    // stale branch's tip is *visible* while the branch itself is out of the
    // ref set. Decorations come from the full snapshot, so the tip is labelled.
    let now = seconds_ago(0);
    let source = SyntheticSource::for_graph_with_refs(
        vec![
            commit("m4", &["m3", "st1"], now - 30),
            commit("m3", &["m0"], now - 90),
            commit("m0", &[], now - 120),
            commit("st1", &[], now - 15 * DAY),
        ],
        vec![
            git_ref("HEAD", "m4", RefKind::WorktreeHead, now - 30, None),
            git_ref("main", "m4", RefKind::Local, now - 30, Some("origin/main")),
            remote_ref("origin/main", "m4", now - 30),
            remote_ref("origin/stale", "st1", now - 15 * DAY),
        ],
    );

    let pane = Pane::load(&source);

    assert_eq!(
        pill_of(&pane, "st1"),
        Some(&Pill {
            name: "origin/stale".to_string(),
            style: PillStyle::Outline,
            plus_count: 0,
        }),
        "the stale branch is out of the ref set, but its visible tip stays labelled"
    );
    assert_eq!(
        pill_of(&pane, "m4").map(|pill| &pill.name),
        Some(&"main".to_string())
    );
}
