// Emil owns this file — ticket 01 will add `open(path)` plus the `CommitSource` impl here.
use gix;
use groves_core::{Commit, CommitSource};
use std::path::Path;
use thiserror::Error;

pub struct Repo {
    commits: Vec<Commit>,
}

impl CommitSource for Repo {
    fn commits(&self) -> Vec<Commit> {
        self.commits.clone()
    }
}
#[derive(Error, Debug)]
pub enum RepoError {
    #[error("can't open repo: {0}")]
    NotARepository(#[source] gix::Error),
    #[error("repository has no head")]
    NoHead(#[source] gix::Error),
    #[error("head has no commits")]
    NoCommits(#[source] gix::Error),
    #[error("error retrieving commit info")]
    NoCommitInfo(#[source] gix::Error),
    #[error("commit could not be found")]
    CantFindCommit(#[source] gix::Error),
    #[error("can't traverse commit tree")]
    Walk(#[source] gix::Error),
    #[error("commit has no message {0}")]
    NoCommitMessage(#[source] gix::Error),
    #[error("commit has no auhor {0}")]
    MissingAuthor(#[source] gix::Error),
}

pub fn open(path: &Path) -> Result<Repo, RepoError> {
    let repo = gix::open(path).map_err(RepoError::NotARepository)?;
    let head = repo.head_commit().map_err(RepoError::NoHead)?;

    let walk_platform = repo.rev_walk([head.id]);
    let configured_walk = walk_platform.sorting(gix::revision::walk::Sorting::ByCommitTime(
        gix::traverse::commit::simple::CommitTimeOrder::NewestFirst,
    ));

    let mut commit_iter = configured_walk.all().map_err(RepoError::Walk)?;

    let mut commits: Vec<Commit> = Vec::new();

    while let Some(walk_result) = commit_iter.next() {
        let info = walk_result.map_err(RepoError::NoCommitInfo)?;
        let commit = repo
            .find_commit(info.id)
            .map_err(RepoError::CantFindCommit)?;

        let message = commit.message().map_err(RepoError::NoCommitMessage)?;
        let summary = message.summary();
        let _author = commit.author().map_err(RepoError::MissingAuthor)?;

        commits.push(Commit {
            parents: info.parent_ids.iter().map(|id| id.to_string()).collect(),
            time: info.commit_time(),
            id: info.id.to_string(),
            title: summary.to_string(),
            refs: Vec::new(),
        });
    }

    Ok(Repo { commits })
}
