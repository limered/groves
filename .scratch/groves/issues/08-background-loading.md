# 08: Background loading

**Parent:** spec 015

**What to build:** Opening a repository never freezes the UI. The pane shows a loading state while a worker thread builds the DAG and the layout, then swaps in the new `Arc<Layout>` atomically. Relayouts also run off the UI thread, and the old layout keeps being drawn until the new one is ready. The repo handle is opened inside the task and dropped when the task ends.

**Blocked by:** 03

**Status:** ready-for-agent

**Rust focus:** `std::thread`, `mpsc` channels, `Arc`, what `Send`/`Sync` mean and why the compiler demands them, requesting an egui repaint from another thread. The mentor starts with a short primer.

- [ ] A load/relayout job API that returns results over a channel (tests without egui)
- [ ] The UI thread never calls gix
- [ ] A stale result (an older generation) is discarded instead of overwriting a newer one
- [ ] Manual: the window stays responsive while the Factory repo opens
