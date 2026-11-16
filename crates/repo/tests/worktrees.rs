//! Ticket 05 red tests: `repo::open` lists every worktree's HEAD (main
//! first, named after the worktree, short SHA when detached), draws history
//! reachable only from secondary worktree heads, and opens a linked
//! worktree folder as the same repository.
//! Owned by the mentor. Emil makes these pass by changing `crates/repo`.

use groves_core::{CommitSource, Pane, RefKind};
use groves_testkit::init_worktree_fixture_repo;

fn worktree_heads<'a>(
    repo: &'a groves_repo::Repo,
) -> Vec<&'a groves_core::GitRef> {
    repo.ref_snapshot()
        .refs()
        .iter()
        .filter(|git_ref| git_ref.kind == RefKind::WorktreeHead)
        .collect()
}

/// `commits()` yields an owned clone, so the helper hands back the id.
fn commit_id_by_title(repo: &groves_repo::Repo, title: &str) -> String {
    repo.commits()
        .iter()
        .find(|commit| commit.title == title)
        .map(|commit| commit.id.clone())
        .unwrap_or_else(|| panic!("commit {title:?} must be walked"))
}

#[test]
fn snapshot_lists_all_worktree_heads_main_first_with_names() {
    let fixture = init_worktree_fixture_repo();
    let repo = groves_repo::open(&fixture.main).expect("fixture must open");
    let heads = worktree_heads(&repo);

    assert_eq!(
        heads.len(),
        3,
        "main worktree + attached side checkout + detached checkout"
    );

    let main_id = commit_id_by_title(&repo, "Root");
    let side_tip_id = commit_id_by_title(&repo, "Side only work");
    let stray_id = commit_id_by_title(&repo, "Detached only work");

    assert_eq!(heads[0].name, "repo", "the main worktree is named after its folder");
    assert_eq!(
        heads[1].name,
        &stray_id[..7],
        "a detached worktree head is named by its short SHA"
    );
    assert_eq!(heads[2].name, "side-checkout", "a linked worktree is named by its git worktree id");

    assert_eq!(heads[0].target, main_id, "main worktree HEAD at the main tip");
    assert_eq!(heads[1].target, stray_id, "detached worktree HEAD at the stray commit");
    assert_eq!(heads[2].target, side_tip_id, "side worktree HEAD at its own tip");

    for head in &heads {
        assert!(
            head.target.len() >= 7 && head.tip_time > 0,
            "each worktree head carries its target commit's time"
        );
    }
}

#[test]
fn history_reachable_only_from_secondary_worktree_heads_is_drawn() {
    let fixture = init_worktree_fixture_repo();
    let repo = groves_repo::open(&fixture.main).expect("fixture must open");
    let pane = Pane::load(&repo);

    let titles: Vec<&str> = pane.rows().iter().map(|row| row.title.as_str()).collect();
    assert!(
        titles.contains(&"Side only work"),
        "the side worktree's own commit is drawn (side {titles:?})"
    );
    assert!(
        titles.contains(&"Detached only work"),
        "the detached worktree's stray commit is drawn, though no branch reaches it (titles {titles:?})"
    );

    // The side worktree head carries its marker on the drawn row.
    let side_tip_id = commit_id_by_title(&repo, "Side only work");
    let side_row = pane
        .rows()
        .iter()
        .find(|row| row.id == side_tip_id)
        .expect("side tip has a row");
    assert!(
        side_row.head_markers.iter().any(|m| m.name == "side-checkout"),
        "the side worktree's marker is on its HEAD row"
    );
}

#[test]
fn opening_a_linked_worktree_folder_yields_the_same_repository() {
    let fixture = init_worktree_fixture_repo();

    let from_main = groves_repo::open(&fixture.main).expect("main folder opens");
    let from_side = groves_repo::open(&fixture.side).expect("linked worktree folder opens");

    let ids = |repo: &groves_repo::Repo| -> Vec<String> {
        repo.commits().iter().map(|commit| commit.id.clone()).collect()
    };
    assert_eq!(
        ids(&from_main),
        ids(&from_side),
        "opening the linked worktree folder yields the same history"
    );

    let head_names = |repo: &groves_repo::Repo| -> Vec<String> {
        worktree_heads(repo).iter().map(|head| head.name.clone()).collect()
    };
    assert_eq!(
        head_names(&from_main),
        head_names(&from_side),
        "the same worktree heads in the same order, opened from either folder"
    );
}
