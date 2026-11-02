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
    // histogram + where the high lanes live
    let mut per_lane: Vec<Vec<usize>> = vec![Vec::new(); pane.lane_count()];
    for (row, r) in pane.rows().iter().enumerate() {
        per_lane[r.lane].push(row);
    }
    for (lane, rows) in per_lane.iter().enumerate() {
        if rows.is_empty() {
            println!("lane {lane}: EMPTY (wasted width)");
        } else {
            println!(
                "lane {lane}: {} rows, first row {} ({})",
                rows.len(),
                rows[0],
                pane.rows()[rows[0]].short
            );
        }
    }
    for (row, r) in pane.rows().iter().enumerate() {
        if r.lane >= 3 {
            println!("HIGH lane row {row:>2}: lane {} {} {}", r.lane, r.short, r.title);
        }
    }
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
