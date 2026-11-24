#[derive(Clone, Debug)]
pub struct Commit {
    pub id: String,
    pub parents: Vec<String>,
    pub time: i64,
    pub title: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum RefKind {
    Local,
    Remote { remote: String },
    Tag,
    WorktreeHead,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GitRef {
    pub name: String,
    pub target: String,
    pub kind: RefKind,
    pub tip_time: i64,
    pub upstream: Option<String>,
}

#[derive(Default)]
pub struct RefSnapshot {
    refs: Vec<GitRef>,
}

impl RefSnapshot {
    pub fn new(refs: Vec<GitRef>) -> Self {
        Self { refs }
    }
    pub fn refs(&self) -> &[GitRef] {
        &self.refs
    }
}
