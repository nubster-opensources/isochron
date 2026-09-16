# Governance

isochron is a Nubster open-source project. Its governance is intentionally
minimal so that decisions can be made quickly while the project is still
pre-stable.

This document describes the repository as it is configured, not as it might
one day be run. Every rule below that a repository setting can enforce is
enforced by one, and the two are changed in the same pull request.

## Roles

### Benevolent Dictator For Life (BDFL)

- **Pierrick Fonquerne**, Founder.

The BDFL has the final say on:

- The product vision and the scope boundaries declared in the
  [roadmap](explanation/roadmap.md).
- Public API design decisions and semver classification of changes.
- Acceptance, rejection or deferral of contributions.
- Release cadence and the contents of each release.

Decisions are made with input from maintainers and the community, but the
BDFL retains tie-breaking authority. The intent is to keep the project
coherent while it finds its shape.

### Maintainers

Maintainers are contributors trusted with merge rights on `main`. Their
responsibilities:

- Review and merge pull requests.
- Triage issues and shepherd discussions.
- Uphold the trunk-based development discipline described below.

The project currently has **one maintainer**, the BDFL. Branch protection
therefore requires **zero approving reviews**: there is nobody else to give
one, and a requirement that could only ever be satisfied by bypassing it
would be worse than no requirement at all.

New maintainers are nominated by an existing maintainer and confirmed by the
BDFL. The pull request that adds a second maintainer is also the one that
raises the required approving review count to one and rewrites this
paragraph.

### Contributors

Anyone who opens an issue, comments on a discussion, or submits a pull
request. Contribution guidelines live in
[`CONTRIBUTING.md`](../CONTRIBUTING.md) and behaviour expectations live in
[`CODE_OF_CONDUCT.md`](../CODE_OF_CONDUCT.md).

## Development discipline

### Trunk-based development

- `main` is the single long-lived branch. It is always releasable.
- Work happens on short-lived branches named `feature/...`, `fix/...`,
  `chore/...`, `docs/...` or `refactor/...`, branched from `main`.
- Branches are expected to live for hours or days, not weeks.

### Pull requests

- Every change to `main` lands through a pull request.
- No approving review is required, for the reason given under
  [Maintainers](#maintainers). The automated review, reported as the
  `team / ai-review` status check, is required and runs on every pull request.
- These status checks must pass before a pull request can merge, under exactly
  these names: `test (ubuntu-latest)`, `test (macos-latest)`,
  `test (windows-latest)`, `msrv`, `deny`, `team / ai-review` and
  `Documentation`.
- Formatting and linting have no check of their own. `cargo fmt --all --check`
  and `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  run inside the `test` job, so a formatting slip turns `test` red rather than
  a check named after it.
- A branch must be up to date with `main` before it can merge, so the checks
  that passed ran against the code that actually lands.
- Commits follow [Conventional Commits](https://www.conventionalcommits.org/).
- Pull requests land as **merge commits**. Squash and rebase merges are
  disabled on the repository, and the merged branch is deleted automatically.

### Why merge commits

`main` is deliberately not a linear history. Because every pull request lands
as a merge commit, the first parent of each commit on `main` is the previous
state of `main`, so walking first parents yields exactly the states `main` has
pointed at, each one validated by CI as `main`.

The release tooling depends on that property: `cargo xtask release-verify`
refuses to publish a tag whose commit is not on the first-parent line of
`origin/main`. Under a squash or rebase workflow the first-parent line would
be the whole of `main` and the check would accept anything reachable from it.
The merge strategy is therefore a release-safety property, not a matter of
taste. See [`RELEASE_PROCESS.md`](RELEASE_PROCESS.md).

### Protected `main` and release tags

- No direct pushes to `main`, and the protection applies to administrators
  too. The rule is enforced by the repository, not left to discipline.
- No force push to `main` under any circumstance, and `main` cannot be
  deleted. If `main` needs to move backwards, it is done through a revert pull
  request.
- Tags matching `v*` cannot be created, updated or deleted except by a
  repository administrator, so a published version keeps the tag it was
  published from.

## Versioning

- isochron follows [Semantic Versioning](https://semver.org/) once it
  reaches `v1.0.0`. Pre-`v1.0.0` releases follow the conventions described
  in [`docs/SEMVER_POLICY.md`](SEMVER_POLICY.md).
- The minimum supported Rust version policy lives in
  [`docs/MSRV_POLICY.md`](MSRV_POLICY.md).
- Semver classification of a change (major, minor, patch) is proposed by the
  author in the pull request and adjudicated by the BDFL when the impact is
  non-obvious.

## Changes to this document

Governance changes are themselves pull requests, carrying a
`docs(governance):` Conventional Commit and adjudicated by the BDFL. A change
that contradicts a repository setting is not a governance change: it is either
a setting change or a documentation bug, and it is fixed as one.
