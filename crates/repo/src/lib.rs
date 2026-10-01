// Emil owns this file — ticket 01 will add `open(path)` plus the `CommitSource` impl here.
use gix::{self};
use groves_core::{Commit, CommitSource};
use std::path::PathBuf;

pub struct Repo {
    commits: Vec<Commit>,
}

impl CommitSource for Repo {
    fn commits(&self) -> Vec<Commit> {
        self.commits.clone()
    }
}

pub fn open(path: &PathBuf) -> Result<Repo, Box<dyn std::error::Error>> {
    let repo = gix::open(path)?;
    let head = repo.head_commit()?;
    let commits = head
        .ancestors()
        .all()?
        .map(|item| -> Result<Commit, Box<dyn std::error::Error>> {
            let info = item?;

            let commit = repo.find_commit(info.id)?;
            let title = commit
                .message_raw()?
                .to_string()
                .lines()
                .next()
                .unwrap_or("")
                .to_owned();
            Ok(Commit {
                id: info.id.to_string(),
                title,
            })
        })
        .collect::<Result<_, _>>()?;

    Ok(Repo { commits })
}
