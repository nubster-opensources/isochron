# Roadmap

isochron is pre-stable. This document captures the intended trajectory of the
project up to v1.0, ordered by release. **No dates are committed.** The
project is sponsored on a best-effort basis by Nubster, and releases ship when
they are ready, not when a calendar says so.

The roadmap mirrors the
[milestones](https://github.com/nubster-opensources/isochron/milestones)
one-for-one. Each section here is the public, prose form of a milestone.

## Out of scope

isochron is a pure, deterministic cron occurrence engine. The following will
never be in scope, regardless of demand:

- **Scheduling runtime.** isochron computes occurrences; it does not spawn
  threads, tasks or timers. Combine it with `tokio::time` or any async runtime
  of your choice.
- **Timezone support.** All computation is UTC. Per-expression timezones are
  not planned; convert before calling.
- **Job storage or persistence.** isochron has no concept of missed jobs,
  state machines, or durable queues.

These boundaries are deliberate. If a feature request crosses one of them, it
belongs in another project.

## v0.1.0: Foundation (DONE)

**Goal.** A pure, no-unsafe cron occurrence engine with a fully validated
public API.

**Shipped:**

- `CronSchedule::parse` accepting five-field and six-field (with leading
  seconds) Vixie expressions plus the standard macros
  (`@yearly`, `@annually`, `@monthly`, `@weekly`, `@daily`, `@midnight`,
  `@hourly`).
- `CronSchedule::next_after` and `CronSchedule::prev_before` computing the
  nearest occurrence within a five-year horizon.
- `CronSchedule::upcoming`: a lazy `Iterator` of successive occurrences.
- `CronSchedule::time_until_next`: the duration until the next occurrence.
- `CronSchedule::describe`: an English human-readable summary.
- `CronError` typed error enum covering all parse failure modes.
- Day-of-month / day-of-week union (Vixie semantics).
- Case-insensitive month and weekday names.
- Full rustdoc coverage and 49 unit tests plus a crate-level doctest.
- Zero unsafe code, zero async, zero chrono.

## v0.2.0: Ergonomics and Extensions

**Goal.** Refine the describe output, improve performance for dense
expressions, and add optional integration features.

**Planned scope:**

- Full seconds in `describe` output for six-field expressions.
- Month-skip optimisation for expressions with very sparse month sets.
- Optional `serde` feature: `Serialize`/`Deserialize` for `CronSchedule`
  and `CronError`.
- Quartz extensions (`L`, `W`, `#`, `?`) behind an opt-in Cargo feature.

## v0.3.0 and beyond

Scope is not committed yet. Candidates include:

- A year field behind an opt-in feature.
- `CronSchedule::prev_n` and `CronSchedule::next_n` convenience methods.
- `no_std` compatibility (contingent on the `time` crate).

## How this roadmap is maintained

Changes to this document are made by pull request, with a
`docs(roadmap):` Conventional Commit. The scope of any released version is
locked once its tag is pushed; the scope of later releases stays adjustable
until the previous release ships.

If you spot something missing, redundant or out of scope, open an issue against
the relevant milestone and tag it `discussion`.
