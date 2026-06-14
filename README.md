# isochron

A cron occurrence engine on `time`: parse Vixie-standard cron expressions and
compute next and previous occurrences in strict UTC. Pure and deterministic, no
async, no `chrono`, no `unsafe`.

## Example

```rust
use isochron::CronSchedule;
use time::macros::datetime;

let schedule = CronSchedule::parse("0 9 * * MON-FRI").unwrap();
let after = datetime!(2026-01-01 12:00:00 UTC);
let next = schedule.next_after(after).unwrap();
println!("{}", schedule.describe());
```

## Supported syntax

Five fields (`minute hour day-of-month month day-of-week`) or six with a leading
seconds field. Tokens: `*`, values, ranges `a-b`, steps `*/n` and `a-b/n`, comma
lists, and case-insensitive month and weekday names. Macros: `@yearly`,
`@annually`, `@monthly`, `@weekly`, `@daily`, `@midnight`, `@hourly`. The
day-of-month / day-of-week union follows Vixie semantics.

Not supported: Quartz extensions (`L`, `W`, `#`, `?`), `@reboot`, a year field,
and per-expression timezones (evaluation is UTC).

## License

Licensed under either of MIT or Apache-2.0 at your option.
