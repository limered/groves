// Emil owns this file — ticket 01 will add `CommitSource`, `Commit`, `Row` and `Pane` here.

#[derive(Clone)]
pub struct Commit {
    pub id: String,
    pub title: String,
}

#[derive(Clone)]
pub struct Row {
    pub short: String,
    pub title: String,
}

pub struct Pane {
    rows: Vec<Row>,
}

impl Pane {
    pub fn load(source: &impl CommitSource) -> Self {
        let rows = source
            .commits()
            .into_iter()
            .map(|c| Row {
                short: c.id.get(..7).unwrap_or(&c.id).to_string(),
                title: c.title,
            })
            .collect();
        Pane { rows }
    }
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }
}

pub trait CommitSource {
    fn commits(&self) -> Vec<Commit>;
}
