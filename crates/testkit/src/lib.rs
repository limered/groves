//! Seeded synthetic `CommitSource` and a `git`-built fixture repo for tests.
//! Owned by the mentor — product crates must not depend on this.

use groves_core::{Commit, CommitSource};
use std::path::PathBuf;
use std::process::Command;

/// A synthetic source that yields exactly the commits it was built with,
/// newest first. Drives `core::Pane` without touching git.
pub struct SyntheticSource {
    commits: Vec<Commit>,
}

impl SyntheticSource {
    /// Build from `(full_sha, title)` pairs in newest-first order.
    pub fn new(newest_first: Vec<(String, String)>) -> Self {
        Self {
            commits: newest_first
                .into_iter()
                .map(|(id, title)| Commit { id, title })
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

/// Build a fresh on-disk repo with the three `FIXTURE_TITLES_NEWEST_FIRST`
/// commits (oldest committed first so history order is deterministic) and
/// return its path. Rebuilt on every call so no stale cache can leak between
/// runs.
pub fn init_fixture_repo() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("groves-skeleton-fixture-{}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("remove stale fixture repo");
    }
    std::fs::create_dir_all(&dir).expect("create fixture repo dir");
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
