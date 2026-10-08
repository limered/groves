//! Seeded synthetic `CommitSource` and `git`-built fixture repos for tests.
//! Owned by the mentor — product crates must not depend on this.
//!
//! Ticket 02 additions: the source can now describe a graph (parents, times,
//! ref names) so `core::Pane` layout tests can assert lanes per commit.

use groves_core::{Commit, CommitSource};
use std::path::PathBuf;
use std::process::Command;

/// A synthetic source that yields exactly the commits it was built with,
/// newest first. Drives `core::Pane` without touching git.
pub struct SyntheticSource {
    commits: Vec<Commit>,
}

/// One commit of a synthetic graph: ids are free-form strings in tests
/// (readable like `"m2"`), parents name other commits' ids.
pub struct GraphCommit {
    pub id: String,
    pub parents: Vec<String>,
    pub title: String,
    pub time: i64,
    pub refs: Vec<String>,
}

impl SyntheticSource {
    /// Build from `(full_sha, title)` pairs in newest-first order.
    /// No parents, no refs — the ticket-01 shape. Kept so the skeleton
    /// tests keep driving the same facade.
    pub fn new(newest_first: Vec<(String, String)>) -> Self {
        let n = newest_first.len();
        Self {
            commits: newest_first
                .into_iter()
                .enumerate()
                .map(|(i, (id, title))| Commit {
                    id,
                    parents: vec![],
                    // Descending times keep the old newest-first order temporal.
                    time: (n - i) as i64,
                    title,
                    refs: vec![],
                })
                .collect(),
        }
    }

    /// Three commits with fixed 40-hex-char ids, newest first.
    pub fn three() -> Self {
        Self::new(vec![
            (
                "c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3".to_string(),
                "Fix lane reuse".to_string(),
            ),
            (
                "b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2".to_string(),
                "Add filter box".to_string(),
            ),
            (
                "a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1".to_string(),
                "Initial commit".to_string(),
            ),
        ])
    }

    /// Build from an explicit graph, given newest first (row order is the
    /// pane's job — it must reorder when this input isn't topological).
    pub fn for_graph(newest_first: Vec<GraphCommit>) -> Self {
        Self {
            commits: newest_first
                .into_iter()
                .map(|g| Commit {
                    id: g.id,
                    parents: g.parents,
                    time: g.time,
                    title: g.title,
                    refs: g.refs,
                })
                .collect(),
        }
    }

    fn g(id: &str, parents: &[&str], title: &str, time: i64, refs: &[&str]) -> GraphCommit {
        GraphCommit {
            id: id.to_string(),
            parents: parents.iter().map(|p| (*p).to_string()).collect(),
            title: title.to_string(),
            time,
            refs: refs.iter().map(|r| (*r).to_string()).collect(),
        }
    }

    /// Linear history `m0 <- m1 <- m2`, HEAD on the tip.
    pub fn linear_three() -> Self {
        Self::for_graph(vec![
            Self::g("m2", &["m1"], "Tip", 300, &["HEAD", "main"]),
            Self::g("m1", &["m0"], "Middle", 200, &[]),
            Self::g("m0", &[], "Root", 100, &[]),
        ])
    }

    /// A side branch off `m0`, merged back at the HEAD tip:
    /// `m0 <- m1 <- m3 (HEAD)`, `m0 <- s1 <- m3`.
    /// After the merge the freed side lane must be reused, so the graph
    /// never needs more than two lanes.
    pub fn branch_and_merge() -> Self {
        Self::for_graph(vec![
            Self::g(
                "m3",
                &["m1", "s1"],
                "Merge side branch",
                400,
                &["HEAD", "main"],
            ),
            Self::g("s1", &["m0"], "Side work", 300, &[]),
            Self::g("m1", &["m0"], "Main work", 200, &[]),
            Self::g("m0", &[], "Root", 100, &[]),
        ])
    }

    /// The side-branch tip is *newer* than HEAD's tip, but HEAD's
    /// first-parent chain (`m0 <- m1 <- m2`) must still own lane 0:
    /// lanes are seeded from HEAD/main, not from recency.
    pub fn head_newer_side_branch() -> Self {
        Self::for_graph(vec![
            Self::g("s1", &["m0"], "Newer side work", 400, &["origin/side"]),
            Self::g("m2", &["m1"], "HEAD tip", 300, &["HEAD", "main"]),
            Self::g("m1", &["m0"], "Main work", 200, &[]),
            Self::g("m0", &[], "Root", 100, &[]),
        ])
    }

    /// Same commits as [`Self::branch_and_merge`], but the source lists a
    /// parent (`m0`) before its children. The pane must still draw children
    /// above parents, in the same order as the well-ordered input.
    pub fn shuffled_branch_and_merge() -> Self {
        Self::for_graph(vec![
            Self::g("m0", &[], "Root", 100, &[]),
            Self::g(
                "m3",
                &["m1", "s1"],
                "Merge side branch",
                400,
                &["HEAD", "main"],
            ),
            Self::g("m1", &["m0"], "Main work", 200, &[]),
            Self::g("s1", &["m0"], "Side work", 300, &[]),
        ])
    }
}

impl CommitSource for SyntheticSource {
    fn commits(&self) -> Vec<Commit> {
        self.commits.clone()
    }
}

/// Titles the fixture repo is built with, newest first.
pub const FIXTURE_TITLES_NEWEST_FIRST: &[&str] = &[
    "Third: fix lane reuse",
    "Second: add filter box",
    "First: initial commit",
];

fn git(dir: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "Groves Test")
        .env("GIT_AUTHOR_EMAIL", "test@groves.invalid")
        .env("GIT_COMMITTER_NAME", "Groves Test")
        .env("GIT_COMMITTER_EMAIL", "test@groves.invalid")
        .status()
        .expect("git must be on PATH for fixture setup");
    assert!(status.success(), "git {args:?} failed in {}", dir.display());
}

fn fresh_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("groves-{name}-{}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("remove stale fixture repo");
    }
    std::fs::create_dir_all(&dir).expect("create fixture repo dir");
    dir
}

/// Build a fresh on-disk repo with the three `FIXTURE_TITLES_NEWEST_FIRST`
/// commits (oldest committed first so history order is deterministic) and
/// return its path. Rebuilt on every call so no stale cache can leak between
/// runs.
pub fn init_fixture_repo() -> PathBuf {
    let dir = fresh_dir("skeleton-fixture");
    git(&dir, &["init", "-q"]);
    // Oldest first: commit the fixture titles in reverse.
    for (i, title) in FIXTURE_TITLES_NEWEST_FIRST.iter().rev().enumerate() {
        let file = dir.join(format!("file{i}.txt"));
        std::fs::write(&file, format!("content {title}\n")).expect("write fixture file");
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-q", "-m", title]);
    }
    dir
}

/// Build a fresh on-disk repo with a side branch merged back into main:
/// `root <- main-tip` is HEAD, with `root <- side work <- merge` joining at
/// the tip. Returns the repo path.
pub fn init_branch_fixture_repo() -> PathBuf {
    let dir = fresh_dir("branch-fixture");
    git(&dir, &["init", "-q", "-b", "main"]);
    std::fs::write(dir.join("root.txt"), "root\n").expect("write root file");
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "Root"]);
    git(&dir, &["checkout", "-q", "-b", "side"]);
    std::fs::write(dir.join("side.txt"), "side\n").expect("write side file");
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "Side work"]);
    git(&dir, &["checkout", "-q", "main"]);
    std::fs::write(dir.join("main.txt"), "main\n").expect("write main file");
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "Main work"]);
    git(
        &dir,
        &["merge", "-q", "--no-ff", "-m", "Merge side branch", "side"],
    );
    dir
}
