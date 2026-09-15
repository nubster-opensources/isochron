//! Formats a Unix timestamp as a proleptic Gregorian, UTC calendar date.

/// Formats a Unix timestamp as a UTC calendar date, `YYYY-MM-DD`.
///
/// Implements Howard Hinnant's `civil_from_days` inverse algorithm for the
/// proleptic Gregorian calendar, which needs no external dependency and is
/// exact over the whole range representable by an `i64` day count.
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub(crate) fn civil_date_from_unix_seconds(seconds: u64) -> String {
    const SECONDS_PER_DAY: u64 = 86_400;

    let days_since_epoch = (seconds / SECONDS_PER_DAY) as i64;
    let shifted_days = days_since_epoch + 719_468;

    let era = shifted_days.div_euclid(146_097);
    let day_of_era = shifted_days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_position = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_position + 2) / 5 + 1;
    let month = if month_position < 10 {
        month_position + 3
    } else {
        month_position - 9
    };
    let year = if month <= 2 { year + 1 } else { year };

    format!("{year:04}-{month:02}-{day:02}")
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
