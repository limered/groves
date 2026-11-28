# Changelog

User-facing changes per release. Each section is written by the `release` skill and also serves as the text of the GitHub Release.

## [0.6.0] - 2026-11-28

### Added
- Opening a repository no longer freezes the window: the graph loads in the background while a loading placeholder shows, so big repositories open into a responsive window that stays usable during the load. (08)

## [0.5.0] - 2026-11-24

### Added
- Worktree HEAD markers: every worktree of the repository shows its marker ring on the graph — the main worktree is named after its repo folder, linked worktree names show on hover or click, and a detached HEAD shows its short SHA. Opening a linked worktree folder draws the same graph as the main one. (05)
- `h` jumps to the main worktree's HEAD row, centred; pressing it again cycles through the other worktree heads and wraps back around. (05)
- Clicking a commit's short hash copies the full SHA to the clipboard. (05)

### Changed
- Nicer pill icons (synced ⇅ → 🔘, new folder icon for worktree markers), and the 20-character pill budget is now shared dynamically between branch pill, `+n` badge and worktree marker: short branch names hand their leftover space to the marker instead of truncating early. (05)

### Fixed
- Opening a repository path like `.` or with a trailing slash no longer panics. (05)

## [0.4.1] - 2026-11-16

### Fixed
- Launching groves with no arguments now opens the repository at the current working directory, so the .exe can be started by double-clicking it instead of from the command line.

## [0.4.0] - 2026-11-16

### Added
- Branch pills: rows with branches show their branch name in a fixed column left of the graph — filled for local branches, outlined for remotes, one synced pill (⇅) when a branch and its upstream point at the same commit, `+N` for further refs, and a thin connector to the row's dot. (04)
- Lane colours: graph lines, dots and pills are now drawn in per-lane colours from a woody palette, so neighbouring branch lines can be told apart at a glance. (04)

## [0.3.1] - 2026-11-03

### Fixed
- Edge lines no longer thread straight through the commits they pass: a line keeps its own lane and bends once into the parent's line, so branch lines no longer fuse with merge curves on busy graphs — bends read like `git log --graph`.

### Changed
- Edges bend with one constant rounded corner plus a straight run, the same shape at every lane distance, instead of a full-gap curve.

## [0.3.0] - 2026-11-02

### Added
- Panes draw only history reachable from the default ref set — local branches, their upstreams, `origin/HEAD`, every worktree head and remote branches updated in the last 14 days. Stale remote branches, tags and history only they reach no longer bury the graph on large repositories.

### Fixed
- Repositories managed by GitKraken no longer fail to open: refs that don't peel to a commit are skipped.

## [0.2.0] - 2026-10-26

### Added
- View a repository's HEAD commit graph with branch and merge edges; the HEAD/main path stays leftmost, and long histories draw only visible rows for smooth scrolling.

## [0.1.0]

### Added
- Walking skeleton: `groves <path-to-repo>` opens a window that lists the commits reachable from HEAD, newest first, each with its short SHA and title. If the path is not a repository, the window shows the error.
- Portable release builds: a Windows zip and a Linux tarball (glibc ≥ 2.31, Wayland + X11).
