//! Ticket 05 red test: jump-to-HEAD cycling is a pure step — first press
//! lands on main, repeated presses cycle through the other worktrees and
//! wrap around. Owned by the mentor. Emil makes it pass in `crates/app`.

use groves_app::head_jump::next_head_jump;

#[test]
fn jump_cycles_main_then_others_then_wraps() {
    // Targets come from core::Pane::head_jump_rows(): worktree order,
    // main's row first. Row numbers need not be ascending.
    let targets = [3usize, 7, 1];

    assert_eq!(
        next_head_jump(None, &targets),
        Some((0, 3)),
        "first press: the main worktree's row"
    );
    assert_eq!(next_head_jump(Some(0), &targets), Some((1, 7)));
    assert_eq!(next_head_jump(Some(1), &targets), Some((2, 1)));
    assert_eq!(
        next_head_jump(Some(2), &targets),
        Some((0, 3)),
        "the next press after the last worktree wraps back to main"
    );
}

#[test]
fn jump_is_a_noop_without_worktree_heads() {
    assert_eq!(next_head_jump(None, &[]), None, "nothing to jump to");
    assert_eq!(next_head_jump(Some(0), &[]), None);
}
