# Changelog

## 0.1.0 (2026-06-15)

- Parse Vixie-standard cron expressions (five or six fields, macros).
- Compute next and previous occurrences in strict UTC with a bounded horizon.
- Planning helpers: `upcoming` iterator and `time_until_next`.
- English `describe` of a schedule.
- Value equality and hashing ignore source spelling: equivalent schedules (for example `0` and `7`, or `MON` and `1`) now compare equal.
- `describe` collapses full fields instead of enumerating them and renders six-field expressions with clearer phrasing.
