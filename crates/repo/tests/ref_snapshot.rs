//! Ticket 03 red tests: `repo::open` reports the ref snapshot needed to build
//! the default ref set and retains commits reachable outside the current HEAD.

use groves_core::{CommitSource, RefKind};
use groves_testkit::init_ref_fixture_repo;

fn ref_named<'a>(repo: &'a groves_repo::Repo, name: &str) -> &'a groves_core::GitRef {
    repo.ref_snapshot()
        .refs()
        .iter()
        .find(|git_ref| git_ref.name == name)
        .unwrap_or_else(|| panic!("ref {name} must be in the snapshot"))
}

#[test]
fn open_reports_loose_packed_upstream_symbolic_and_tag_refs() {
    let path = init_ref_fixture_repo();
    let repo = groves_repo::open(&path).expect("fixture must open");
    let refs = repo.ref_snapshot().refs();

    assert!(
        refs.iter()
            .all(|git_ref| !git_ref.name.starts_with("refs/")),
        "refs are reported under short names like 'main' and 'origin/main'"
    );

    let main = ref_named(&repo, "main");
    assert!(matches!(&main.kind, RefKind::Local));
    assert_eq!(main.upstream.as_deref(), Some("origin/main"));

    let packed_remote = ref_named(&repo, "origin/active");
    assert!(matches!(&packed_remote.kind, RefKind::Remote { .. }));
    let loose_remote = ref_named(&repo, "origin/loose");
    assert!(matches!(&loose_remote.kind, RefKind::Remote { .. }));

    let origin_head = ref_named(&repo, "origin/HEAD");
    assert!(matches!(&origin_head.kind, RefKind::Remote { .. }));
    assert_eq!(origin_head.target, ref_named(&repo, "origin/main").target);

    // Ticket 05 renamed worktree HEAD refs to worktree names; the main
    // worktree is the first WorktreeHead ref, named after its folder.
    let main_worktree_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .expect("fixture path is valid UTF-8");
    let head = refs
        .iter()
        .find(|git_ref| git_ref.kind == RefKind::WorktreeHead)
        .expect("the main worktree's HEAD must be in the snapshot");
    assert_eq!(head.name, main_worktree_name);
    assert_eq!(head.target, ref_named(&repo, "main").target);

    let tag = ref_named(&repo, "v1");
    assert!(matches!(&tag.kind, RefKind::Tag));
    assert_eq!(tag.target, ref_named(&repo, "origin/stale").target);

    let commits = repo.commits();
    for git_ref in refs {
        let target = commits
            .iter()
            .find(|commit| commit.id == git_ref.target)
            .unwrap_or_else(|| panic!("target {} must be cached", git_ref.name));
        assert_eq!(
            git_ref.tip_time, target.time,
            "{} carries its target commit's committer time",
            git_ref.name
        );
    }
    assert!(
        commits
            .iter()
            .any(|commit| commit.title == "Old remote history"),
        "the source cache includes commits reachable only from a non-HEAD ref"
    );
}
