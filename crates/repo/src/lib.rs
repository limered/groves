// Emil owns this file — ticket 01 will add `open(path)` plus the `CommitSource` impl here.
use groves_core::{CommitSource, Commit};
use std::path::PathBuf;

pub struct Repo {}

impl CommitSource for Repo{
    fn commits(&self) -> Vec<Commit>{
        Vec::<Commit>::new()
    }
}

pub fn open(_path: &PathBuf) -> Repo{
    Repo{}
}