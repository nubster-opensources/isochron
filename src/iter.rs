//! A lazy iterator over upcoming occurrences.

use time::OffsetDateTime;

use crate::expression::CronSchedule;

/// A lazy iterator yielding successive occurrences strictly after a cursor.
///
/// Created by [`CronSchedule::upcoming`]. Each call to `next` advances the
/// cursor to the yielded occurrence, so iteration is strictly increasing. It
/// ends when no further occurrence exists within the search horizon.
#[derive(Debug, Clone)]
pub struct Upcoming<'a> {
    schedule: &'a CronSchedule,
    cursor: OffsetDateTime,
}

impl<'a> Upcoming<'a> {
    pub(crate) fn new(schedule: &'a CronSchedule, from: OffsetDateTime) -> Self {
        Self {
            schedule,
            cursor: from,
        }
    }
}

impl Iterator for Upcoming<'_> {
    type Item = OffsetDateTime;

    fn next(&mut self) -> Option<Self::Item> {
        let occurrence = self.schedule.next_after(self.cursor)?;
        self.cursor = occurrence;
        Some(occurrence)
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::CronSchedule;
    use time::macros::datetime;

    #[test]
    fn upcoming_yields_increasing_occurrences() {
        let schedule = CronSchedule::parse("0 0 * * *").expect("valid");
        let occurrences: Vec<_> = schedule
            .upcoming(datetime!(2026-01-01 12:00:00 UTC))
            .take(3)
            .collect();
        assert_eq!(
            occurrences,
            vec![
                datetime!(2026-01-02 00:00:00 UTC),
                datetime!(2026-01-03 00:00:00 UTC),
                datetime!(2026-01-04 00:00:00 UTC),
            ]
        );
    }

    #[test]
    fn time_until_next_measures_the_gap() {
        let schedule = CronSchedule::parse("0 0 * * *").expect("valid");
        let gap = schedule
            .time_until_next(datetime!(2026-01-01 23:00:00 UTC))
            .expect("exists");
        assert_eq!(gap, time::Duration::hours(1));
    }
}
