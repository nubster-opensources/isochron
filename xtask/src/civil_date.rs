//! Formats a Unix timestamp as a proleptic Gregorian, UTC calendar date.

/// Formats a Unix timestamp as a UTC calendar date, `YYYY-MM-DD`.
pub(crate) fn civil_date_from_unix_seconds(_seconds: u64) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::civil_date_from_unix_seconds;

    #[test]
    fn formats_the_unix_epoch() {
        assert_eq!(civil_date_from_unix_seconds(0), "1970-01-01");
    }

    #[test]
    fn formats_a_leap_day() {
        assert_eq!(civil_date_from_unix_seconds(951_782_400), "2000-02-29");
    }

    #[test]
    fn formats_the_day_after_a_leap_day() {
        assert_eq!(civil_date_from_unix_seconds(951_868_800), "2000-03-01");
    }

    #[test]
    fn formats_an_arbitrary_timestamp() {
        // External oracle: `date -u -d @1782000000 +%Y-%m-%d` prints `2026-06-21`.
        assert_eq!(civil_date_from_unix_seconds(1_782_000_000), "2026-06-21");
    }

    #[test]
    fn formats_the_turn_of_the_twenty_second_century() {
        // External oracle: `date -u -d @4102444800 +%Y-%m-%d` prints `2100-01-01`.
        assert_eq!(civil_date_from_unix_seconds(4_102_444_800), "2100-01-01");
    }
}
