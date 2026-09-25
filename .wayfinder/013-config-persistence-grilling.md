---
id: 013
title: What is persisted, where and in what format?
labels: [wayfinder:grilling]
parent: 000
blocked_by: []
assignee: emil.bohleber
status: closed
---

## Question

Decide the config and persistence model: what is persisted (workspace layout + repo per pane from 006, fetch interval, staleness window, keymap?), file format (TOML/JSON/RON), location per OS (Windows/Linux), per-repo vs. global settings, and handling of missing repos or corrupt files on startup.

## Resolution

Grilled 2026-09-25.

- **Two files, strictly separated by writer:**
  - `config.toml` - hand-edited settings. Global only (no per-repo overrides). Keys: `fetch_interval_minutes = 5`, `stale_after_days = 14`. Everything else (ref-poll 10 s, age-line thresholds, keymap) stays hard-coded. No settings UI. The app writes it exactly once: on first start, with commented defaults; never again (user comments survive).
  - `state.json` - app-written. Contains: `version`, workspaces, pane tree + split ratios, repo per pane, window position/size/maximized, last active workspace. Not persisted: per-pane view state (009), lane-hint maps, recent-repo list.
- **Format:** TOML for config; JSON (serde) for state. A DB (SQLite) was considered for state and rejected: state is < 10 KB, JSON parse is sub-millisecond, a DB adds ~1 MB C dependency, migrations and loses hand-repairability.
- **Location:** portable first: `<exe dir>/commit-graph-data/{config.toml,state.json}` if that directory is writable; otherwise fall back to OS dirs via `directories` (`%APPDATA%\commit-graph\` on Windows; `$XDG_CONFIG_HOME/commit-graph/` for config, `$XDG_STATE_HOME/commit-graph/` for state on Linux).
- **Write timing:** state written debounced (~1 s) on every layout change plus once on exit; atomic (temp file + rename).
- **Failure handling:**
  - Missing repo -> "missing, remove?" placeholder (006).
  - Config parse error -> run on defaults, warning badge naming the failing line, never overwrite the file.
  - Corrupt/incompatible state -> rename to `state.json.bak-<timestamp>`, start with one empty workspace, one-line notice. Unknown keys ignored.
