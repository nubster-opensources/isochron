# isochron

[![CI](https://github.com/nubster-opensources/isochron/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/nubster-opensources/isochron/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue.svg)](./docs/MSRV_POLICY.md)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Status](https://img.shields.io/badge/status-alpha-yellow)](#status)
[![Made with Rust](https://img.shields.io/badge/made%20with-Rust-orange?logo=rust)](https://www.rust-lang.org/)

A cron occurrence engine on `time`: parse Vixie-standard cron expressions and
compute next and previous occurrences in strict UTC. Pure and deterministic, no
async, no `chrono`, no `unsafe`.

isochron is sponsored by [Nubster](https://nubster.com).

## Example

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

## Supported syntax

Five fields (`minute hour day-of-month month day-of-week`) or six with a leading
seconds field. Tokens: `*`, values, ranges `a-b`, steps `*/n` and `a-b/n`, comma
lists, and case-insensitive month and weekday names. Macros: `@yearly`,
`@annually`, `@monthly`, `@weekly`, `@daily`, `@midnight`, `@hourly`. The
day-of-month / day-of-week union follows Vixie semantics.

Not supported: Quartz extensions (`L`, `W`, `#`, `?`), `@reboot`, a year field,
and per-expression timezones (evaluation is UTC).

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

## Contributing

Contributions are welcome. Please read [`CONTRIBUTING.md`](./CONTRIBUTING.md)
first for the workflow and conventions, and [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md)
for the community guidelines. For vulnerability reports, see
[`SECURITY.md`](./SECURITY.md).

Stability and versioning are documented in
[`docs/SEMVER_POLICY.md`](./docs/SEMVER_POLICY.md) and
[`docs/MSRV_POLICY.md`](./docs/MSRV_POLICY.md).

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
