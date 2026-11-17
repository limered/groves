// Emil owns this file — ticket 01 will add `open(path)` plus the `CommitSource` impl here.
use gix::Repository;
use gix::refs::{Category, FullNameRef};
use gix::remote::Direction;
use groves_core::{Commit, CommitSource, GitRef, RefKind, RefSnapshot};
use std::path::Path;
use thiserror::Error;

#[derive(Default)]
pub struct Repo {
    commits: Vec<Commit>,
    ref_snapshot: RefSnapshot,
}

impl CommitSource for Repo {
    fn commits(&self) -> Vec<Commit> {
        self.commits.clone()
    }
    fn ref_snapshot(&self) -> &RefSnapshot {
        &self.ref_snapshot
    }
}
#[derive(Error, Debug)]
pub enum RepoError {
    #[error("can't open repo: {0}")]
    CantOpenRepository(#[source] gix::Error),
    #[error("can't open head: {0}")]
    Head(#[source] gix::Error),
    #[error("couldn't get refs in repo, error: {0}")]
    Refs(#[source] gix::Error),
    #[error("error retrieving commit info")]
    NoCommitInfo(#[source] gix::Error),
    #[error("commit could not be found")]
    CantFindCommit(#[source] gix::Error),
    #[error("can't traverse commit tree")]
    Walk(#[source] gix::Error),
    #[error("commit has no message {0}")]
    NoCommitMessage(#[source] gix::Error),
    #[error("ref has no upstream {0}")]
    NoUpstream(#[source] gix::Error),
    #[error("error reading worktrees {0}")]
    WorktreesMissing(#[source] gix::Error),
}

pub fn open(path: &Path) -> Result<Repo, RepoError> {
    let repo = gix::open(path).map_err(RepoError::CantOpenRepository)?;
    let mut walk_refs: Vec<GitRef> = Vec::new();
    let mut ref_commits: Vec<gix::ObjectId> = Vec::new();

    let main_repo = repo.main_repo().map_err(RepoError::CantOpenRepository)?;
    let main_repo_name = main_repo
        .workdir()
        .unwrap_or_else(|| main_repo.git_dir())
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let mut head = main_repo.head().map_err(RepoError::Head)?;
    let head_target = head.peel_to_commit().map_err(RepoError::Head)?;
    let time = head_target.time().map_err(RepoError::Head)?;
    let upstream_ref = resolve_upstream_ref(&repo, head.name());

    walk_refs.push(GitRef {
        name: main_repo_name,
        target: head_target.id.to_string(),
        kind: RefKind::WorktreeHead,
        tip_time: time.seconds,
        upstream: upstream_ref,
    });
    ref_commits.push(head_target.id);

    let worktree_proxies = repo.worktrees().map_err(RepoError::WorktreesMissing)?;
    for proxy in worktree_proxies {
        let mut name = proxy.id().unwrap().to_string();
        let Ok(wkt_repo) = proxy.into_repo_with_possibly_inaccessible_worktree() else {
            continue;
        };

        let Ok(mut head) = wkt_repo.head() else {
            continue;
        };
        let Ok(target) = head.peel_to_commit() else {
            continue;
        };

        let mut upstream_ref : Option<String> = None;
        let wkt_ref = head.referent_name();
        if let Some(wkt_ref_name) = wkt_ref{
            upstream_ref = resolve_upstream_ref(&repo, wkt_ref_name);
        }else {
            name = target.short_id().unwrap().to_string();
        }

        let Ok(time) = target.time() else { continue };
        walk_refs.push(GitRef {
            name: name.to_string(),
            target: target.id.to_string(),
            kind: RefKind::WorktreeHead,
            tip_time: time.seconds,
            upstream: upstream_ref,
        });
        ref_commits.push(target.id);
    }

    let all_refs = repo.references().map_err(RepoError::Refs)?;
    for one_ref in all_refs.all().map_err(RepoError::Refs)? {
        let Ok(me) = one_ref else { continue };
        let name = me.name();
        let remote_name = match me.remote_name(Direction::Fetch) {
            Some(remote_name) => remote_name.as_bstr().to_string(),
            None => "".to_string(),
        };
        let Some(target) = me.clone().peel_to_commit().ok() else {
            continue;
        };
        let Ok(time) = target.time() else { continue };
        let upstream_ref = resolve_upstream_ref(&repo, name);

        let Some((category, short_name)) = name.category_and_short_name() else {
            continue;
        };

        let Some(kind) = map_category_to_ref_kind(category, remote_name) else {
            continue;
        };

        walk_refs.push(GitRef {
            name: short_name.to_string(),
            target: target.id.to_string(),
            tip_time: time.seconds,
            kind,
            upstream: upstream_ref,
        });
        ref_commits.push(target.id);
    }

    let walk_platform = repo.rev_walk(ref_commits);
    let configured_walk = walk_platform.sorting(gix::revision::walk::Sorting::ByCommitTime(
        gix::traverse::commit::simple::CommitTimeOrder::NewestFirst,
    ));

    let commit_iter = configured_walk.all().map_err(RepoError::Walk)?;

    let mut commits: Vec<Commit> = Vec::new();
    for walk_result in commit_iter {
        let info = walk_result.map_err(RepoError::NoCommitInfo)?;
        let commit = repo
            .find_commit(info.id)
            .map_err(RepoError::CantFindCommit)?;

        let message = commit.message().map_err(RepoError::NoCommitMessage)?;
        let summary = message.summary();

        commits.push(Commit {
            parents: info.parent_ids.iter().map(|id| id.to_string()).collect(),
            time: info.commit_time(),
            id: info.id.to_string(),
            title: summary.to_string(),
        });
    }

    Ok(Repo {
        commits,
        ref_snapshot: RefSnapshot::new(walk_refs),
    })
}

fn resolve_upstream_ref(repo: &Repository, name: &FullNameRef) -> Option<String> {
    repo.branch_remote_tracking_ref_name(name, Direction::Fetch)
        .transpose()
        .ok()
        .flatten()
        .map(|n| n.shorten().to_string())
}

fn map_category_to_ref_kind(category: Category, remote: String) -> Option<RefKind> {
    match category {
        Category::LocalBranch => Some(RefKind::Local),
        Category::PseudoRef => Some(RefKind::WorktreeHead),
        Category::RemoteBranch => Some(RefKind::Remote { remote }),
        Category::Tag => Some(RefKind::Tag),
        _ => None,
    }
}
