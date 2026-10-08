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

1. Bump `version` in `crates/app/Cargo.toml` (and the other crates for consistency), run `cargo build` to update `Cargo.lock`, then commit. CI builds with `--locked`.
2. Tag and push. The tag must match the app version exactly, or the workflow fails:

   ```sh
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

3. The CD workflow (`.github/workflows/cd.yml`) runs the tests on Windows and Linux, builds both portable archives and publishes them as a GitHub Release with generated notes.

To test packaging without publishing a release, start the workflow manually (`workflow_dispatch`). The archives are then attached only to the workflow run.
