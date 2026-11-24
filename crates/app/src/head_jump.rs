pub fn next_head_jump(current: Option<usize>, targets: &[usize]) -> Option<(usize, usize)> {
    if targets.is_empty() {
        return None;
    }
    if let Some(current) = current {
        let next_index = current.saturating_add(1) % targets.len();
        Some((next_index, targets[next_index]))
    } else {
        Some((0, targets[0]))
    }
}
