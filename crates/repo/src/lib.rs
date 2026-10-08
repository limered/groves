use gix;
// Emil owns this file — ticket 01 will add `open(path)` plus the `CommitSource` impl here.
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
    NotARepository(gix::Error),
    #[error("repository has no head")]
    NoHead(gix::Error),
    #[error("head has no commits")]
    NoCommits(gix::Error),
    #[error("error retrieving commit info")]
    NoCommitInfo(gix::Error),
    #[error("commit could not be found")]
    CantFindCommit(gix::Error),
}

pub fn open(path: &Path) -> Result<Repo, RepoError> {
    let repo = gix::open(path).map_err(RepoError::NotARepository)?;
    let head = repo.head_commit().map_err(RepoError::NoHead)?;

    let walk_platform = repo.rev_walk([head.id]);

    let head_commits = head.ancestors().all().map_err(RepoError::NoCommits)?;

    let commits = head_commits
        .map(|item| -> Result<Commit, RepoError> {
            let info = item.map_err(RepoError::NoCommitInfo)?;
            let commit = repo
                .find_commit(info.id)
                .map_err(RepoError::CantFindCommit)?;
            let title = commit
                .message_raw()
                .unwrap_or_default()
                .to_string()
                .lines()
                .next()
                .unwrap_or("")
                .to_owned();

            Ok(Commit {
                parents: info.parent_ids.iter().map(|id| id.to_string()).collect(),
                time: info.commit_time(),
                id: info.id.to_string(),
                title,
                refs: Vec::new(),
            })
        })
        .collect::<Result<_, _>>()?;

    Ok(Repo { commits })
}
