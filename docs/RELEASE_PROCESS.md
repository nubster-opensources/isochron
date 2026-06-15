# Release process

isochron uses a two-layer combo to ship new versions to crates.io. Whatever
surface you choose, the underlying flow is identical: bump the version,
graduate CHANGELOG, pre-flight checks, open a release prep PR, then push a
tag that fires the publish workflow.

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
4. The workflow runs `scripts/release.sh` in CI and opens a release prep PR.
5. Review the PR, merge it, then follow [Tagging](#tagging).

### Surface 2: local script

Use this when you want full control over the pre-flight (run tests locally,
tweak CHANGELOG manually, etc.).

```sh
./scripts/release.sh patch          # or minor / major / 0.2.0
```

Requirements (must be installed on your machine):

- `bash`, `git`, `python3`, `gh`
- `cargo` and the `cargo-release` subcommand:
  `cargo install --locked cargo-release@0.25.20`

The script:

1. Refuses to run if you are not on `main` or if the working tree is dirty.
2. Pulls `origin/main`.
3. Computes the target version from the bump level.
4. Creates `release/v<TARGET>-prep` branch.
5. Graduates `CHANGELOG.md`: renames the current `## x.y.z (unreleased)`
   heading to a dated `## x.y.z (YYYY-MM-DD)` heading, and prepends a fresh
   `## x.y.z (unreleased)` section for the next development cycle.
6. Runs `cargo release <LEVEL> --execute --no-confirm` which bumps the
   `Cargo.toml` version field in a single consolidated commit.
7. Runs `cargo fmt --check`, `clippy` strict, full test suite.
8. Pushes the branch and opens a PR.

## Tagging

After the release prep PR is merged into `main`, push the tag manually:

```sh
git checkout main
git pull origin main
git tag -a v<TARGET> -m "v<TARGET>"
git push origin v<TARGET>
```

The tag push triggers `.github/workflows/release.yml`, which:

1. Publishes `isochron` to crates.io.
2. Creates a release whose notes are extracted from the `## <TARGET> (<DATE>)`
   section of `CHANGELOG.md`.

Tagging is deliberately a manual step so the human reviewing the PR is also
the one who triggers the publish, with full awareness of what is about to
leave the workshop.

## What the bump script does NOT do

- It does not publish to crates.io. The tag does, via `release.yml`.
- It does not create the release. The tag does.
- It does not edit your `[Unreleased]` items. Whatever you wrote there is
  preserved verbatim under the new `[<TARGET>]` section.
- It does not skip pre-flight checks. If `cargo fmt` or `clippy` or the test
  suite fails, the bump aborts.

## Failure modes

- **`error: must be on main`**: switch back to main, then retry.
- **`error: working tree must be clean`**: commit or stash your local changes.
- **Pre-flight failure**: fix the failure on `main` first via a normal PR,
  then retry the bump.
- **`gh pr create` fails**: verify authentication status. In CI the
  `GITHUB_TOKEN` is provided automatically.
- **CHANGELOG section not found**: the Python script expects a heading matching
  `## x.y.z (unreleased)`. If that heading is absent or has been renamed, the
  script emits a warning and leaves CHANGELOG.md unchanged.
