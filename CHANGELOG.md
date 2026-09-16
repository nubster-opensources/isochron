# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- A stability rule for the wording of `describe()`: unchanged within a minor release line, documented in the rustdoc and in `docs/SEMVER_POLICY.md`. (#43)

### Changed

- Raise the MSRV from Rust 1.88 to 1.89 for the August 2026 fleet baseline and prefer MSRV-compatible dependency versions during Cargo updates.
- Release tooling moved from a shell script and cargo-release to a dependency-free `cargo xtask` (`release-prep`, `release-notes`).

### Fixed

- `CronSchedule::is_match` now returns `false` for instants with a non-zero nanosecond, consistent with `next_after` and `prev_before` which only yield whole-second occurrences. (#40)
- Day-of-week out-of-range errors now report the accepted upper bound `7` (the Sunday alias) instead of `6`. (#44)
- A day-of-week step starting from the Sunday alias, such as `7/2`, is now rejected with an explicit reason instead of a misleading range error. (#63)
- The README quick start now shows the actual `next_after` and `describe` results and runs as a doctest, so it can no longer drift. (#43)
- Release preparation no longer aborts on the current changelog format: graduation turns `[Unreleased]` into a dated version heading, reopens an empty `[Unreleased]` section, updates the compare links, and fails with an explicit message before creating a branch when the section is missing, malformed or empty. The preparation branch is now `chore/release-vX.Y.Z`. (#29)
- Release notes are now extracted from the matching changelog section by exact heading comparison, and a missing section fails the release instead of publishing placeholder notes. (#42)

### Security

- A release tag is now verified before anything can reach crates.io: the tag must read `vX.Y.Z`, match the packaged version, carry a non-empty changelog section, point at the commit being built, and sit on the first-parent line of `origin/main`. Publication moved to a protected environment that requires a maintainer approval and mints a short lived crates.io token through trusted publishing, so no long lived registry token is used. The manual dispatch of the release workflow can no longer publish. (#39)

## [0.1.1] - 2026-06-18

### Changed

- Stricter field parsing: a leading `+` sign (`+5`) or zero-padded numbers (`007`, `00`) are now rejected as non-canonical, in both values and `/N` step tokens, matching standard Vixie cron. The single digit `0` stays valid.
- The `Upcoming` iterator type is now `#[must_use]`, so binding it without consuming the iterator is linted.

## [0.1.0] - 2026-06-15

### Added

- Parse Vixie-standard cron expressions (five or six fields, macros).
- Compute next and previous occurrences in strict UTC with a bounded horizon.
- Planning helpers: `upcoming` iterator and `time_until_next`.
- English `describe` of a schedule.
- Value equality and hashing ignore source spelling: equivalent schedules (for example `0` and `7`, or `MON` and `1`) now compare equal.
- `describe` collapses full fields instead of enumerating them and renders six-field expressions with clearer phrasing.

[Unreleased]: https://github.com/nubster-opensources/isochron/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/nubster-opensources/isochron/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.0
