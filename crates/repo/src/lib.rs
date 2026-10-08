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
    NotARepository(#[source] Box<gix::open::Error>),
    #[error("repository has no head")]
    NoHead(#[from] gix::reference::head_commit::Error),
    #[error("head has no commits")]
    NoCommits(#[from] gix::revision::walk::Error),
    #[error("error retrieving commit info")]
    NoCommitInfo(#[from] gix::revision::walk::iter::Error),
    #[error("commit could not be found")]
    CantFindCommit(#[from] gix::object::find::existing::with_conversion::Error),
}

impl From<gix::open::Error> for RepoError {
    fn from(error: gix::open::Error) -> Self {
        Self::NotARepository(Box::new(error))
    }
}

pub fn open(path: &Path) -> Result<Repo, RepoError> {
    let repo = gix::open(path)?;
    let head = repo.head_commit()?;

    let head_commits = head.ancestors().all()?;

    let commits = head_commits
        .map(|item| -> Result<Commit, RepoError> {
            let info = item?;
            let commit = repo.find_commit(info.id)?;
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
