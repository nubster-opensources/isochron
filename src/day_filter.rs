//! The effective day restriction of a schedule, reduced to the predicate it
//! actually imposes.

use time::OffsetDateTime;

use crate::field::FieldSchedule;

/// The day restriction of a schedule, in canonical form.
///
/// Built by [`DayFilter::new`], which folds a restriction accepting every day
/// into [`DayFilter::EveryDay`], so that two schedules imposing the same days
/// hold the same value. Equality and hashing of a schedule compare this filter
/// rather than the spelling of its day fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DayFilter {
    /// Every day matches.
    EveryDay,
    /// Only the day-of-month field restricts.
    DayOfMonth(FieldSchedule),
    /// Only the day-of-week field restricts.
    DayOfWeek(FieldSchedule),
    /// Both fields restrict; a day matches when either does (Vixie union).
    Union {
        /// The accepted days of the month.
        day_of_month: FieldSchedule,
        /// The accepted days of the week.
        day_of_week: FieldSchedule,
    },
}

impl DayFilter {
    /// Build the canonical filter. `None` means the field is a bare `*`.
    ///
    /// A restricted field that accepts its whole range imposes nothing, and
    /// under the Vixie union a full member absorbs the other, so both cases
    /// fold into [`DayFilter::EveryDay`].
    pub(crate) fn new(
        day_of_month: Option<FieldSchedule>,
        day_of_week: Option<FieldSchedule>,
    ) -> Self {
        let _ = (day_of_month, day_of_week);
        unimplemented!("day filter canonicalisation")
    }

    /// Whether `datetime` falls on a day this filter accepts.
    ///
    /// The offset is normalised to UTC before the day is read, so the calendar
    /// day of the argument is not necessarily the day being judged.
    pub(crate) fn matches(self, datetime: OffsetDateTime) -> bool {
        let _ = datetime;
        unimplemented!("day filter matching")
    }
}

#[cfg(test)]
mod tests {
    use super::DayFilter;
    use crate::field::{self, FieldSchedule};
    use time::macros::datetime;

    fn day_of_month(token: &str) -> FieldSchedule {
        FieldSchedule::parse(token, field::DAY_OF_MONTH).expect("valid")
    }

    fn day_of_week(token: &str) -> FieldSchedule {
        FieldSchedule::parse(token, field::DAY_OF_WEEK).expect("valid")
    }

    #[test]
    fn no_restriction_accepts_every_day() {
        assert_eq!(DayFilter::new(None, None), DayFilter::EveryDay);
    }

    #[test]
    fn full_day_of_month_alone_folds() {
        assert_eq!(
            DayFilter::new(Some(day_of_month("1-31")), None),
            DayFilter::EveryDay
        );
    }

    #[test]
    fn partial_day_of_month_alone_is_kept() {
        let days = day_of_month("13");
        assert_eq!(DayFilter::new(Some(days), None), DayFilter::DayOfMonth(days));
    }

    #[test]
    fn full_day_of_week_alone_folds() {
        assert_eq!(
            DayFilter::new(None, Some(day_of_week("0-6"))),
            DayFilter::EveryDay
        );
    }

    #[test]
    fn partial_day_of_week_alone_is_kept() {
        let days = day_of_week("1");
        assert_eq!(DayFilter::new(None, Some(days)), DayFilter::DayOfWeek(days));
    }

    // Under the union a member accepting every day absorbs the other, so the
    // pair imposes nothing even though both fields are restricted.
    #[test]
    fn full_day_of_week_absorbs_a_restricted_day_of_month() {
        assert_eq!(
            DayFilter::new(Some(day_of_month("13")), Some(day_of_week("0-6"))),
            DayFilter::EveryDay
        );
    }

    #[test]
    fn full_day_of_month_absorbs_a_restricted_day_of_week() {
        assert_eq!(
            DayFilter::new(Some(day_of_month("1-31")), Some(day_of_week("1"))),
            DayFilter::EveryDay
        );
    }

    #[test]
    fn two_partial_fields_keep_the_union() {
        let dom = day_of_month("1");
        let dow = day_of_week("1");
        assert_eq!(
            DayFilter::new(Some(dom), Some(dow)),
            DayFilter::Union {
                day_of_month: dom,
                day_of_week: dow,
            }
        );
    }

    #[test]
    fn every_day_matches_any_date() {
        assert!(DayFilter::EveryDay.matches(datetime!(2026-06-16 00:00:00 UTC)));
    }

    // 2026-06-01 is both day 1 and a Monday, 2026-06-15 is a Monday, and
    // 2026-07-01 is day 1 on a Wednesday. 2026-06-16 is neither.
    #[test]
    fn union_matches_when_either_field_matches() {
        let filter = DayFilter::Union {
            day_of_month: day_of_month("1"),
            day_of_week: day_of_week("1"),
        };
        assert!(filter.matches(datetime!(2026-06-01 00:00:00 UTC)));
        assert!(filter.matches(datetime!(2026-06-15 00:00:00 UTC)));
        assert!(filter.matches(datetime!(2026-07-01 00:00:00 UTC)));
        assert!(!filter.matches(datetime!(2026-06-16 00:00:00 UTC)));
    }

    #[test]
    fn day_of_month_filter_ignores_the_weekday() {
        let filter = DayFilter::DayOfMonth(day_of_month("15"));
        assert!(filter.matches(datetime!(2026-06-15 00:00:00 UTC)));
        assert!(!filter.matches(datetime!(2026-06-16 00:00:00 UTC)));
    }

    #[test]
    fn day_of_week_filter_ignores_the_day_of_month() {
        let filter = DayFilter::DayOfWeek(day_of_week("1"));
        assert!(filter.matches(datetime!(2026-06-15 00:00:00 UTC)));
        assert!(filter.matches(datetime!(2026-06-22 00:00:00 UTC)));
        assert!(!filter.matches(datetime!(2026-06-16 00:00:00 UTC)));
    }

    // The offset is normalised before the day is read, so 01:00 at +02:00 is
    // the previous day in UTC and the calendar day of the argument is not the
    // day being judged.
    #[test]
    fn matching_is_evaluated_in_utc() {
        let filter = DayFilter::DayOfMonth(day_of_month("15"));
        assert!(!filter.matches(datetime!(2026-06-15 01:00:00 +02:00)));
        assert!(filter.matches(datetime!(2026-06-16 01:00:00 +02:00)));
    }
}
