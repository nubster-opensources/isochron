//! A parsed cron schedule and its public parsing entry points.

use core::fmt;
use core::str::FromStr;

use time::{OffsetDateTime, UtcOffset};

use crate::error::CronError;
use crate::field::{self, FieldSchedule};

/// A parsed cron schedule, evaluated in strict UTC.
///
/// Build one with [`CronSchedule::parse`]. Compute occurrences with
/// [`CronSchedule::next_after`] and [`CronSchedule::prev_before`].
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// when a leading seconds field is present, plus the macros `@yearly`,
    /// `@annually`, `@monthly`, `@weekly`, `@daily`, `@midnight`, and `@hourly`.
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

    /// An English human-readable description of the schedule.
    #[must_use]
    pub fn describe(&self) -> String {
        crate::describe::describe(self)
    }

    /// Whether the given instant matches the schedule. UTC is enforced.
    // Used in expression tests and future tasks; occurrence uses day_matches directly.
    #[allow(dead_code)]
    pub(crate) fn matches(&self, datetime: OffsetDateTime) -> bool {
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
        assert!(schedule.matches(datetime!(2026-06-15 00:00:00 UTC))); // Monday
        assert!(schedule.matches(datetime!(2026-07-01 00:00:00 UTC))); // day 1
        assert!(!schedule.matches(datetime!(2026-06-16 00:00:00 UTC))); // neither
    }

    #[test]
    fn matches_with_single_day_constraint() {
        let schedule = CronSchedule::parse("0 0 15 * *").expect("valid");
        assert!(schedule.matches(datetime!(2026-06-15 00:00:00 UTC)));
        assert!(!schedule.matches(datetime!(2026-06-16 00:00:00 UTC)));
    }

    #[test]
    fn display_round_trips() {
        let schedule = CronSchedule::parse("  0   0 1,15 * MON  ").expect("valid");
        let rendered = schedule.to_string();
        let reparsed = CronSchedule::parse(&rendered).expect("valid");
        assert_eq!(schedule, reparsed);
    }
}
