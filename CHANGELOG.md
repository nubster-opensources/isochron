# Changelog

## 0.1.1 (2026-06-18)

- Stricter field parsing: a leading `+` sign (`+5`) or zero-padded numbers (`007`, `00`) are now rejected as non-canonical, in both values and `/N` step tokens, matching standard Vixie cron. The single digit `0` stays valid.
- The `Upcoming` iterator type is now `#[must_use]`, so binding it without consuming the iterator is linted.

## 0.1.0 (2026-06-15)

- Parse Vixie-standard cron expressions (five or six fields, macros).
- Compute next and previous occurrences in strict UTC with a bounded horizon.
- Planning helpers: `upcoming` iterator and `time_until_next`.
- English `describe` of a schedule.
- Value equality and hashing ignore source spelling: equivalent schedules (for example `0` and `7`, or `MON` and `1`) now compare equal.
- `describe` collapses full fields instead of enumerating them and renders six-field expressions with clearer phrasing.
