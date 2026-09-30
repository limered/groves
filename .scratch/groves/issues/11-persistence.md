# 11: Persistence

**Parent:** spec 015

**What to build:** The app restores its setup at startup. `state.json` holds `version`, the groves, the pane trees with their split ratios, the repo per pane, the window geometry and the last active grove. It is written atomically (temp file + rename), debounced about 1 s after changes and once on exit. A corrupt or incompatible state is backed up as `state.json.bak-<timestamp>`, and the app starts with one empty grove and a one-line notice. `config.toml` (`fetch_interval_minutes`, `stale_after_days`, optional `git_path`) is created once with commented defaults and never rewritten. A bad config falls back to defaults with a warning naming the line, and `stale_after_days` replaces the hard-coded 14. Data lives in `<exe dir>/groves-data/` when that is writable, otherwise in the OS dirs. A pane whose repository has disappeared shows a "missing, remove?" placeholder.

**Blocked by:** 10

**Status:** ready-for-agent

**Rust focus:** serde derive (`Serialize`/`Deserialize`, `#[serde(default)]`), `toml`, file I/O, atomic rename, error fallbacks instead of panics, the `directories` crate.

- [ ] `state.json` round trip; unknown keys are ignored (tests)
- [ ] Corrupt state → backup file created, empty state returned (tests with a temp dir)
- [ ] Bad config → defaults plus a warning with the line number (tests)
- [ ] Portable vs OS-dir location choice (tests)
- [ ] Manual: quit and restart restores everything; a moved repo shows the placeholder
