//! Ticket 01 red tests: `repo::open` exposes an on-disk repo as a
//! `CommitSource` (newest first), and errors on paths that aren't repos.
//! Owned by the mentor. Emil makes these pass by changing `crates/repo`.

use groves_core::CommitSource;
use groves_testkit::{FIXTURE_TITLES_NEWEST_FIRST, init_fixture_repo};

#[test]
fn open_yields_fixture_commits_newest_first() {
    let path = init_fixture_repo();
    let repo = groves_repo::open(&path).expect("fixture repo should open");

    let titles: Vec<String> = repo.commits().iter().map(|c| c.title.clone()).collect();
    let expected: Vec<String> = FIXTURE_TITLES_NEWEST_FIRST
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        titles, expected,
        "commits reachable from HEAD, newest first"
    );
    assert!(!repo.commits().is_empty());
    for commit in repo.commits() {
        assert!(!commit.id.is_empty(), "each commit carries its id");
    }
}

#[test]
fn open_rejects_a_path_that_is_not_a_repo() {
    let dir = std::env::temp_dir().join(format!("groves-not-a-repo-{}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).unwrap();
    }
    std::fs::create_dir_all(&dir).unwrap();

    assert!(
        groves_repo::open(&dir).is_err(),
        "opening a plain directory must fail instead of yielding empty history"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}
