# isochron

[![crates.io](https://img.shields.io/crates/v/isochron.svg)](https://crates.io/crates/isochron)
[![docs.rs](https://img.shields.io/docsrs/isochron)](https://docs.rs/isochron)
[![CI](https://github.com/nubster-opensources/isochron/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/nubster-opensources/isochron/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue.svg)](./docs/MSRV_POLICY.md)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Status](https://img.shields.io/badge/status-alpha-yellow)](#status)
[![Made with Rust](https://img.shields.io/badge/made%20with-Rust-orange?logo=rust)](https://www.rust-lang.org/)

> A pure, deterministic cron occurrence engine on `time`: parse Vixie-standard cron expressions and compute next and previous occurrences in strict UTC.

isochron parses five- and six-field Vixie cron expressions and computes the next or previous occurrence within a bounded horizon. It is a library, not a scheduler: it tells you when something should run and leaves running it to your own runtime. No async, no `chrono`, no `unsafe`.

isochron is sponsored by [Nubster](https://nubster.com).

## Status

isochron is alpha (`0.1.x`), published on [crates.io](https://crates.io/crates/isochron). The public API is usable and covered by unit tests and doctests, but it may still change before `1.0` under the policy in [`docs/SEMVER_POLICY.md`](./docs/SEMVER_POLICY.md). The trajectory to `1.0` is documented in [`docs/explanation/roadmap.md`](./docs/explanation/roadmap.md).

## Quick start

Add to your `Cargo.toml`:

```toml
isochron = "0.1"
time = "0.3"   # OffsetDateTime appears in the public API
```

```rust
use isochron::CronSchedule;
use time::macros::datetime;

let schedule = CronSchedule::parse("0 9 * * MON-FRI").unwrap();
let after = datetime!(2026-01-01 12:00:00 UTC);
let next = schedule.next_after(after).unwrap();
println!("next: {}", next);          // next: 2026-01-02 09:00:00.0 +00:00:00
println!("{}", schedule.describe()); // "At 09:00, Monday through Friday"
```

## Why isochron

- Pure and deterministic: no global clock, no ambient state. Given an expression and an instant, the next occurrence is a total function of its inputs.
- Built on `time`, not `chrono`: occurrences are `OffsetDateTime` values in UTC, with no transitive `chrono` dependency.
- Vixie-faithful semantics: the day-of-month / day-of-week union, Sunday as both `0` and `7`, and canonical token parsing follow standard cron rather than Quartz.
- Zero unsafe, zero async: `unsafe_code` is forbidden at the crate level, and the engine is synchronous so it composes with any runtime.

## What isochron is not

isochron computes occurrences; it does not run them. It is not:

- a scheduling runtime: it spawns no threads, tasks or timers (combine it with `tokio::time` or any async runtime of your choice),
- timezone-aware: all computation is UTC (convert before calling),
- a job store: it has no concept of missed jobs, state machines or durable queues.

It also does not implement Quartz extensions (`L`, `W`, `#`, `?`), `@reboot`, a year field, or per-expression timezones.

## Supported syntax

Five fields (`minute hour day-of-month month day-of-week`) or six with a leading
seconds field. Tokens: `*`, values, ranges `a-b`, steps `*/n` and `a-b/n`, comma
lists, and case-insensitive month and weekday names. Macros: `@yearly`,
`@annually`, `@monthly`, `@weekly`, `@daily`, `@midnight`, `@hourly`. The
day-of-month / day-of-week union follows Vixie semantics.

### Semantics

**Field order.** The six-field form puts seconds first:
`second minute hour day-of-month month day-of-week`. The five-field form omits
the leading seconds field (implicitly second 0).

**Ranges.** Ranges must be non-wrapping: the start must be less than or equal to
the end. An inverted range such as `22-2` for hours is rejected as a parse error.
Use a comma list instead: `22-23,0-2`.

**Sunday in day-of-week.** Both `0` and `7` denote Sunday. `7` is accepted in
ranges: `5-7` means Friday, Saturday, Sunday.

**Day-of-month / day-of-week union (Vixie semantics).** When BOTH fields are
restricted (not a bare `*`), a day matches if EITHER the day-of-month OR the
day-of-week matches. Only the literal `*` disables a field's restriction: a range
like `1-31` still counts as restricted. This differs from Quartz which requires a
`?` placeholder and uses AND logic.

## Documentation

- API reference on [docs.rs](https://docs.rs/isochron).
- Stability and versioning: [`docs/SEMVER_POLICY.md`](./docs/SEMVER_POLICY.md) and [`docs/MSRV_POLICY.md`](./docs/MSRV_POLICY.md).
- Release process: [`docs/RELEASE_PROCESS.md`](./docs/RELEASE_PROCESS.md).
- Roadmap and design boundaries: [`docs/explanation/roadmap.md`](./docs/explanation/roadmap.md).

## Contributing

Contributions are welcome. Please read [`CONTRIBUTING.md`](./CONTRIBUTING.md)
first for the workflow and conventions, and [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md)
for the community guidelines. For vulnerability reports, see
[`SECURITY.md`](./SECURITY.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual-licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md) for details, including the Contributor
License Agreement (CLA).

Copyright (c) Nubster.
