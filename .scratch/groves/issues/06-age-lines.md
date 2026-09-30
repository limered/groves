# 06: Age lines

**Parent:** spec 015

**What to build:** Horizontal **Age lines** at 3h, 1d, 1w and 3w by committer date. Each line is drawn once, above the first row from the top that is older than its threshold, and labelled at the right edge (`── 1d ago`). Thresholds that no row reaches are skipped, and rows that are out of order are ignored.

**Blocked by:** 02

**Status:** ready-for-agent

**Rust focus:** pure functions, injecting "now" so tests are deterministic, `Duration` arithmetic.

- [ ] Line positions are correct for a synthetic history with a fixed "now"
- [ ] Thresholds that no row reaches produce no line
- [ ] An out-of-order old row near the top doesn't pull a line up
- [ ] Manual: lines and labels read cleanly and don't overlap the pills
