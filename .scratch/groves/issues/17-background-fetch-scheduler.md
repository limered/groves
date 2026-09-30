# 17: Background fetch scheduler

**Parent:** spec 015

**What to build:** Every open pane in every grove is fetched in the background every `fetch_interval_minutes`, with the first fetch about 10 s after the pane opens. At most 2 fetches run at once, round-robin, and never two for the same repo. Manual fetches jump the queue. Background fetches are silent (`GIT_TERMINAL_PROMPT=0`, `GCM_INTERACTIVE=never`). After 3 consecutive failures the interval doubles, capped at 1 h, until a success or a manual `r`.

**Blocked by:** 16

**Status:** ready-for-agent

**Rust focus:** modelling the scheduler as a state machine over enums, driving it with an injected clock so it's testable without sleeping.

- [ ] Queue order, max-2 concurrency, one fetch per repo (tests with a fake clock and a fake fetcher)
- [ ] Manual fetches jump the queue (tests)
- [ ] Backoff doubles after 3 failures, caps at 1 h, resets on success or manual fetch (tests)
- [ ] Manual: idle CPU stays under 1% with 3 panes open
