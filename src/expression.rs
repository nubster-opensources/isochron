//! A parsed cron schedule and its public parsing entry points.

use core::fmt;
use core::str::FromStr;

use time::{Duration, OffsetDateTime, UtcOffset};

use crate::iter::Upcoming;

use crate::error::CronError;
use crate::field::{self, FieldSchedule};

/// A parsed cron schedule, evaluated in strict UTC.
///
/// Build one with [`CronSchedule::parse`]. Compute occurrences with
/// [`CronSchedule::next_after`] and [`CronSchedule::prev_before`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CronSchedule {
    pub(crate) second: FieldSchedule,
    pub(crate) minute: FieldSchedule,
    pub(crate) hour: FieldSchedule,
    pub(crate) day_of_month: FieldSchedule,
    pub(crate) month: FieldSchedule,
    pub(crate) day_of_week: FieldSchedule,
    pub(crate) has_seconds: bool,
    pub(crate) dom_restricted: bool,
    pub(crate) dow_restricted: bool,
    normalized: String,
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
    /// **Sunday in day-of-week.** Both `0` and `7` denote Sunday. `7` is valid
    /// in ranges: `5-7` matches Friday, Saturday, and Sunday.
    ///
    /// **Day union (Vixie semantics).** When BOTH the day-of-month and
    /// day-of-week fields are restricted (i.e. not a bare `*`), a day matches if
    /// EITHER field matches (OR logic). Only the literal `*` disables a field's
    /// restriction; a range such as `1-31` still counts as restricted. This
    /// differs from Quartz, which uses AND logic with an explicit `?` placeholder.
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
            day_of_month,
            month,
            day_of_week,
            has_seconds,
            dom_restricted: dom_token != "*",
            dow_restricted: weekday_token != "*",
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
    #[must_use]
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
    #[must_use]
    pub fn describe(&self) -> String {
        crate::describe::describe(self)
    }

    /// Returns true if `datetime` is an occurrence of this schedule, evaluated in UTC.
    ///
    /// The offset of `datetime` is normalised to UTC before matching, so any
    /// `OffsetDateTime` is accepted regardless of its original offset.
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
    /// ```
    #[must_use]
    pub fn is_match(&self, datetime: OffsetDateTime) -> bool {
        let datetime = datetime.to_offset(UtcOffset::UTC);
        self.second.contains(datetime.second())
            && self.minute.contains(datetime.minute())
            && self.hour.contains(datetime.hour())
            && self.month.contains(u8::from(datetime.month()))
            && self.day_matches(datetime)
    }

    /// The day-of-month / day-of-week union rule.
    pub(crate) fn day_matches(&self, datetime: OffsetDateTime) -> bool {
        let dom = self.day_of_month.contains(datetime.day());
        let dow = self
            .day_of_week
            .contains(datetime.weekday().number_days_from_sunday());
        match (self.dom_restricted, self.dow_restricted) {
            (true, true) => dom || dow,
            (true, false) => dom,
            (false, true) => dow,
            (false, false) => true,
        }
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
    use crate::error::CronError;
    use time::macros::datetime;

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
    fn restricted_flags_track_wildcards() {
        let schedule = CronSchedule::parse("0 0 1 * 1").expect("valid");
        assert!(schedule.dom_restricted);
        assert!(schedule.dow_restricted);
        let loose = CronSchedule::parse("0 0 * * *").expect("valid");
        assert!(!loose.dom_restricted);
        assert!(!loose.dow_restricted);
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
}
