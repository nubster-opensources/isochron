//! A field schedule: the set of matching values for one cron field, stored as a
//! bitset over the field's inclusive range.

// Items are used by expression, occurrence, iter, describe (added in later tasks).
#![allow(dead_code)]

use crate::error::CronError;

/// The set of matching values for one cron field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FieldSchedule {
    mask: u64,
    min: u8,
    max: u8,
}

/// Static description of a cron field used to drive parsing and error
/// reporting.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FieldKind {
    /// The named field, for example "minute".
    pub name: &'static str,
    /// The lowest valid value.
    pub min: u8,
    /// The highest valid value.
    pub max: u8,
    /// Optional name table; index 0 maps to `min`. For weekdays the table is
    /// `["SUN", "MON", ...]` with index 0 mapping to value 0.
    pub names: Option<&'static [&'static str]>,
}

pub(crate) const SECOND: FieldKind = FieldKind {
    name: "second",
    min: 0,
    max: 59,
    names: None,
};
pub(crate) const MINUTE: FieldKind = FieldKind {
    name: "minute",
    min: 0,
    max: 59,
    names: None,
};
pub(crate) const HOUR: FieldKind = FieldKind {
    name: "hour",
    min: 0,
    max: 23,
    names: None,
};
pub(crate) const DAY_OF_MONTH: FieldKind = FieldKind {
    name: "day-of-month",
    min: 1,
    max: 31,
    names: None,
};
pub(crate) const MONTH: FieldKind = FieldKind {
    name: "month",
    min: 1,
    max: 12,
    names: Some(&[
        "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
    ]),
};
pub(crate) const DAY_OF_WEEK: FieldKind = FieldKind {
    name: "day-of-week",
    min: 0,
    max: 6,
    names: Some(&["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"]),
};

impl FieldSchedule {
    /// Whether `value` is in the set. Values above 63 are never present.
    pub(crate) fn contains(self, value: u8) -> bool {
        value <= 63 && self.mask & (1u64 << value) != 0
    }

    /// The matching values in ascending order.
    pub(crate) fn values(self) -> Vec<u8> {
        (self.min..=self.max)
            .filter(|v| self.contains(*v))
            .collect()
    }

    /// Whether every value in the field range is present.
    pub(crate) fn is_full(self) -> bool {
        (self.min..=self.max).all(|v| self.contains(v))
    }

    /// Parse one cron field token into a `FieldSchedule`.
    ///
    /// # Errors
    ///
    /// Returns [`CronError::InvalidField`] or [`CronError::ValueOutOfRange`] if
    /// the token is malformed or references a value outside the field range.
    pub(crate) fn parse(token: &str, kind: FieldKind) -> Result<Self, CronError> {
        let mut mask = 0u64;
        for part in token.split(',') {
            mask |= parse_part(part, kind)?;
        }
        Ok(Self {
            mask,
            min: kind.min,
            max: kind.max,
        })
    }
}

fn invalid(kind: FieldKind, token: &str, reason: &str) -> CronError {
    CronError::InvalidField {
        field: kind.name,
        token: token.to_owned(),
        reason: reason.to_owned(),
    }
}

fn parse_part(part: &str, kind: FieldKind) -> Result<u64, CronError> {
    if part.is_empty() {
        return Err(invalid(kind, part, "empty list entry"));
    }
    let (range_token, step) = match part.split_once('/') {
        Some((range_token, step_token)) => {
            let step = step_token
                .parse::<u8>()
                .map_err(|_| invalid(kind, part, "step is not a number"))?;
            if step == 0 {
                return Err(invalid(kind, part, "step must be greater than zero"));
            }
            (range_token, Some(step))
        }
        None => (part, None),
    };

    let (start, end) = if range_token == "*" {
        (kind.min, kind.max)
    } else if let Some((low, high)) = range_token.split_once('-') {
        (
            resolve_value(low, kind, part)?,
            resolve_value(high, kind, part)?,
        )
    } else {
        let value = resolve_value(range_token, kind, part)?;
        match step {
            // `a/n` means `a-max/n`.
            Some(_) => (value, kind.max),
            None => (value, value),
        }
    };

    if start > end {
        return Err(invalid(kind, part, "range start is after range end"));
    }

    let step = step.unwrap_or(1);
    let mut mask = 0u64;
    let mut value = start;
    while value <= end {
        mask |= 1u64 << value;
        value = value.saturating_add(step);
    }
    Ok(mask)
}

fn resolve_value(raw: &str, kind: FieldKind, part: &str) -> Result<u8, CronError> {
    let value = if let Some(names) = kind.names {
        match names.iter().position(|name| name.eq_ignore_ascii_case(raw)) {
            Some(index) => kind.min.saturating_add(u8::try_from(index).unwrap_or(0)),
            None => parse_numeric(raw, kind, part)?,
        }
    } else {
        parse_numeric(raw, kind, part)?
    };

    // Day-of-week accepts 7 as an alias for Sunday (0).
    let value = if kind.name == DAY_OF_WEEK.name && value == 7 {
        0
    } else {
        value
    };

    if value < kind.min || value > kind.max {
        return Err(CronError::ValueOutOfRange {
            field: kind.name,
            value: u32::from(value),
            min: kind.min,
            max: kind.max,
        });
    }
    Ok(value)
}

fn parse_numeric(raw: &str, kind: FieldKind, part: &str) -> Result<u8, CronError> {
    // Parse as u32 first so out-of-range values produce ValueOutOfRange rather
    // than a generic parse error, and clamp 7 (day-of-week Sunday alias).
    let parsed = raw
        .parse::<u32>()
        .map_err(|_| invalid(kind, part, "not a number"))?;
    let ceiling = if kind.name == DAY_OF_WEEK.name {
        7
    } else {
        u32::from(kind.max)
    };
    if parsed > ceiling {
        return Err(CronError::ValueOutOfRange {
            field: kind.name,
            value: parsed,
            min: kind.min,
            max: kind.max,
        });
    }
    Ok(u8::try_from(parsed).unwrap_or(kind.max))
}

#[cfg(test)]
mod tests {
    use super::{DAY_OF_WEEK, FieldSchedule, HOUR, MINUTE, MONTH};
    use crate::error::CronError;

    #[test]
    fn wildcard_fills_the_whole_range() {
        let field = FieldSchedule::parse("*", MINUTE).expect("valid");
        assert!(field.is_full());
        assert_eq!(field.values().len(), 60);
    }

    #[test]
    fn single_value_holds_only_that_value() {
        let field = FieldSchedule::parse("30", MINUTE).expect("valid");
        assert_eq!(field.values(), vec![30]);
    }

    #[test]
    fn range_is_inclusive() {
        let field = FieldSchedule::parse("9-11", HOUR).expect("valid");
        assert_eq!(field.values(), vec![9, 10, 11]);
    }

    #[test]
    fn step_over_wildcard() {
        let field = FieldSchedule::parse("*/15", MINUTE).expect("valid");
        assert_eq!(field.values(), vec![0, 15, 30, 45]);
    }

    #[test]
    fn step_over_range() {
        let field = FieldSchedule::parse("0-10/5", MINUTE).expect("valid");
        assert_eq!(field.values(), vec![0, 5, 10]);
    }

    #[test]
    fn value_slash_step_runs_to_max() {
        let field = FieldSchedule::parse("5/10", HOUR).expect("valid");
        assert_eq!(field.values(), vec![5, 15]);
    }

    #[test]
    fn comma_list_unions_entries() {
        let field = FieldSchedule::parse("1,15,30", MINUTE).expect("valid");
        assert_eq!(field.values(), vec![1, 15, 30]);
    }

    #[test]
    fn month_names_are_case_insensitive() {
        let field = FieldSchedule::parse("jan-mar", MONTH).expect("valid");
        assert_eq!(field.values(), vec![1, 2, 3]);
    }

    #[test]
    fn weekday_seven_is_sunday() {
        let field = FieldSchedule::parse("7", DAY_OF_WEEK).expect("valid");
        assert_eq!(field.values(), vec![0]);
    }

    #[test]
    fn weekday_names_resolve() {
        let field = FieldSchedule::parse("MON,FRI", DAY_OF_WEEK).expect("valid");
        assert_eq!(field.values(), vec![1, 5]);
    }

    #[test]
    fn out_of_range_value_is_reported() {
        let error = FieldSchedule::parse("99", MINUTE).unwrap_err();
        assert!(matches!(
            error,
            CronError::ValueOutOfRange {
                field: "minute",
                value: 99,
                ..
            }
        ));
    }

    #[test]
    fn zero_step_is_rejected() {
        let error = FieldSchedule::parse("*/0", MINUTE).unwrap_err();
        assert!(matches!(
            error,
            CronError::InvalidField {
                field: "minute",
                ..
            }
        ));
    }

    #[test]
    fn reversed_range_is_rejected() {
        let error = FieldSchedule::parse("10-5", HOUR).unwrap_err();
        assert!(matches!(
            error,
            CronError::InvalidField { field: "hour", .. }
        ));
    }
}
