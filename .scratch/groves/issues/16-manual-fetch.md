# 16: Manual fetch `r` / `Shift+R`

**Parent:** spec 015

**What to build:** `r` fetches the focused pane and `Shift+R` fetches all panes, by running system `git fetch --all --prune`, where credential prompts are allowed. git is found on `PATH` or at `git_path`. On Windows the process uses `CREATE_NO_WINDOW`. The pane header shows a spinner while fetching, a warning badge with the last error on hover, and "last fetched X ago". Without git, fetching is disabled and "git not found" is shown. No repo handle is held during the fetch. A successful fetch triggers the refresh path from 15.

**Blocked by:** 15

**Status:** ready-for-agent

**Rust focus:** `std::process::Command`, environment variables, `cfg(windows)` and `CommandExt`, capturing stderr into an error.

- [ ] Fetch from a local bare "remote" fixture brings in new commits (tests)
- [ ] Missing git → a clear error, and the viewer still works (tests)
- [ ] Manual: spinner, error badge, "last fetched"
