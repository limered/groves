---
name: release
description: Release groves - merge dev into main, bump the workspace version (minor by default, or the version the user gives), write the CHANGELOG.md section from the tickets closed since the last release, commit, tag vX.Y.Z and push. Use when the user says "release", "cut a release" or "ship vX.Y.Z".
---

# release

Merging `dev` into `main` is a release. Pushing the tag `vX.Y.Z` starts `.github/workflows/cd.yml`. It builds the portable archives and publishes a GitHub Release, using this version's `CHANGELOG.md` section as the release text.

Argument: an optional version `X.Y.Z` (a leading `v` is allowed and gets stripped). Without one, bump the minor version and reset the patch: `0.1.0 -> 0.2.0`, `0.2.3 -> 0.3.0`.

## 1. Preflight (stop on any failure, change nothing)

- `git status --porcelain` is empty. The working tree is clean.
- `git fetch origin --tags`. Local `dev` and `main` are not behind `origin`. If they are, ask before fast-forwarding them.
- `PREV` = the latest `v*` tag (`git describe --tags --abbrev=0 --match "v*" origin/main`, or the highest tag from `git tag --list "v*" --sort=-v:refname`).
- `git log --oneline main..dev` is not empty. Otherwise there is nothing to release.
- `CUR` = `version` in `crates/app/Cargo.toml`. Compute `NEW`. A version the user gives must be valid semver and greater than `CUR`, and the tag `vNEW` must not exist yet.
- `cargo test --workspace` and `cargo clippy --all-targets` pass on `dev`. The CD workflow runs the tests too, but a red tag is annoying to undo.

## 2. Collect the closed tickets

Tickets are `.scratch/groves/issues/NN-*.md` with a `**Status:** done` line. A ticket counts as closed in this release if its status became done since the last tag:

```sh
git diff PREV..dev --name-only -- .scratch/groves/issues
git diff PREV..dev -- .scratch/groves/issues/<file>   # look for an added "+**Status:** done"
```

If there is no previous tag, use every ticket that is currently done. For each closed ticket, read the title (the `# NN: ...` heading) and `**What to build:**`. Then write **one user-facing line**: what someone using groves can now do or see. Leave out Rust internals, `## Learned` notes and test details.

Also scan `git log --oneline PREV..dev` for user-visible changes that belong to no ticket, such as packaging or fixes. Put them under `### Other` only if they matter to a user.

## 3. Write the release on main

```sh
git switch main
git merge --no-ff dev -m "release vNEW"
```

If there are conflicts, stop and show them. Do not resolve product code on your own.

Then make the changes below on `main`. Amend the merge commit with them, or make one follow-up commit `release vNEW: bump version and changelog`:

- Set `version = "NEW"` in **all four** crates: `crates/{core,repo,app,testkit}/Cargo.toml`. They are kept in sync.
- Run `cargo update --workspace` to refresh `Cargo.lock`. CD builds with `--locked`, so a stale lock fails the release.
- Add the new section to the top of `CHANGELOG.md`, below the intro. Create the file if it is missing. The CD workflow extracts the section by the exact heading `## [NEW]`:

  ```markdown
  ## [NEW] - YYYY-MM-DD

  ### Added
  - <ticket NN: user-facing line>

  ### Other
  - ...
  ```

  Use `### Added` for new capabilities, `### Changed` for changed behaviour and `### Fixed` for bug fixes. Leave out empty headings.

- Run `cargo build -p groves-app --locked` as a final sanity check.

## 4. Confirm, tag, push

Show the user the version change, the CHANGELOG section and `git log --oneline origin/main..main`. **Ask once** before pushing. Then:

```sh
git tag -a vNEW -m "groves vNEW"
git push origin main
git push origin vNEW
```

## 5. Sync dev back

Merge `main` back into `dev` so that dev carries the version bump and the changelog, and the next release merges cleanly:

```sh
git switch dev
git merge --ff-only main   # dev has no new commits yet, so this fast-forwards
git push origin dev
```

Finish by linking the Actions run: `https://github.com/limered/groves/actions`.
