# Release process

isochron ships new versions to crates.io through a single tool: the `xtask`
binary in this workspace (`xtask/`, a dependency-free member of `[workspace]`).
Whatever surface you choose, the underlying flow is identical: bump the
version, graduate the changelog, run local checks, open a release
preparation pull request, then push a tag that fires the publish workflow.

## Surfaces

### Surface 1: manual dispatch via the UI

Use this when you want to bump from your browser, or when you do not have a
local Rust toolchain handy.

1. Open the Actions tab and select the **Bump** workflow.
2. Click **Run workflow**.
3. Pick the **level** input:
   - `patch`: `0.1.0` to `0.1.1` (bug fixes)
   - `minor`: `0.1.0` to `0.2.0` (breaking changes allowed in 0.x per
     [SEMVER_POLICY.md](SEMVER_POLICY.md))
   - `major`: `1.2.3` to `2.0.0` (breaking changes in 1.x+)
   - explicit `x.y.z`: e.g. `0.2.0`
4. Optionally set **dry_run** to prepare the release locally in the runner,
   without pushing the branch or opening a pull request.
5. The workflow runs `cargo xtask release-prep` in CI and, unless dry run was
   requested, opens a release preparation pull request.
6. Review the PR, merge it, then follow [Tagging](#tagging).

### Surface 2: local xtask command

Use this when you want full control over the pre-flight (run tests locally,
inspect the changelog before it graduates, and so on).

```sh
cargo xtask release-prep patch           # or minor / major / 0.2.0
cargo xtask release-prep patch --dry-run # prepare locally, skip push and PR
```

Requirements (must be installed on your machine): `git`, `cargo`, and `gh`
authenticated for this repository. No Python, no `cargo-release`.

`cargo xtask release-prep <level>` does, in order:

1. Refuses to run if you are not on `main`, or if the working tree is dirty.
2. Pulls `origin/main` fast-forward only.
3. Reads the current version from `Cargo.toml` and resolves the target
   version from the requested level.
4. Graduates `CHANGELOG.md` in memory: the `## [Unreleased]` section becomes
   `## [X.Y.Z] - YYYY-MM-DD`, a fresh empty `## [Unreleased]` section is
   reopened above it, and the `[Unreleased]` and `[X.Y.Z]` compare links at
   the bottom of the file are updated accordingly. This step fails, before
   touching the working tree, if the changelog does not have exactly one
   non-empty `## [Unreleased]` section with its compare link (see
   [Failure modes](#failure-modes)).
5. Creates the branch `chore/release-vX.Y.Z` (compliant with the branch
   naming in [GOVERNANCE.md](GOVERNANCE.md)).
6. Writes the graduated changelog and the bumped `Cargo.toml`, and commits
   both in a single commit.
7. Runs `cargo fmt --all --check`, `cargo clippy --workspace --all-targets
   --all-features -- -D warnings`, then `cargo test --workspace
   --all-features`.
8. Unless `--dry-run` was passed, pushes the branch and opens a pull request
   with `gh pr create`.

## Tagging

After the release preparation pull request is merged into `main`, push the
tag manually:

```sh
git checkout main
git pull origin main
git tag -a v<TARGET> -m "v<TARGET>"
git push origin v<TARGET>
```

The tag push triggers `.github/workflows/release.yml`, which:

1. Publishes `isochron` to crates.io.
2. Creates a GitHub Release whose notes come from `cargo xtask release-notes
   <TARGET>`, which extracts the `## [<TARGET>]` section of `CHANGELOG.md` by
   exact heading comparison. A missing or empty section fails the release
   job instead of publishing placeholder notes.

Tagging is deliberately a manual step so the human reviewing the PR is also
the one who triggers the publish, with full awareness of what is about to
leave the workshop.

## What release preparation does NOT do

- It does not publish to crates.io. The tag does, via `release.yml`.
- It does not create the release. The tag does.
- It does not invent changelog content. Whatever you wrote under
  `[Unreleased]` is preserved verbatim under the new `[X.Y.Z]` section.
- It does not skip checks. If `cargo fmt`, `clippy` or the test suite fails,
  the command stops and the working tree is left on the release branch for
  inspection.

## Failure modes

- **`Must be on branch main to prepare a release`**: switch back to `main`,
  then retry.
- **`The working tree has uncommitted changes`**: commit or stash your local
  changes.
- **`CHANGELOG.md has no [Unreleased] heading`**: add a `## [Unreleased]`
  heading above the latest release.
- **`CHANGELOG.md has a heading that looks like the unreleased section but
  is not exactly [Unreleased]`**: rename the offending heading (for example
  `## Unreleased` or `## [unreleased]`) to exactly `## [Unreleased]`.
- **`The [Unreleased] section in CHANGELOG.md has no content`**: add at
  least one bullet under `## [Unreleased]` before releasing.
- **`Version already has a released section in CHANGELOG.md`**: the target
  version was already released; choose a different level or version.
- **`CHANGELOG.md has no [Unreleased] link reference definition`**: add a
  `[Unreleased]: .../compare/vX.Y.Z...HEAD` line at the bottom of the file.
- **A `cargo fmt`, `clippy` or test failure**: fix it on `main` first via a
  normal pull request, then retry the release preparation.
- **`gh pr create` fails**: verify `gh` authentication. In CI, the
  `GITHUB_TOKEN` is provided automatically.
- **`CHANGELOG.md has no [X.Y.Z] section` (at release time)**: the tag was
  pushed for a version that was never graduated into the changelog; fix the
  changelog on `main` and re-tag.
