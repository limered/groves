# Changelog

User-facing changes per release. Each section is written by the `release` skill and also serves as the text of the GitHub Release.

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
