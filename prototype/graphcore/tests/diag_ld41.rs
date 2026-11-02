//! Throwaway: feed the LD41 shape through the prototype's layout to see what
//! columns it actually produces. Deleted after the diagnosis.

use graphcore::{Commit, Graph};

fn c(sha: &str, parents: Vec<u32>) -> Commit {
    Commit {
        sha: sha.to_string(),
        parents,
        author: "e".into(),
        time: 0,
        subject: sha.to_string(),
        refs: vec![graphcore::RefLabel {
            name: sha.to_string(),
            kind: graphcore::RefKind::Local,
        }],
    }
}

#[test]
fn prototype_columns_on_the_ld41_shape() {
    // rows newest first: m2 -> d1 -> rel -> c1 -> s -> root
    let commits = vec![
        c("m2", vec![4, 2]), // master/HEAD merge: first parent s, second rel
        c("d1", vec![3, 2]), // develop merge:     first parent c1, second rel
        c("rel", vec![3]),   // the release spur
        c("c1", vec![4]),    // top of the shared chain
        c("s", vec![5]),     // deep shared history
        c("root", vec![]),
    ];
    let g = Graph::build(commits, 0.0);
    for (i, co) in g.commits.iter().enumerate() {
        println!("row {i} lane {}: {}", g.lane[i], co.sha);
    }
    println!("max_lanes = {}", g.max_lanes);
}
