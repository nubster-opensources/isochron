//! Deterministic English description of a cron schedule.

use crate::expression::CronSchedule;

const WEEKDAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// Render an English description of `schedule`.
pub(crate) fn describe(schedule: &CronSchedule) -> String {
    let time_clause = time_clause(schedule);
    let day_clause = day_clause(schedule);
    let month_clause = month_clause(schedule);
    format!("{time_clause} {day_clause}{month_clause}")
}

fn time_clause(schedule: &CronSchedule) -> String {
    let minutes = schedule.minute.values();
    let hours = schedule.hour.values();
    let minute_full = schedule.minute.is_full();
    let hour_full = schedule.hour.is_full();

    if minute_full && hour_full {
        "every minute".to_owned()
    } else if minutes.len() == 1 && hours.len() == 1 {
        format!("at {:02}:{:02}", hours[0], minutes[0])
    } else if hour_full {
        format!("at minute {} of every hour", join_numbers(&minutes))
    } else if minute_full {
        format!("every minute past hour {}", join_numbers(&hours))
    } else {
        format!(
            "at minute {} past hour {}",
            join_numbers(&minutes),
            join_numbers(&hours)
        )
    }
}

fn day_clause(schedule: &CronSchedule) -> String {
    match (schedule.dom_restricted, schedule.dow_restricted) {
        (false, false) => "every day".to_owned(),
        (true, false) => {
            format!(
                "on day {} of the month",
                join_numbers(&schedule.day_of_month.values())
            )
        }
        (false, true) => format!(
            "on {}",
            join_named(&schedule.day_of_week.values(), &WEEKDAYS, 0)
        ),
        (true, true) => format!(
            "on day {} of the month or on {}",
            join_numbers(&schedule.day_of_month.values()),
            join_named(&schedule.day_of_week.values(), &WEEKDAYS, 0)
        ),
    }
}

fn month_clause(schedule: &CronSchedule) -> String {
    if schedule.month.is_full() {
        String::new()
    } else {
        format!(" in {}", join_named(&schedule.month.values(), &MONTHS, 1))
    }
}

fn join_numbers(values: &[u8]) -> String {
    let rendered: Vec<String> = values.iter().map(u8::to_string).collect();
    join_with_and(&rendered)
}

fn join_named(values: &[u8], names: &[&str], offset: u8) -> String {
    let rendered: Vec<String> = values
        .iter()
        .map(|value| {
            let index = usize::from(value.saturating_sub(offset));
            names
                .get(index)
                .map_or_else(|| value.to_string(), |name| (*name).to_owned())
        })
        .collect();
    join_with_and(&rendered)
}

fn join_with_and(parts: &[String]) -> String {
    match parts {
        [] => String::new(),
        [single] => single.clone(),
        [head @ .., last] => format!("{} and {last}", head.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::CronSchedule;

    fn describe(expression: &str) -> String {
        CronSchedule::parse(expression).expect("valid").describe()
    }

    #[test]
    fn daily_midnight() {
        assert_eq!(describe("0 0 * * *"), "at 00:00 every day");
    }

    #[test]
    fn day_of_month() {
        assert_eq!(describe("0 0 1 * *"), "at 00:00 on day 1 of the month");
    }

    #[test]
    fn weekday() {
        assert_eq!(describe("0 0 * * 1"), "at 00:00 on Monday");
    }

    #[test]
    fn dom_or_dow_union() {
        assert_eq!(
            describe("0 0 1 * 1"),
            "at 00:00 on day 1 of the month or on Monday"
        );
    }

    #[test]
    fn step_minutes() {
        assert_eq!(
            describe("*/15 * * * *"),
            "at minute 0, 15, 30 and 45 of every hour every day"
        );
    }

    #[test]
    fn month_restricted() {
        assert_eq!(
            describe("0 0 1 1 *"),
            "at 00:00 on day 1 of the month in January"
        );
    }

    #[test]
    fn every_minute() {
        assert_eq!(describe("* * * * *"), "every minute every day");
    }
}
