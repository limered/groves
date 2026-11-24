# AGENTS.md

groves: a read-only commit-graph viewer in Rust (egui/eframe 0.36, gix). Learning project: Emil writes the product code.

## Ground rules
- Read `.opencode/agent/groves-mentor.md` before working on a ticket. Agents write tests, `crates/testkit` and tickets. Do not write product code unless Emil explicitly asks for it ("show me").
- Once a test is red, do not edit it to make it pass. Fix the product code. If the test itself is wrong, say so openly.
- `.wayfinder/015-commit-graph-viewer-spec.md` is the spec and decides behaviour, architecture and budgets. Other `.wayfinder/*` files are earlier research and decisions.
- `CONTEXT.md` is the glossary. Use its terms (pane, grove, ref set, pill, pin) in code and tests. Never call a pane a "tree".
- Tickets: `.scratch/groves/issues/NN-*.md`. Each has `Status:` and `Blocked by:` fields. The frontier is every ticket whose blockers are all done.

## Layout
- Cargo workspace: `crates/core` (domain, no gix or egui), `crates/repo` (gix adapter implementing `CommitSource`), `crates/app` (egui UI, binary `groves`), `crates/testkit` (dev-only fixtures).
- Dependencies point inward: app → repo → core. Do not add gix or egui to `core`.
- `prototype/` is a **separate** workspace (UI throwaways, graphcore). The root `cargo` commands do not build it. Use it as a reference, not as code to copy.
- The root `tests/scroll.rs` belongs to no crate, so cargo never compiles it. It is a scratch file.
- No async runtime. Heavy work runs on worker threads, and the UI thread only draws.

## Commands
- The package names have a `groves-` prefix, and the mentor file's `-p app` is wrong. Run the app with `cargo run -p groves-app -- <repo-path>`.
- Single crate tests: `cargo test -p groves-core` / `-p groves-repo`. Single test file: `cargo test -p groves-repo --test graph`.
- Review gate: `cargo test --workspace`, `cargo clippy --all-targets`, `cargo fmt --check`.
- The testkit fixtures shell out to `git` (`git fast-import`), so `git` must be on PATH.
- The UI has no automated tests. Tickets list UI behaviour under `## Manual checks`.

## Branches & release
- Work happens on `dev`. A merge of `dev` into `main` **is** a release. Never commit feature work directly to `main`.
- Release only through the `release` skill (`.opencode/skills/release/SKILL.md`). It merges dev into main, bumps the version in all four crates (in sync), writes the `CHANGELOG.md` section from the tickets closed since the last tag, then tags and pushes, and fast-forwards dev.
- The CD release text is the `## [X.Y.Z]` section of `CHANGELOG.md`, so keep that heading format exact.
- `.github/workflows/cd.yml` runs on `v*` tags. The tag must equal the `version` in `crates/app/Cargo.toml`, or the job fails. CI uses `--locked`, so commit `Cargo.lock`.
- The Linux build uses `cargo zigbuild` targeting glibc 2.31. Windows produces a zip containing `groves.exe`.

## Architecture rules

- Organize by feature, then responsibility. Prefer `commit_graph/{selection,layout,presentation}.rs` over project-wide `services/` and `models/`. Split when responsibilities differ, not at an arbitrary file size.
- Separate I/O from computation. Load data, read clocks, and wire dependencies at application boundaries; pass explicit inputs into core logic. Example: `build(commits, refs, options)` instead of `Pane::load(source)` reading the clock internally.
- Keep policy, algorithms, and rendering distinct. Example: selection decides which commits appear; layout assigns rows and lanes; presentation creates labels; the renderer owns pixel dimensions.
- Make dependencies explicit and directional. Core rules must not depend on UI or storage implementations. Pass required dependencies rather than global state or an oversized application context.
- Prefer simple functions and concrete types. Add traits only for meaningful interchangeable dependencies; add crates only when an enforceable boundary justifies them. Do not create an abstraction for every struct or pipeline stage.
- Minimize exposed APIs. Keep implementation modules and fields private; use `pub(super)` or `pub(crate)` where sufficient, and re-export only intended entry points.
- Make contracts explicit. Return named intermediate results instead of ambiguous tuples; distinguish IDs, indices, and coordinates when confusion is plausible. Example: edge row indices must address the final emitted rows.
- Define failure behavior consistently. Never silently replace a missing required value with an unrelated default. Explicitly choose whether incomplete input is valid, skipped, or an error.
- Test responsibilities and invariants independently. Example: test visibility thresholds separately from lane placement; assert every edge references an existing row. Define deterministic tie-breakers where output order matters.
- Refactor incrementally. Capture existing behavior, extract one responsibility at a time, and separate structural changes from bug fixes or policy changes. Avoid speculative infrastructure.
