# 08: Background loading

**Parent:** spec 015

**What to build:** Opening a repository never freezes the UI. The pane shows a loading state while a worker thread builds the DAG and the layout, then swaps in the new `Arc<Layout>` atomically. Relayouts also run off the UI thread, and the old layout keeps being drawn until the new one is ready. The repo handle is opened inside the task and dropped when the task ends.

**Blocked by:** 03

**Status:** in-progress

**Rust focus:** `std::thread`, `mpsc` channels, `Arc`, what `Send`/`Sync` mean and why the compiler demands them, requesting an egui repaint from another thread. The mentor starts with a short primer.

- [ ] A load/relayout job API that returns results over a channel (tests without egui)
- [ ] The UI thread never calls gix
- [ ] A stale result (an older generation) is discarded instead of overwriting a newer one
- [ ] Manual: the window stays responsive while the Factory repo opens

## Manual checks

- [ ] `cargo run -p groves-app -- <path to a big repo>`: the window appears immediately with a loading placeholder (today it doesn't appear at all until the pane is loaded), and the graph arrives on its own — without touching the mouse (the worker's repaint request at work).
- [ ] While loading, the window stays responsive: drag it, resize it, scroll — the UI thread never blocks.
- [ ] The repo handle story: the load task opens the repo itself and drops it at task end; nothing gix-related is reachable from the UI thread (review gate, not a manual step).

**Known simplification (accepted for this ticket):** a panicking worker never sends a result, so `busy` would stay true. Revisited when the fetch scheduler (17) starts sharing the runner.
