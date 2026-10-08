# groves

A read-only commit-graph viewer for git repositories, built in Rust with egui and gix.

You can arrange several repository graphs into nine **groves** of tiled **panes**. Each pane draws the commit graph of one repository and marks the HEAD of every worktree. See `CONTEXT.md` for the glossary.

> Early development: the app is built ticket by ticket (`.scratch/groves/issues/`). The spec is in `.wayfinder/015-commit-graph-viewer-spec.md`.

## Run

```sh
cargo run -p groves-app -- <path-to-repo>
```

## Test

```sh
cargo test --workspace      # needs `git` on PATH (fixtures use git fast-import)
cargo clippy --all-targets
cargo fmt --check
```

## Layout

| Crate | Role |
|---|---|
| `crates/core` | Domain model and graph logic (no git or UI dependencies) |
| `crates/repo` | Reads repositories through gix |
| `crates/app` | egui desktop app, binary `groves` |
| `crates/testkit` | Test fixtures |

`prototype/` holds early UI experiments in a separate workspace.

## Releases

### Using a release (portable)

Download the archive for your OS from GitHub Releases:

- Windows: `groves-vX.Y.Z-x86_64-windows.zip` (contains `groves.exe`)
- Linux: `groves-vX.Y.Z-x86_64-linux-gnu.tar.gz` (contains `groves`; needs glibc ≥ 2.31, works on Wayland and X11)

Nothing needs to be installed. Unpack the archive and start the binary with the path to a repository:

```sh
./groves.exe <path-to-repo>   # Windows
./groves <path-to-repo>       # Linux
```

> This way of starting groves is temporary and will change in a later version.

### Publishing a release

Development happens on `dev`, and a merge into `main` is a release. Run the `release` skill in OpenCode (optionally with a version, e.g. `release 1.0.0`). By default it bumps the minor version, e.g. `0.1.0 → 0.2.0`. The skill:

1. checks that the working tree is clean and that tests and clippy pass on `dev`
2. merges `dev` into `main`
3. bumps the version in all crates and refreshes `Cargo.lock`
4. writes a new `CHANGELOG.md` section from the tickets closed since the last release
5. asks once, then tags `vX.Y.Z` and pushes `main` and the tag
6. fast-forwards `dev` to `main`

The tag starts the CD workflow (`.github/workflows/cd.yml`). It tests on Windows and Linux, builds both portable archives and publishes a GitHub Release whose text is the version's `CHANGELOG.md` section.

To test packaging without publishing a release, start the workflow manually (`workflow_dispatch`). The archives are then attached only to the workflow run.
