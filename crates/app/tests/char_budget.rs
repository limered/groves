//! The 20-character pill budget: how many characters the branch pill, the
//! `+n` pill and the worktree-head-marker pill each get when they share a
//! row (ticket 20). The marker budget is how much of the worktree path the
//! marker pill may show; it is only consumed when a head marker is drawn,
//! so tests without one bind it to `_`.
//! Owned by the mentor; Emil makes these pass by changing `crates/app`.
//!
//! Expected facade:
//! ```rust,ignore
//! // crates/app/src/render.rs
//! pub fn calculate_char_budget(
//!     has_head_marker: bool,
//!     has_plus_marker: bool,
//!     pill_name_length: usize,
//! ) -> (usize, usize); // (branch_char_budget, marker_budget)
//! ```

use groves_app::render::calculate_char_budget;

#[test]
fn a_branch_alone_gets_the_full_budget() {
    // 30 chars: longer than any slice of the budget, nothing to hand back.
    let (branch, _) = calculate_char_budget(false, false, 30);
    assert_eq!(branch, 20, "rule 1: only a branch, so the full 20");
}

#[test]
fn a_plus_marker_takes_three_from_the_branch_budget() {
    let (branch, _) = calculate_char_budget(false, true, 30);
    assert_eq!(branch, 17, "rule 2: +n costs the branch 3");
}

#[test]
fn a_head_marker_takes_nine_from_the_branch_budget() {
    assert_eq!(
        calculate_char_budget(true, false, 30),
        (11, 9),
        "rule 3: marker costs the branch 9, marker keeps 9"
    );
}

#[test]
fn head_marker_and_plus_marker_combine() {
    assert_eq!(
        calculate_char_budget(true, true, 30),
        (8, 9),
        "rule 4: 20 - 9 for the marker - 3 for +n"
    );
}

#[test]
fn a_short_branch_hands_its_leftover_to_the_marker() {
    // "main" needs 4 of its 11, so the marker grows: 9 + 7 = 16. A long path
    // behind a short branch name shows more of itself.
    assert_eq!(
        calculate_char_budget(true, false, 4),
        (4, 16),
        "rule 5: branch gets its length, marker fills the rest"
    );
}

#[test]
fn a_short_branch_hands_its_leftover_to_the_marker_with_plus_too() {
    // The branch budget is 20 - 9 - 3 = 8; "main" hands 4 back: 9 + 4 = 13.
    assert_eq!(
        calculate_char_budget(true, true, 4),
        (4, 13),
        "rule 6: rule 5 counts the +n as well"
    );
}

#[test]
fn at_the_budget_edge_the_split_stays_fixed() {
    // 11 is not *shorter* than the branch budget of 20 - 9 = 11, so rule 5
    // does not kick in and the marker stays at 9.
    assert_eq!(
        calculate_char_budget(true, false, 11),
        (11, 9),
        "exactly at the budget: fixed split"
    );
    // Longer branches truncate to their budget instead of growing the marker.
    assert_eq!(
        calculate_char_budget(true, false, 12),
        (11, 9),
        "over the budget: fixed split"
    );
}
