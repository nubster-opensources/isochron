# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.1.1]: https://github.com/nubster-opensources/isochron/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.0
