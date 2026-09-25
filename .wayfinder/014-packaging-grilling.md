---
id: 014
title: How is the viewer packaged and distributed?
labels: [wayfinder:grilling]
parent: 000
blocked_by: [013]
assignee: emil.bohleber

status: closed
---

## Question

Decide packaging for Windows + Linux: single static-ish binary vs. installer/MSI/AppImage/deb, dependency on system git for fetch, OpenGL requirements, update mechanism (none?), and the typical install location (013 decided: data goes to `<exe dir>/commit-graph-data/` when writable, else OS dirs - so packaging decides which case is normal).


## Resolution

- **Artifact**: one portable binary per OS: a zip holding the exe on Windows, a tarball holding the binary on Linux. No installer. The normal data location is `<exe dir>/commit-graph-data/`; the OS-folder fallback applies only when that location can't be written to.
- **System git**: optional. Visualization is the core and runs entirely on gix. Look for git on `PATH`, or at `git_path` in `config.toml` if set. If git is missing, fetching is turned off and the pane header shows "git not found"; the user can still fetch outside the app, and the ref poll picks up the change.
- **Graphics**: OpenGL only (egui + glow, GL 3.3 / GLES 3.0). If that isn't available, show a clear error dialog naming the GL version found. No wgpu or software fallback.
- **Linux**: Wayland and X11 both built into one binary (winit default). Built with `cargo zigbuild` against glibc 2.31; musl rejected (GL/Wayland libraries are loaded at runtime).
- **Updates**: none; replace the file by hand. The only network operation stays `fetch`.
- **Windows extras**: embedded icon and version resource, per-monitor DPI awareness. No code signing, no Start-menu shortcut.
- **Release source**: the owner builds releases locally with one documented script (`cargo build --release` + zigbuild). GitHub Actions possibly later (tracked with CI under Not yet specified).
