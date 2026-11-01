//! Throwaway diagnostic: runs the real pipeline on a local LD41 checkout to
//! dump rows/lanes/edges. Machine-specific path — deleted after diagnosis,
//! never meant to be a keeper.

use groves_core::{CommitSource, Pane};
use std::path::Path;

#[test]
fn diagnostic_dump_ld41() {
    let repo = groves_repo::open(Path::new("/home/limered/Projects/LD41"))
        .expect("LD41 checkout must open");
    let pane = Pane::load(&repo);

    for (row, r) in pane.rows().iter().enumerate().take(6) {
        println!("row {row:>2} lane {}: {} {}", r.lane, r.short, r.title);
    }
    println!("lane_count = {}", pane.lane_count());
    println!("edges touching the first 3 rows:");
    for e in pane.edges() {
        if e.child_row <= 3 {
            println!(
                "  child(row {}, lane {}) -> parent(row {}, lane {})",
                e.child_row, e.child_lane, e.parent_row, e.parent_lane
            );
        }
    }
    // keep the assertion trivial so the dump prints, then fails loudly either way
    assert!(pane.rows().len() > 4);
}
