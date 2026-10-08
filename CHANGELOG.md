# Changelog

User-facing changes per release. Each section is written by the `release` skill and also serves as the text of the GitHub Release.

## [0.1.0]

### Added
- Walking skeleton: `groves <path-to-repo>` opens a window that lists the commits reachable from HEAD, newest first, each with its short SHA and title. If the path is not a repository, the window shows the error.
- Portable release builds: a Windows zip and a Linux tarball (glibc ≥ 2.31, Wayland + X11).
