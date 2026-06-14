//! Occurrence search: next and previous matching instants.

use time::{Date, Duration, Month, OffsetDateTime, Time, UtcOffset};

use crate::expression::CronSchedule;

/// How many years forward or backward the search scans before giving up. An
/// expression that never matches within this horizon (for example 30 February)
/// yields `None`.
const SEARCH_YEARS: i32 = 5;

impl CronSchedule {
    /// The first occurrence strictly after `after`, or `None` if none exists
    /// within the search horizon.
    #[must_use]
    pub fn next_after(&self, after: OffsetDateTime) -> Option<OffsetDateTime> {
        let after = after.to_offset(UtcOffset::UTC);
        let limit_year = after.year().checked_add(SEARCH_YEARS)?;
        let mut candidate = if self.has_seconds {
            next_second(after)?
        } else {
            start_of_next_minute(after)?
        };
        loop {
            if candidate.year() > limit_year {
                return None;
            }
            if !self.month.contains(u8::from(candidate.month())) {
                candidate = start_of_next_month(candidate)?;
                continue;
            }
            if !self.day_matches(candidate) {
                candidate = start_of_next_day(candidate)?;
                continue;
            }
            if !self.hour.contains(candidate.hour()) {
                candidate = start_of_next_hour(candidate)?;
                continue;
            }
            if !self.minute.contains(candidate.minute()) {
                candidate = start_of_next_minute(candidate)?;
                continue;
            }
            if self.has_seconds && !self.second.contains(candidate.second()) {
                candidate = next_second(candidate)?;
                continue;
            }
            return Some(candidate);
        }
    }
}

fn truncate_to_minute(datetime: OffsetDateTime) -> Option<OffsetDateTime> {
    datetime.replace_second(0).ok()?.replace_nanosecond(0).ok()
}

fn truncate_to_second(datetime: OffsetDateTime) -> Option<OffsetDateTime> {
    datetime.replace_nanosecond(0).ok()
}

fn next_second(datetime: OffsetDateTime) -> Option<OffsetDateTime> {
    truncate_to_second(datetime)?.checked_add(Duration::seconds(1))
}

fn start_of_next_minute(datetime: OffsetDateTime) -> Option<OffsetDateTime> {
    truncate_to_minute(datetime)?.checked_add(Duration::minutes(1))
}

fn start_of_next_hour(datetime: OffsetDateTime) -> Option<OffsetDateTime> {
    datetime
        .replace_minute(0)
        .ok()?
        .replace_second(0)
        .ok()?
        .replace_nanosecond(0)
        .ok()?
        .checked_add(Duration::hours(1))
}

fn start_of_next_day(datetime: OffsetDateTime) -> Option<OffsetDateTime> {
    let date = datetime.date().next_day()?;
    Some(date.with_time(Time::MIDNIGHT).assume_utc())
}

fn start_of_next_month(datetime: OffsetDateTime) -> Option<OffsetDateTime> {
    let (year, month) = match datetime.month() {
        Month::December => (datetime.year().checked_add(1)?, Month::January),
        other => (datetime.year(), other.next()),
    };
    let date = Date::from_calendar_date(year, month, 1).ok()?;
    Some(date.with_time(Time::MIDNIGHT).assume_utc())
}

#[cfg(test)]
mod tests {
    use crate::expression::CronSchedule;
    use time::macros::datetime;

    #[test]
    fn next_after_is_strictly_after() {
        let schedule = CronSchedule::parse("0 0 * * *").expect("valid");
        let next = schedule
            .next_after(datetime!(2026-01-01 00:00:00 UTC))
            .expect("exists");
        assert_eq!(next, datetime!(2026-01-02 00:00:00 UTC));
    }

    #[test]
    fn next_after_within_the_same_day() {
        let schedule = CronSchedule::parse("30 9 * * *").expect("valid");
        let next = schedule
            .next_after(datetime!(2026-01-01 08:00:00 UTC))
            .expect("exists");
        assert_eq!(next, datetime!(2026-01-01 09:30:00 UTC));
    }

    #[test]
    fn next_after_crosses_month_boundary() {
        let schedule = CronSchedule::parse("0 0 1 * *").expect("valid");
        let next = schedule
            .next_after(datetime!(2026-01-15 12:00:00 UTC))
            .expect("exists");
        assert_eq!(next, datetime!(2026-02-01 00:00:00 UTC));
    }

    #[test]
    fn next_after_handles_leap_day() {
        let schedule = CronSchedule::parse("0 0 29 2 *").expect("valid");
        // 2027 is not a leap year, 2028 is.
        let next = schedule
            .next_after(datetime!(2026-03-01 00:00:00 UTC))
            .expect("exists");
        assert_eq!(next, datetime!(2028-02-29 00:00:00 UTC));
    }

    #[test]
    fn impossible_expression_returns_none() {
        let schedule = CronSchedule::parse("0 0 30 2 *").expect("valid");
        assert!(
            schedule
                .next_after(datetime!(2026-01-01 00:00:00 UTC))
                .is_none()
        );
    }

    #[test]
    fn misfire_collapse_returns_first_after_anchor() {
        // An hourly schedule asked far past its last due returns the single next
        // occurrence after the anchor, not every missed one.
        let schedule = CronSchedule::parse("0 * * * *").expect("valid");
        let next = schedule
            .next_after(datetime!(2026-01-01 10:30:00 UTC))
            .expect("exists");
        assert_eq!(next, datetime!(2026-01-01 11:00:00 UTC));
    }

    #[test]
    fn dom_or_dow_union_picks_earliest() {
        // Day-of-month 1 OR Monday. From mid-June 2026 the next is the next
        // Monday (2026-06-15) or day 1 (2026-07-01), whichever is earlier.
        let schedule = CronSchedule::parse("0 0 1 * 1").expect("valid");
        let next = schedule
            .next_after(datetime!(2026-06-10 00:00:00 UTC))
            .expect("exists");
        assert_eq!(next, datetime!(2026-06-15 00:00:00 UTC));
    }

    #[test]
    fn seconds_resolution_advances_by_second() {
        let schedule = CronSchedule::parse("*/30 * * * * *").expect("valid");
        let next = schedule
            .next_after(datetime!(2026-01-01 00:00:05 UTC))
            .expect("exists");
        assert_eq!(next, datetime!(2026-01-01 00:00:30 UTC));
    }
}
