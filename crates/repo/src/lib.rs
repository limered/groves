use gix::{ObjectId, objs::kind};
// Emil owns this file — ticket 01 will add `open(path)` plus the `CommitSource` impl here.
use groves_core::{Commit, CommitSource, GitRef, RefKind, RefSnapshot};
use std::{any::Any, collections::HashMap, path::Path};
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
    #[error("coudn't get refs in repo, error: {0}")]
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

fn upstream_name(worktree_repo: &gix::Repository) -> Result<Option<String>, RepoError> {
    let head = worktree_repo.head().map_err(RepoError::NoHead)?;
    let Some(branch_name) = head.referent_name() else {
        return Ok(None);
    };
    let upstream_repo = worktree_repo
        .branch_remote_tracking_ref_name(branch_name, gix::remote::Direction::Fetch)
        .transpose()
        .map_err(RepoError::NoUpstream)?;

    Ok(upstream_repo.map(|n| n.to_string()))
}

pub fn open(path: &Path) -> Result<Repo, RepoError> {
    let repo = gix::open(path).map_err(RepoError::NotARepository)?;
    let refs: Vec<GitRef> = Vec::new();
    let head = repo.head_commit().map_err(RepoError::NoHead)?;

    let mut wkt_heads: Vec<GitRef> = Vec::new();
    let mut ref_commits: HashMap<String, gix::ObjectId> = HashMap::new();
    ref_commits.insert("HEAD".to_string(), head.id);

    let worktrees = repo.worktrees_including_main().map_err(RepoError::Refs)?;
    for worktree in worktrees {
        let wkt = worktree.map_err(RepoError::Refs)?;
        let mut wkt_head = wkt.head().map_err(RepoError::NoHead)?;
        let wkt_head_commit = wkt_head.peel_to_commit().map_err(RepoError::Refs)?;
        let name = match wkt.head_name().map_err(RepoError::NoHead)? {
            Some(name) => name.as_bstr().to_string(),
            None => "HEAD".to_string(),
        };
        let time = wkt_head_commit.time().map_err(RepoError::Refs)?.seconds;

        let wkt_upstream = upstream_name(&wkt)?;
        wkt_heads.push(GitRef {
            name: name.clone(),
            target: wkt_head_commit.id.to_string(),
            kind: RefKind::WorktreeHead,
            tip_time: time,
            upstream: wkt_upstream,
        });
        ref_commits.insert(name ,wkt_head_commit.id);
    }
    println!("{:?}", wkt_heads);

    let walk_platform = repo.rev_walk([head.id]);
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
        ref_snapshot: RefSnapshot::new(Vec::new()),
    })
}
