// Emil owns this file — ticket 01 will add `open(path)` plus the `CommitSource` impl here.
use gix::refs::Category;
use gix::remote::Direction;
use groves_core::RefKind::WorktreeHead;
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
    NotARepository(#[source] gix::Error),
    #[error("repository has no head")]
    NoHead(#[source] gix::Error),
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
}

fn map_category_to_ref_kind(category: Category, remote: String) -> RefKind {
    match category {
        Category::LocalBranch => RefKind::Local,
        Category::PseudoRef => RefKind::WorktreeHead,
        Category::RemoteBranch => RefKind::Remote { remote },
        Category::Tag => RefKind::Tag,
        _ => RefKind::WorktreeHead,
    }
}

pub fn open(path: &Path) -> Result<Repo, RepoError> {
    let repo = gix::open(path).map_err(RepoError::NotARepository)?;
    let _refs: Vec<GitRef> = Vec::new();

    let mut walk_refs: Vec<GitRef> = Vec::new();
    let mut ref_commits: Vec<gix::ObjectId> = Vec::new();
    
    let worktrees = repo.worktrees_including_main().map_err(RepoError::Refs)?;
    for worktree in worktrees {
        let wkt = worktree.map_err(RepoError::Refs)?;
        let mut head = wkt.head().map_err(RepoError::NoHead)?;
        let target = head.peel_to_commit().map_err(RepoError::Refs)?;
        let Some(wkt_ref_name) = head.referent_name() else {
            continue;
        };
        let time = target.time().map_err(RepoError::Refs)?.seconds;

        let upstream_ref = repo
            .branch_remote_tracking_ref_name(wkt_ref_name, gix::remote::Direction::Fetch)
            .transpose()
            .map_err(RepoError::NoUpstream)?
            .map(|n| n.shorten().to_string());

        let remote = match wkt.remote_default_name(Direction::Fetch) {
            Some(remote_name) => remote_name.to_string(),
            None => "".to_string(),
        };

        walk_refs.push(GitRef {
            name: "HEAD".to_string(),
            target: target.id.to_string(),
            kind: WorktreeHead,
            tip_time: time,
            upstream: upstream_ref,
        });
        ref_commits.push(target.id);
    }

    let all_refs = repo.references().map_err(RepoError::Refs)?;
    for one_ref in all_refs.all().map_err(RepoError::Refs)? {
        let r = one_ref.map_err(RepoError::Refs)?;
        let name = r.name();
        let remote_name = match r.remote_name(Direction::Fetch) {
            Some(remote_name) => remote_name.as_bstr().to_string(),
            None => "".to_string(),
        };
        let target = r.clone().peel_to_commit().map_err(RepoError::Refs)?;
        let time = target.time().map_err(RepoError::Refs)?.seconds;
        let upstream_ref = repo
            .branch_remote_tracking_ref_name(name, gix::remote::Direction::Fetch)
            .transpose()
            .map_err(RepoError::NoUpstream)?
            .map(|n| n.shorten().to_string());

        let (category, short_name) = name.category_and_short_name().unwrap();

        walk_refs.push(GitRef {
            name: short_name.to_string(),
            target: target.id.to_string(),
            tip_time: time,
            kind: map_category_to_ref_kind(category, remote_name),
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
            refs: Vec::new(),
        });
    }

    Ok(Repo {
        commits,
        ref_snapshot: RefSnapshot::new(walk_refs),
    })
}
