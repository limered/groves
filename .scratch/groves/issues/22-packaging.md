# 22: Packaging

**Parent:** spec 015

**What to build:** One portable binary per OS. Windows gets a zip with an embedded icon, version info and per-monitor DPI awareness. Linux gets a tarball built with `cargo zigbuild` against glibc 2.31, with winit's Wayland and X11 both enabled. If OpenGL 3.3 / GLES 3.0 is missing, an error dialog names the version found. One documented release script builds both.

**Blocked by:** 11

**Status:** ready-for-agent

**Rust focus:** `build.rs`, `cfg(target_os)`, release profiles, cross-compiling.

- [ ] The release script produces both archives
- [ ] Manual: the Windows exe shows its icon and is sharp on a HiDPI monitor
- [ ] Manual: the Linux binary runs on Wayland (Hyprland) and X11
- [ ] Manual: the GL error dialog appears when forced
