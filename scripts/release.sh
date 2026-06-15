#!/usr/bin/env bash
#
# scripts/release.sh: release preparation wrapper for isochron.
#
# Usage:
#   bash scripts/release.sh <patch|minor|major|x.y.z>
#
# What it does (in order):
#   1. Pre-flight: must be on main, working tree must be clean, main must
#      be in sync with origin/main.
#   2. Computes the target version from the requested bump level.
#   3. Creates branch release/v<TARGET>-prep.
#   4. Graduates CHANGELOG.md: renames the current "(unreleased)" section
#      to the dated release heading, prepends a fresh unreleased section.
#   5. Runs cargo-release to bump the version in Cargo.toml in a single
#      commit (publish=false, tag=false, push=false per release.toml).
#   6. Pre-flight checks: cargo fmt --check, clippy strict, full test suite.
#   7. Pushes the branch and opens a pull request via gh.
#
# After the reviewer merges the PR, push the tag manually:
#
#   git tag -a v<TARGET> -m "v<TARGET>"
#   git push origin v<TARGET>
#
# That tag triggers .github/workflows/release.yml which publishes to
# crates.io and creates the GitHub Release.
#
# Dependencies on the runner: bash, git, cargo, cargo-release, gh, python3.

set -euo pipefail

LEVEL="${1:-}"
if [[ -z "${LEVEL}" ]]; then
  cat >&2 <<'USAGE'
Usage: bash scripts/release.sh <patch|minor|major|x.y.z>

Examples:
  bash scripts/release.sh patch          # 0.1.0 -> 0.1.1
  bash scripts/release.sh minor          # 0.1.0 -> 0.2.0
  bash scripts/release.sh major          # 1.2.3 -> 2.0.0
  bash scripts/release.sh 0.3.0          # explicit version
USAGE
  exit 1
fi

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "${REPO_ROOT}"

# 1. Pre-flight
CURRENT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"
if [[ "${CURRENT_BRANCH}" != "main" ]]; then
  echo "error: must be on main (current: ${CURRENT_BRANCH})" >&2
  exit 1
fi
if [[ -n "$(git status --porcelain)" ]]; then
  echo "error: working tree must be clean" >&2
  git status --short >&2
  exit 1
fi
git pull --ff-only origin main

# 2. Compute target version
CURRENT_VERSION="$(grep -m1 -E '^version = "[0-9]+\.[0-9]+\.[0-9]+"' Cargo.toml | sed -E 's/^version = "([^"]+)".*/\1/')"
if [[ -z "${CURRENT_VERSION}" ]]; then
  echo "error: cannot read current package version from Cargo.toml" >&2
  exit 1
fi

if [[ "${LEVEL}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  NEW_VERSION="${LEVEL}"
else
  case "${LEVEL}" in
    patch)
      NEW_VERSION="$(python3 -c "import sys; v=sys.argv[1].split('.'); v[2]=str(int(v[2])+1); print('.'.join(v))" "${CURRENT_VERSION}")"
      ;;
    minor)
      NEW_VERSION="$(python3 -c "import sys; v=sys.argv[1].split('.'); v[1]=str(int(v[1])+1); v[2]='0'; print('.'.join(v))" "${CURRENT_VERSION}")"
      ;;
    major)
      NEW_VERSION="$(python3 -c "import sys; v=sys.argv[1].split('.'); v[0]=str(int(v[0])+1); v[1]='0'; v[2]='0'; print('.'.join(v))" "${CURRENT_VERSION}")"
      ;;
    *)
      echo "error: unknown level '${LEVEL}' (use patch|minor|major or x.y.z)" >&2
      exit 1
      ;;
  esac
fi

echo "Bumping ${CURRENT_VERSION} -> ${NEW_VERSION}"

# 3. Create release branch
BRANCH="release/v${NEW_VERSION}-prep"
if git show-ref --quiet --verify "refs/heads/${BRANCH}"; then
  echo "error: branch ${BRANCH} already exists locally" >&2
  exit 1
fi
git checkout -b "${BRANCH}"

# 4. Graduate CHANGELOG.md
DATE="$(date -u +%Y-%m-%d)"
python3 - "${NEW_VERSION}" "${DATE}" <<'PYEOF'
import pathlib
import re
import sys

new_version = sys.argv[1]
date = sys.argv[2]
path = pathlib.Path("CHANGELOG.md")

if not path.exists():
    raise SystemExit("CHANGELOG.md not found")

content = path.read_text(encoding="utf-8")

# Match a heading of the form "## X.Y.Z (unreleased)" (any version in the unreleased slot).
unreleased_re = re.compile(r"^## \d+\.\d+\.\d+ \(unreleased\)", re.MULTILINE)
m = unreleased_re.search(content)
if m is None:
    # If the pattern is not found, emit a warning and leave CHANGELOG unchanged.
    print(
        "warning: CHANGELOG.md has no '## x.y.z (unreleased)' heading; "
        "leaving it unchanged",
        file=sys.stderr,
    )
    sys.exit(0)

# Replace the matched heading with a dated heading and prepend a new unreleased section.
dated_heading = f"## {new_version} ({date})"
new_unreleased_block = (
    f"## {new_version} (unreleased)\n\n"
    f"### Added\n\n"
    f"- _Nothing yet._\n\n"
)
content = unreleased_re.sub(lambda _: new_unreleased_block + dated_heading, content, count=1)

path.write_text(content, encoding="utf-8")
print(f"CHANGELOG.md graduated: {new_version} (unreleased) -> {new_version} ({date})")
PYEOF

# 4b. Commit the graduated CHANGELOG before invoking cargo-release, which
# refuses to operate on a dirty working tree.
git add CHANGELOG.md
git commit -m "docs(changelog): graduate ${NEW_VERSION} unreleased to ${DATE}"

# 5. Bump the package version in Cargo.toml (no --workspace: single crate).
cargo release "${LEVEL}" --execute --no-confirm

# 6. Pre-flight checks
echo "Running cargo fmt --check"
cargo fmt -- --check
echo "Running cargo clippy --all-targets --all-features -- -D warnings"
cargo clippy --all-targets --all-features -- -D warnings
echo "Running cargo test --all-features"
cargo test --all-features

# 7. Push branch and open the pull request
git push -u origin "${BRANCH}"

PR_BODY=$(cat <<EOF
Pre-flight release prep generated by scripts/release.sh.

## Pre-flight (passed locally)

- cargo fmt -- --check
- cargo clippy --all-targets --all-features -- -D warnings
- cargo test --all-features

## Tagging instructions

After this PR is merged, push the v${NEW_VERSION} tag to fire .github/workflows/release.yml:

\`\`\`
git tag -a v${NEW_VERSION} -m "v${NEW_VERSION}"
git push origin v${NEW_VERSION}
\`\`\`

The release workflow then publishes the crate to crates.io and creates the GitHub Release.
EOF
)

gh pr create \
  --title "release(v${NEW_VERSION}): bump isochron and finalise release notes" \
  --body "${PR_BODY}" \
  --label "kind:chore,phase:release" \
  --base main \
  --head "${BRANCH}"

cat <<EOF

=========================================
Release prep PR opened for v${NEW_VERSION}.

Next steps:
  1. Review and merge the PR.
  2. git tag -a v${NEW_VERSION} -m "v${NEW_VERSION}"
  3. git push origin v${NEW_VERSION}
=========================================
EOF
