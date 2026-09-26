//! A parsed cron schedule and its public parsing entry points.

use core::fmt;
use core::hash::{Hash, Hasher};
use core::str::FromStr;

use time::{Duration, OffsetDateTime, UtcOffset};

use crate::iter::Upcoming;

use crate::day_filter::DayFilter;
use crate::error::CronError;
use crate::field::{self, FieldSchedule};

/// A parsed cron schedule, evaluated in strict UTC.
///
/// Build one with [`CronSchedule::parse`]. Compute occurrences with
/// [`CronSchedule::next_after`] and [`CronSchedule::prev_before`].
///
/// # Equality
///
/// Two schedules are equal when they impose the same instants: the same sets
/// of seconds, minutes, hours and months, and the same effective day filter.
/// How the expression was written never enters the comparison, so a five-field
/// expression equals its six-field form with seconds pinned to zero, and a day
/// restriction that accepts every day equals a bare `*`. `Hash` agrees with
/// `Eq` on exactly those components.
///
/// The comparison is structural on that filter, not extensional on the
/// occurrences: it does not decide whether two expressions fire at the same
/// instants in general. Two schedules that never fire, such as `0 0 30 2 *`
/// and `0 0 31 2 *`, remain distinct. Equality also says nothing about
/// [`Display`](std::fmt::Display), which renders the expression as written.
#[derive(Debug, Clone)]
pub struct CronSchedule {
    pub(crate) second: FieldSchedule,
    pub(crate) minute: FieldSchedule,
    pub(crate) hour: FieldSchedule,
    pub(crate) month: FieldSchedule,
    pub(crate) days: DayFilter,
    pub(crate) has_seconds: bool,
    normalized: String,
}

impl PartialEq for CronSchedule {
    fn eq(&self, other: &Self) -> bool {
        self.second == other.second
            && self.minute == other.minute
            && self.hour == other.hour
            && self.month == other.month
            && self.days == other.days
    }
}

impl Eq for CronSchedule {}

impl Hash for CronSchedule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.second.hash(state);
        self.minute.hash(state);
        self.hour.hash(state);
        self.month.hash(state);
        self.days.hash(state);
    }
}

impl CronSchedule {
    /// Parse and fully validate a cron expression.
    ///
    /// Accepts five fields (`minute hour day-of-month month day-of-week`) or six
    /// when a leading seconds field is present (`second minute hour day-of-month
    /// month day-of-week`), plus the macros `@yearly`, `@annually`, `@monthly`,
    /// `@weekly`, `@daily`, `@midnight`, and `@hourly`. The five-field form
    /// implicitly sets seconds to 0.
    ///
    /// # Semantics
    ///
    /// **Ranges.** Ranges must be non-wrapping (start <= end). An inverted range
    /// such as `22-2` for hours is a parse error; use a comma list `22-23,0-2`
    /// instead.
    ///
    /// **Steps.** `*/n` steps over the whole field, `a-b/n` over a range, and
    /// `a/n` from `a` up to the field maximum (so `5/10` in hours matches 5 and
    /// 15). In day-of-week a step cannot start from the Sunday alias `7`: `7/n`
    /// is a parse error; start from `0` or `SUN` instead.
    ///
    /// **Sunday in day-of-week.** Both `0` and `7` denote Sunday. `7` is valid
    /// in ranges: `5-7` matches Friday, Saturday, and Sunday.
    ///
    /// **Day union (Vixie semantics).** When BOTH the day-of-month and
    /// day-of-week fields are restricted (i.e. not a bare `*`), a day matches if
    /// EITHER field matches (OR logic). Only the literal `*` disables a field's
    /// restriction; a range such as `1-31` still counts as restricted. This
    /// differs from Quartz, which uses AND logic with an explicit `?` placeholder.
    ///
    /// A restriction that accepts every value imposes nothing, and under the
    /// union a member accepting every day absorbs the other. So `0 0 1-31 * *`
    /// and `0 0 13 * 0-6` match every day, exactly like `0 0 * * *`, and they
    /// compare equal to it. See the type's `Equality` section.
    ///
    /// # Errors
    ///
    /// Returns a [`CronError`] if the expression is empty, carries the wrong
    /// number of fields, or any field is malformed.
    pub fn parse(expression: &str) -> Result<Self, CronError> {
        let trimmed = expression.trim();
        if trimmed.is_empty() {
            return Err(CronError::EmptyExpression);
        }
        let expanded = expand_macro(trimmed);
        let fields: Vec<&str> = expanded.split_whitespace().collect();
        let normalized = fields.join(" ");

        let (has_seconds, offset) = match fields.len() {
            5 => (false, 0),
            6 => (true, 1),
            other => return Err(CronError::WrongFieldCount { found: other }),
        };

        let second = if has_seconds {
            FieldSchedule::parse(fields[0], field::SECOND)?
        } else {
            FieldSchedule::parse("0", field::SECOND)?
        };
        let minute = FieldSchedule::parse(fields[offset], field::MINUTE)?;
        let hour = FieldSchedule::parse(fields[offset + 1], field::HOUR)?;
        let dom_token = fields[offset + 2];
        let day_of_month = FieldSchedule::parse(dom_token, field::DAY_OF_MONTH)?;
        let month = FieldSchedule::parse(fields[offset + 3], field::MONTH)?;
        let weekday_token = fields[offset + 4];
        let day_of_week = FieldSchedule::parse(weekday_token, field::DAY_OF_WEEK)?;

        Ok(Self {
            second,
            minute,
            hour,
            month,
            days: DayFilter::new(
                (dom_token != "*").then_some(day_of_month),
                (weekday_token != "*").then_some(day_of_week),
            ),
            has_seconds,
            normalized,
        })
    }

    /// A lazy iterator of occurrences strictly after `from`.
    ///
    /// # Examples
    ///
    /// ```
    /// use isochron::CronSchedule;
    /// use time::macros::datetime;
    ///
    /// let schedule = CronSchedule::parse("0 9 * * *").expect("valid");
    /// let mut iter = schedule.upcoming(datetime!(2026-01-01 00:00:00 UTC));
    /// assert_eq!(iter.next(), Some(datetime!(2026-01-01 09:00:00 UTC)));
    /// assert_eq!(iter.next(), Some(datetime!(2026-01-02 09:00:00 UTC)));
    /// ```
    pub fn upcoming(&self, from: OffsetDateTime) -> Upcoming<'_> {
        Upcoming::new(self, from)
    }

    /// The duration from `from` until the next occurrence, or `None` if no
    /// occurrence exists within the search horizon.
    ///
    /// # Examples
    ///
    /// ```
    /// use isochron::CronSchedule;
    /// use time::{Duration, macros::datetime};
    ///
    /// let schedule = CronSchedule::parse("0 * * * *").expect("valid");
    /// // From midnight the next hourly tick is at 01:00, one hour away.
    /// let wait = schedule
    ///     .time_until_next(datetime!(2026-01-01 00:00:00 UTC))
    ///     .expect("exists");
    /// assert_eq!(wait, Duration::hours(1));
    /// ```
    #[must_use]
    pub fn time_until_next(&self, from: OffsetDateTime) -> Option<Duration> {
        let next = self.next_after(from)?;
        Some(next - from.to_offset(UtcOffset::UTC))
    }

    /// An English human-readable description of the schedule.
    ///
    /// List fields are enumerated value by value: a range such as `MON-FRI`
    /// is spelled out as `Monday, Tuesday, Wednesday, Thursday and Friday`,
    /// not compacted back into a range.
    ///
    /// # Stability
    ///
    /// The wording is stable within a minor release line: a patch release
    /// never changes it except to fix a clear mistake, while a minor release
    /// may change it, including after `1.0`, and the change is then
    /// announced under `Changed` in the changelog. See `docs/SEMVER_POLICY.md`
    /// for the full policy. The text is meant for display; do not parse it.
    ///
    /// # Examples
    ///
    /// ```
    /// use isochron::CronSchedule;
    ///
    /// let schedule = CronSchedule::parse("0 9 * * MON-FRI").expect("valid");
    /// assert_eq!(
    ///     schedule.describe(),
    ///     "at 09:00 on Monday, Tuesday, Wednesday, Thursday and Friday"
    /// );
    /// ```
    #[must_use]
    pub fn describe(&self) -> String {
        crate::describe::describe(self)
    }

    /// Returns true if `datetime` is an occurrence of this schedule, evaluated in UTC.
    ///
    /// The offset of `datetime` is normalised to UTC before matching, so any
    /// `OffsetDateTime` is accepted regardless of its original offset.
    ///
    /// Cron has second resolution, so an instant carrying a non-zero
    /// nanosecond is never an occurrence: `is_match(t)` is true exactly when
    /// `next_after(t - 1ns) == Some(t)`. To test an arbitrary instant, such
    /// as the current time, truncate it to the second first with
    /// `datetime.replace_nanosecond(0)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use isochron::CronSchedule;
    /// use time::macros::datetime;
    ///
    /// let schedule = CronSchedule::parse("0 0 * * *").expect("valid");
    /// assert!(schedule.is_match(datetime!(2026-06-15 00:00:00 UTC)));
    /// assert!(!schedule.is_match(datetime!(2026-06-15 12:00:00 UTC)));
    /// assert!(!schedule.is_match(datetime!(2026-06-15 00:00:00.5 UTC)));
    /// ```
    #[must_use]
    pub fn is_match(&self, datetime: OffsetDateTime) -> bool {
        if datetime.nanosecond() != 0 {
            return false;
        }
        let datetime = datetime.to_offset(UtcOffset::UTC);
        self.second.contains(datetime.second())
            && self.minute.contains(datetime.minute())
            && self.hour.contains(datetime.hour())
            && self.month.contains(u8::from(datetime.month()))
            && self.days.matches(datetime)
    }
}

fn expand_macro(expression: &str) -> String {
    match expression.to_ascii_lowercase().as_str() {
        "@yearly" | "@annually" => "0 0 1 1 *".to_owned(),
        "@monthly" => "0 0 1 * *".to_owned(),
        "@weekly" => "0 0 * * 0".to_owned(),
        "@daily" | "@midnight" => "0 0 * * *".to_owned(),
        "@hourly" => "0 * * * *".to_owned(),
        _ => expression.to_owned(),
    }
}

impl FromStr for CronSchedule {
    type Err = CronError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl fmt::Display for CronSchedule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::CronSchedule;
    use crate::day_filter::DayFilter;
    use crate::error::CronError;
    use time::macros::datetime;
    use time::{Duration, OffsetDateTime, UtcOffset};

    #[test]
    fn five_fields_parse() {
        let schedule = CronSchedule::parse("0 0 * * *").expect("valid");
        assert!(!schedule.has_seconds);
    }

    #[test]
    fn six_fields_parse_with_seconds() {
        let schedule = CronSchedule::parse("30 0 0 * * *").expect("valid");
        assert!(schedule.has_seconds);
        assert!(schedule.second.contains(30));
    }

    #[test]
    fn wrong_field_count_is_rejected() {
        let error = CronSchedule::parse("* * *").unwrap_err();
        assert!(matches!(error, CronError::WrongFieldCount { found: 3 }));
    }

    #[test]
    fn empty_is_rejected() {
        assert_eq!(
            CronSchedule::parse("   ").unwrap_err(),
            CronError::EmptyExpression
        );
    }

    #[test]
    fn daily_macro_expands() {
        let from_macro = CronSchedule::parse("@daily").expect("valid");
        let explicit = CronSchedule::parse("0 0 * * *").expect("valid");
        assert_eq!(from_macro, explicit);
    }

    #[test]
    fn parse_builds_the_effective_day_filter() {
        let union = CronSchedule::parse("0 0 1 * 1").expect("valid");
        assert!(matches!(union.days, DayFilter::Union { .. }));
        let loose = CronSchedule::parse("0 0 * * *").expect("valid");
        assert_eq!(loose.days, DayFilter::EveryDay);
        let day_only = CronSchedule::parse("0 0 1 * *").expect("valid");
        assert!(matches!(day_only.days, DayFilter::DayOfMonth(_)));
        let weekday_only = CronSchedule::parse("0 0 * * 1").expect("valid");
        assert!(matches!(weekday_only.days, DayFilter::DayOfWeek(_)));
    }

    #[test]
    fn matches_respects_dom_or_dow_union() {
        // Day-of-month 1 OR Monday: 2026-06-01 is a Monday, 2026-06-15 is a
        // Monday, 2026-07-01 is a Wednesday (matches by day-of-month).
        let schedule = CronSchedule::parse("0 0 1 * 1").expect("valid");
        assert!(schedule.is_match(datetime!(2026-06-15 00:00:00 UTC))); // Monday
        assert!(schedule.is_match(datetime!(2026-07-01 00:00:00 UTC))); // day 1
        assert!(!schedule.is_match(datetime!(2026-06-16 00:00:00 UTC))); // neither
    }

    #[test]
    fn matches_with_single_day_constraint() {
        let schedule = CronSchedule::parse("0 0 15 * *").expect("valid");
        assert!(schedule.is_match(datetime!(2026-06-15 00:00:00 UTC)));
        assert!(!schedule.is_match(datetime!(2026-06-16 00:00:00 UTC)));
    }

    #[test]
    fn display_round_trips() {
        let schedule = CronSchedule::parse("  0   0 1,15 * MON  ").expect("valid");
        let rendered = schedule.to_string();
        let reparsed = CronSchedule::parse(&rendered).expect("valid");
        assert_eq!(schedule, reparsed);
    }

    fn rendered(expression: &str) -> String {
        CronSchedule::parse(expression).expect("valid").to_string()
    }

    #[test]
    fn display_omits_the_seconds_field_when_seconds_are_zero() {
        assert_eq!(rendered("0 0 * * *"), "0 0 * * *");
        assert_eq!(rendered("0 0 0 * * *"), "0 0 * * *");
    }

    #[test]
    fn display_keeps_the_seconds_field_when_seconds_are_not_zero() {
        assert_eq!(rendered("30 0 0 * * *"), "30 0 0 * * *");
        assert_eq!(rendered("* 0 0 * * *"), "* 0 0 * * *");
        assert_eq!(rendered("0,30 0 0 * * *"), "0,30 0 0 * * *");
    }

    // The five-versus-six-field rule reads the seconds SET, never `has_seconds`,
    // which equality excludes. Reading the flag would break the invariant on this
    // very pair, which differs by nothing else.
    #[test]
    fn display_does_not_read_the_seconds_flag() {
        let five = CronSchedule::parse("0 0 * * *").expect("valid");
        let six = CronSchedule::parse("0 0 0 * * *").expect("valid");
        assert_ne!(five.has_seconds, six.has_seconds);
        assert_eq!(five, six);
        assert_eq!(five.to_string(), six.to_string());
    }

    #[test]
    fn display_renders_names_and_aliases_as_numbers() {
        assert_eq!(rendered("0 9 * * MON-FRI"), "0 9 * * 1-5");
        assert_eq!(rendered("0 0 1 JAN *"), "0 0 1 1 *");
        assert_eq!(rendered("0 0 * * 7"), "0 0 * * 0");
        assert_eq!(rendered("0 0 * dec *"), "0 0 * 12 *");
    }

    #[test]
    fn display_expands_a_macro() {
        assert_eq!(rendered("@yearly"), "0 0 1 1 *");
        assert_eq!(rendered("@annually"), "0 0 1 1 *");
        assert_eq!(rendered("@monthly"), "0 0 1 * *");
        assert_eq!(rendered("@weekly"), "0 0 * * 0");
        assert_eq!(rendered("@daily"), "0 0 * * *");
        assert_eq!(rendered("@midnight"), "0 0 * * *");
        assert_eq!(rendered("@hourly"), "0 * * * *");
    }

    #[test]
    fn display_normalises_whitespace_order_and_duplicates() {
        assert_eq!(rendered("  0   0 15,1,15 * MON  "), "0 0 1,15 * 1");
    }

    #[test]
    fn display_expands_a_step() {
        assert_eq!(rendered("*/15 * * * *"), "0,15,30,45 * * * *");
        assert_eq!(rendered("0 0-6/2 * * *"), "0 0,2,4,6 * * *");
    }

    // #41 folded a day restriction accepting every day into `EveryDay`, so the
    // canonical form loses a spelling that equality had already declared
    // insignificant. Contesting this means contesting #41, not the rendering.
    #[test]
    fn display_drops_a_day_restriction_that_restricts_nothing() {
        assert_eq!(rendered("0 0 1-31 * *"), "0 0 * * *");
        assert_eq!(rendered("0 0 13 * 0-6"), "0 0 * * *");
        assert_eq!(rendered("0 0 1-31 * MON"), "0 0 * * *");
    }

    #[test]
    fn display_keeps_a_day_of_month_that_alone_restricts() {
        assert_eq!(rendered("0 0 13 * *"), "0 0 13 * *");
    }

    #[test]
    fn display_keeps_a_union_of_two_partial_fields() {
        assert_eq!(rendered("0 0 1 * MON"), "0 0 1 * 1");
    }

    // Couples already pinned as equal by #41, now required to render identically.
    const EQUAL_COUPLES: [(&str, &str); 7] = [
        ("0 0 * * *", "0 0 0 * * *"),
        ("0 0 * * *", "0 0 1-31 * *"),
        ("0 0 * * *", "0 0 13 * 0-6"),
        ("0 0 * * 0", "0 0 * * 7"),
        ("0 9 * * MON-FRI", "0 9 * * 1-5"),
        ("0 0 1,15 * *", "0 0 15,1,1 * *"),
        ("@daily", "0 0 * * *"),
    ];

    #[test]
    fn equal_schedules_display_identically() {
        for (left, right) in EQUAL_COUPLES {
            let a = CronSchedule::parse(left).expect("valid");
            let b = CronSchedule::parse(right).expect("valid");
            assert_eq!(a, b, "`{left}` and `{right}` should be equal");
            assert_eq!(
                a.to_string(),
                b.to_string(),
                "`{left}` and `{right}` are equal but render differently"
            );
        }
    }

    // Two schedules that compare equal must fire at the same instants. This does
    // not test `Display`: it tests the equality boundary #41 drew, since
    // `has_seconds` drives the iteration step of `occurrence.rs` while being
    // excluded from the comparison.
    #[test]
    fn equal_schedules_produce_the_same_occurrences() {
        let from = datetime!(2026-01-01 00:00:00 UTC);
        for (left, right) in EQUAL_COUPLES {
            let a = CronSchedule::parse(left).expect("valid");
            let b = CronSchedule::parse(right).expect("valid");
            let first: Vec<OffsetDateTime> = a.upcoming(from).take(12).collect();
            let second: Vec<OffsetDateTime> = b.upcoming(from).take(12).collect();
            assert_eq!(first, second, "`{left}` and `{right}` are equal");
        }
    }

    // An exhaustive generator over a reduced domain: every non-empty subset of
    // five hour values, crossed with the four day-filter shapes. Each case is
    // emitted as a GROUP of redundant spellings of the same schedule, because a
    // generator of already-canonical expressions would only ever compare a
    // schedule with itself and assert nothing.
    fn equivalent_spelling_groups() -> Vec<Vec<String>> {
        let hours = [0u8, 1, 5, 22, 23];
        let day_shapes = [("*", "*"), ("13", "*"), ("*", "1"), ("1", "5")];
        let mut groups = Vec::new();
        for mask in 1u32..(1u32 << hours.len()) {
            let selected: Vec<u8> = hours
                .iter()
                .copied()
                .enumerate()
                .filter(|(index, _)| mask & (1u32 << index) != 0)
                .map(|(_, hour)| hour)
                .collect();
            let list = |values: &[u8]| {
                values
                    .iter()
                    .map(u8::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            };
            let ascending = list(&selected);
            let descending = {
                let mut reversed = selected.clone();
                reversed.reverse();
                list(&reversed)
            };
            let duplicated = format!("{ascending},{}", selected[0]);
            for (day_of_month, day_of_week) in day_shapes {
                let mut group = vec![
                    format!("0 {ascending} {day_of_month} * {day_of_week}"),
                    format!("0 {descending} {day_of_month} * {day_of_week}"),
                    format!("0 {duplicated} {day_of_month} * {day_of_week}"),
                    format!("0 0 {ascending} {day_of_month} * {day_of_week}"),
                    format!("  0   {ascending}  {day_of_month} *  {day_of_week}  "),
                ];
                if day_of_week == "5" {
                    group.push(format!("0 {ascending} {day_of_month} * FRI"));
                }
                // A field written as its full range only restates the schedule
                // when BOTH day fields are bare. Under the Vixie union, writing
                // `1-31` while the weekday is restricted makes the full member
                // absorb the other, which CHANGES the schedule rather than
                // respelling it. That asymmetry is exactly #41's trap, and a
                // generator that ignored it would assert a false equality.
                if day_of_month == "*" && day_of_week == "*" {
                    group.push(format!("0 {ascending} 1-31 * *"));
                    group.push(format!("0 {ascending} * * 0-6"));
                    group.push(format!("0 {ascending} 1-31 * 0-6"));
                }
                groups.push(group);
            }
        }
        groups
    }

    #[test]
    fn every_generated_group_is_equal_and_renders_identically() {
        let mut compared = 0usize;
        for group in equivalent_spelling_groups() {
            let reference_expression = &group[0];
            let reference = CronSchedule::parse(reference_expression).expect("valid");
            for expression in group.iter().skip(1) {
                let schedule = CronSchedule::parse(expression).expect("valid");
                assert_eq!(
                    schedule, reference,
                    "`{expression}` should equal `{reference_expression}`"
                );
                assert_eq!(
                    schedule.to_string(),
                    reference.to_string(),
                    "`{expression}` and `{reference_expression}` are equal but render differently"
                );
                compared += 1;
            }
        }
        // Guards against a generator that silently stops producing distinct
        // spellings, which would leave the assertions above vacuously true.
        assert!(
            compared > 500,
            "only {compared} distinct spellings compared"
        );
    }

    #[test]
    fn every_generated_expression_round_trips_and_is_idempotent() {
        for expression in equivalent_spelling_groups().into_iter().flatten() {
            let schedule = CronSchedule::parse(&expression).expect("valid");
            let once = schedule.to_string();
            let reparsed = CronSchedule::parse(&once).expect("the rendered form parses");
            assert_eq!(reparsed, schedule, "`{expression}` rendered `{once}`");
            assert_eq!(
                reparsed.to_string(),
                once,
                "`{expression}` renders `{once}`, which is not a fixed point"
            );
        }
    }

    #[test]
    fn eq_ignores_sunday_alias() {
        let a = CronSchedule::parse("0 0 * * 0").expect("valid");
        let b = CronSchedule::parse("0 0 * * 7").expect("valid");
        assert_eq!(a, b);
    }

    #[test]
    fn eq_ignores_name_vs_number() {
        let a = CronSchedule::parse("0 0 * * MON").expect("valid");
        let b = CronSchedule::parse("0 0 * * 1").expect("valid");
        assert_eq!(a, b);
    }

    #[test]
    fn eq_macro_matches_expanded() {
        let a = CronSchedule::parse("@yearly").expect("valid");
        let b = CronSchedule::parse("0 0 1 1 *").expect("valid");
        assert_eq!(a, b);
    }

    #[test]
    fn hash_agrees_with_eq() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(CronSchedule::parse("0 0 * * 0").expect("valid"));
        set.insert(CronSchedule::parse("0 0 * * 7").expect("valid"));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn eq_distinguishes_real_differences() {
        let a = CronSchedule::parse("0 0 * * 1").expect("valid");
        let b = CronSchedule::parse("0 0 * * 2").expect("valid");
        assert_ne!(a, b);
    }

    // Issue #63: a step cannot start from the alias 7 (Sunday). The rejection
    // must surface all the way up through CronSchedule::parse.
    #[test]
    fn parse_rejects_step_from_sunday_alias() {
        let error = CronSchedule::parse("0 0 * * 7/2").unwrap_err();
        assert_eq!(
            error.to_string(),
            "invalid day-of-week field, token \"7/2\": a step cannot start from the alias 7; start from 0 instead"
        );
    }

    // Issue #40: cron has second resolution, so an instant carrying a nonzero
    // nanosecond is never an occurrence, regardless of what its second,
    // minute, hour, day, and month fields are.
    #[test]
    fn is_match_rejects_subsecond_instant_five_fields() {
        let schedule = CronSchedule::parse("0 0 * * *").expect("valid");
        assert!(schedule.is_match(datetime!(2026-01-01 00:00:00 UTC)));
        assert!(!schedule.is_match(datetime!(2026-01-01 00:00:00.5 UTC)));
        assert!(!schedule.is_match(datetime!(2026-01-01 00:00:00.000_000_001 UTC)));
    }

    #[test]
    fn is_match_rejects_subsecond_instant_six_fields() {
        let schedule = CronSchedule::parse("30 0 0 * * *").expect("valid");
        assert!(schedule.is_match(datetime!(2026-01-01 00:00:30 UTC)));
        assert!(!schedule.is_match(datetime!(2026-01-01 00:00:30.5 UTC)));
        assert!(!schedule.is_match(datetime!(2026-01-01 00:00:30.000_000_001 UTC)));
    }

    #[test]
    fn is_match_normalises_offset_and_still_rejects_nanoseconds() {
        let schedule = CronSchedule::parse("0 0 * * *").expect("valid");
        assert!(schedule.is_match(datetime!(2026-01-01 02:00:00 +02:00)));
        assert!(!schedule.is_match(datetime!(2026-01-01 02:00:00.000_000_001 +02:00)));
    }

    #[test]
    fn is_match_agrees_with_next_after_on_sampled_instants() {
        let cases: Vec<(&str, OffsetDateTime)> = vec![
            ("0 0 * * *", datetime!(2026-01-01 00:00:00 UTC)),
            ("0 0 * * *", datetime!(2026-01-01 00:00:00.000_000_001 UTC)),
            ("0 0 * * *", datetime!(2026-01-01 00:00:00.5 UTC)),
            ("0 0 * * *", datetime!(2025-12-31 23:59:59.999_999_999 UTC)),
            ("0 0 * * *", datetime!(2026-01-01 12:00:00 UTC)),
            ("30 0 0 * * *", datetime!(2026-01-01 00:00:30 UTC)),
            (
                "30 0 0 * * *",
                datetime!(2026-01-01 00:00:30.000_000_001 UTC),
            ),
            ("30 0 0 * * *", datetime!(2026-01-01 00:00:30.5 UTC)),
            (
                "30 0 0 * * *",
                datetime!(2026-01-01 00:00:29.999_999_999 UTC),
            ),
            ("30 0 0 * * *", datetime!(2026-01-01 00:00:31 UTC)),
            ("0 0 1 * MON", datetime!(2026-06-01 00:00:00 UTC)),
            (
                "0 0 1 * MON",
                datetime!(2026-06-01 00:00:00.000_000_001 UTC),
            ),
            ("0 0 1 * MON", datetime!(2026-06-15 00:00:00 UTC)),
            ("0 0 1 * MON", datetime!(2026-06-16 00:00:00 UTC)),
            ("*/15 * * * *", datetime!(2026-01-01 00:15:00 UTC)),
            (
                "*/15 * * * *",
                datetime!(2026-01-01 00:15:00.000_000_001 UTC),
            ),
            ("*/15 * * * *", datetime!(2026-01-01 00:15:00.5 UTC)),
            ("*/15 * * * *", datetime!(2026-01-01 00:20:00 UTC)),
        ];

        for (expression, instant) in cases {
            let schedule = CronSchedule::parse(expression).expect("valid");
            let previous = instant - Duration::nanoseconds(1);
            let expected_match =
                schedule.next_after(previous) == Some(instant.to_offset(UtcOffset::UTC));
            assert_eq!(
                schedule.is_match(instant),
                expected_match,
                "expression {expression} disagreed with next_after at {instant:?}"
            );
        }
    }

    // Issue #41: equality and hashing compare the matching instants, never the
    // spelling of the expression that produced them.

    /// Expressions that all impose the same instants as `0 0 * * *`.
    const EQUIVALENT_TO_DAILY_MIDNIGHT: [&str; 6] = [
        "0 0 0 * * *",
        "0 0 */1 * *",
        "0 0 1-31 * *",
        "0 0 * * 0-6",
        "0 0 13 * 0-6",
        "@daily",
    ];

    #[test]
    fn eq_ignores_implicit_versus_explicit_zero_seconds() {
        let five_fields = CronSchedule::parse("0 0 * * *").expect("valid");
        let six_fields = CronSchedule::parse("0 0 0 * * *").expect("valid");
        assert_eq!(five_fields, six_fields);
    }

    #[test]
    fn eq_distinguishes_a_real_seconds_field() {
        let midnight = CronSchedule::parse("0 0 * * *").expect("valid");
        let half_past = CronSchedule::parse("30 0 0 * * *").expect("valid");
        assert_ne!(midnight, half_past);
    }

    #[test]
    fn eq_ignores_a_day_restriction_that_restricts_nothing() {
        let daily = CronSchedule::parse("0 0 * * *").expect("valid");
        for expression in EQUIVALENT_TO_DAILY_MIDNIGHT {
            let equivalent = CronSchedule::parse(expression).expect("valid");
            assert_eq!(daily, equivalent, "{expression} should equal 0 0 * * *");
        }
    }

    // The absorbing case above must not degrade into treating a full field as
    // an absent one: with the day-of-week left as a bare star there is no union
    // to absorb, and day 13 still restricts.
    #[test]
    fn eq_keeps_a_day_of_month_that_alone_restricts() {
        let daily = CronSchedule::parse("0 0 * * *").expect("valid");
        let thirteenth = CronSchedule::parse("0 0 13 * *").expect("valid");
        assert_ne!(daily, thirteenth);
    }

    #[test]
    fn eq_keeps_a_union_of_two_partial_fields() {
        let union = CronSchedule::parse("0 0 1 * 1").expect("valid");
        let day_only = CronSchedule::parse("0 0 1 * *").expect("valid");
        assert_ne!(union, day_only);
    }

    #[test]
    fn hash_deduplicates_semantically_equal_schedules() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(CronSchedule::parse("0 0 * * *").expect("valid"));
        for expression in EQUIVALENT_TO_DAILY_MIDNIGHT {
            set.insert(CronSchedule::parse(expression).expect("valid"));
        }
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn hash_keeps_distinct_schedules_apart() {
        use std::collections::HashSet;
        let distinct = ["0 0 * * *", "30 0 0 * * *", "0 0 13 * *", "0 0 1 * 1"];
        let set: HashSet<CronSchedule> = distinct
            .iter()
            .map(|expression| CronSchedule::parse(expression).expect("valid"))
            .collect();
        assert_eq!(set.len(), distinct.len());
    }

    // Every compared component must be able to tell two schedules apart on its
    // own, otherwise one of them could silently drop out of the comparison.
    // Each expression below differs from the reference in exactly one.
    #[test]
    fn eq_distinguishes_every_compared_component() {
        let reference = CronSchedule::parse("0 0 1 1 1").expect("valid");
        let differing = [
            ("second", "30 0 0 1 1 1"),
            ("minute", "1 0 1 1 1"),
            ("hour", "0 1 1 1 1"),
            ("month", "0 0 1 2 1"),
            ("day filter", "0 0 2 1 1"),
        ];
        for (component, expression) in differing {
            let other = CronSchedule::parse(expression).expect("valid");
            assert_ne!(
                reference, other,
                "{component} should tell 0 0 1 1 1 from {expression}"
            );
        }
    }

    // Equality is structural on the day filter, not extensional on the
    // occurrences: two schedules that never fire stay distinct.
    #[test]
    fn eq_does_not_decide_extensional_equivalence() {
        let never_thirty = CronSchedule::parse("0 0 30 2 *").expect("valid");
        let never_thirty_one = CronSchedule::parse("0 0 31 2 *").expect("valid");
        assert_ne!(never_thirty, never_thirty_one);
    }
}
