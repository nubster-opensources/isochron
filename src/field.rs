//! A field schedule: the set of matching values for one cron field, stored as a
//! bitset over the field's inclusive range.

use crate::error::CronError;

/// The set of matching values for one cron field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
            if !is_canonical_decimal(step_token) {
                return Err(invalid(
                    kind,
                    part,
                    "step is not a canonical number (no leading sign or zero-padding)",
                ));
            }
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

    // For day-of-week we allow raw value 7 (Sunday alias) through range
    // resolution so that ranges like "5-7" and "0-7" work correctly.  The
    // remapping 7->0 happens below when we set bits, not here.
    let is_dow = kind.name == DAY_OF_WEEK.name;

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

    // For day-of-week, "5-7" has raw start=5, raw end=7 which is valid (5<=7).
    // For other fields, start > end is always an error.
    if start > end {
        return Err(invalid(kind, part, "range start is after range end"));
    }

    let step = step.unwrap_or(1);
    let mut mask = 0u64;
    let mut value = start;
    while value <= end {
        // For day-of-week, 7 is an alias for Sunday (bit 0).  Apply % 7 only
        // for this field so other fields are never affected.
        let bit = if is_dow {
            u32::from(value) % 7
        } else {
            u32::from(value)
        };
        mask |= 1u64 << bit;
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

    // Day-of-week: 7 is a valid Sunday alias; range check uses max=7 so that
    // "5-7", "0-7", and single "7" all parse.  Bit remapping (7->0) happens in
    // parse_part when the bitset is built, preserving correct range ordering.
    let effective_max = if kind.name == DAY_OF_WEEK.name {
        7
    } else {
        kind.max
    };

    if value < kind.min || value > effective_max {
        return Err(CronError::ValueOutOfRange {
            field: kind.name,
            value: u32::from(value),
            min: kind.min,
            max: kind.max,
        });
    }
    Ok(value)
}

/// Whether `raw` is a canonical non-negative decimal token: ASCII digits only,
/// no leading '+' sign, and no redundant leading zero. The single digit "0" is
/// canonical, but "00", "007" and "+5" are not. `u32`/`u8` parsing would accept
/// the latter silently, so both numeric paths gate on this first.
fn is_canonical_decimal(raw: &str) -> bool {
    !raw.is_empty()
        && raw.bytes().all(|byte| byte.is_ascii_digit())
        && (raw.len() == 1 || raw.as_bytes()[0] != b'0')
}

fn parse_numeric(raw: &str, kind: FieldKind, part: &str) -> Result<u8, CronError> {
    if !is_canonical_decimal(raw) {
        return Err(invalid(
            kind,
            part,
            "not a canonical number (no leading sign or zero-padding)",
        ));
    }
    // Parse as u32 first so out-of-range values produce ValueOutOfRange rather
    // than a generic parse error.
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
    // The ceiling check above guarantees parsed fits in u8; make that
    // invariant explicit rather than silently masking a conversion failure.
    u8::try_from(parsed).map_err(|_| CronError::ValueOutOfRange {
        field: kind.name,
        value: parsed,
        min: kind.min,
        max: kind.max,
    })
}

#[cfg(test)]
mod tests {
    use super::{DAY_OF_WEEK, FieldSchedule, HOUR, MINUTE, MONTH};
    use crate::error::CronError;

    // Issue #3: Vixie-semantics for day-of-week ranges containing 7
    #[test]
    fn weekday_range_five_to_seven_is_fri_sat_sun() {
        // "5-7" = Fri(5), Sat(6), Sun(0) - sorted ascending: [0, 5, 6]
        let field = FieldSchedule::parse("5-7", DAY_OF_WEEK).expect("valid");
        assert_eq!(field.values(), vec![0, 5, 6]);
    }

    #[test]
    fn weekday_range_zero_to_seven_is_all_days() {
        // "0-7" = all seven days
        let field = FieldSchedule::parse("0-7", DAY_OF_WEEK).expect("valid");
        assert_eq!(field.values(), vec![0, 1, 2, 3, 4, 5, 6]);
    }

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

    // Issue #23: standard Vixie cron accepts only plain decimal digits. A
    // leading '+' sign or redundant leading zeros are non-canonical and must
    // be rejected, while the single digit "0" stays valid.
    #[test]
    fn leading_plus_numeric_is_rejected() {
        let error = FieldSchedule::parse("+5", MINUTE).unwrap_err();
        assert!(matches!(
            error,
            CronError::InvalidField {
                field: "minute",
                ..
            }
        ));
    }

    #[test]
    fn zero_padded_numeric_is_rejected() {
        for token in ["007", "00"] {
            let error = FieldSchedule::parse(token, MINUTE).unwrap_err();
            assert!(matches!(
                error,
                CronError::InvalidField {
                    field: "minute",
                    ..
                }
            ));
        }
    }

    #[test]
    fn bare_zero_stays_valid() {
        let field = FieldSchedule::parse("0", MINUTE).expect("valid");
        assert_eq!(field.values(), vec![0]);
    }

    #[test]
    fn weekday_non_canonical_numeric_is_rejected() {
        for token in ["007", "+0"] {
            let error = FieldSchedule::parse(token, DAY_OF_WEEK).unwrap_err();
            assert!(matches!(
                error,
                CronError::InvalidField {
                    field: "day-of-week",
                    ..
                }
            ));
        }
    }

    #[test]
    fn weekday_bare_zero_is_sunday() {
        let field = FieldSchedule::parse("0", DAY_OF_WEEK).expect("valid");
        assert_eq!(field.values(), vec![0]);
    }

    // Issue #27: the step token of `/N` must follow the same canonical rule as
    // values; a leading '+' or zero-padding is rejected.
    #[test]
    fn non_canonical_step_is_rejected() {
        for token in ["*/+5", "*/007", "*/05"] {
            let error = FieldSchedule::parse(token, MINUTE).unwrap_err();
            assert!(matches!(
                error,
                CronError::InvalidField {
                    field: "minute",
                    ..
                }
            ));
        }
    }

    #[test]
    fn canonical_step_stays_valid() {
        let field = FieldSchedule::parse("*/20", MINUTE).expect("valid");
        assert_eq!(field.values(), vec![0, 20, 40]);
    }

    #[test]
    fn zero_step_still_reports_zero_error() {
        let error = FieldSchedule::parse("*/0", MINUTE).unwrap_err();
        assert!(matches!(
            error,
            CronError::InvalidField { field: "minute", reason, .. } if reason.contains("greater than zero")
        ));
    }

    #[test]
    fn non_canonical_error_message_mentions_canonical() {
        for token in ["+5", "*/+5"] {
            let error = FieldSchedule::parse(token, MINUTE).unwrap_err();
            assert!(matches!(
                error,
                CronError::InvalidField { reason, .. } if reason.contains("canonical")
            ));
        }
    }
}
