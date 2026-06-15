//! Typed errors returned when parsing a cron expression.

use thiserror::Error;

/// An error encountered while parsing a cron expression.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum CronError {
    /// The expression was empty or only whitespace.
    #[error("cron expression must not be empty")]
    EmptyExpression,

    /// The expression did not carry five or six fields.
    #[error("cron expression must carry five or six fields, found {found}")]
    WrongFieldCount {
        /// The number of whitespace-separated fields found.
        found: usize,
    },

    /// A field could not be parsed.
    #[error("invalid {field} field, token \"{token}\": {reason}")]
    InvalidField {
        /// The named field, for example "minute".
        field: &'static str,
        /// The offending token within the field.
        token: String,
        /// Why the token is invalid.
        reason: String,
    },

    /// A value fell outside the field's allowed range.
    #[error("{field} value {value} is out of range {min}..={max}")]
    ValueOutOfRange {
        /// The named field.
        field: &'static str,
        /// The offending value.
        value: u32,
        /// The lowest allowed value.
        min: u8,
        /// The highest allowed value.
        max: u8,
    },
}

#[cfg(test)]
mod tests {
    use super::CronError;

    #[test]
    fn wrong_field_count_message_mentions_the_count() {
        let error = CronError::WrongFieldCount { found: 3 };
        assert_eq!(
            error.to_string(),
            "cron expression must carry five or six fields, found 3"
        );
    }

    #[test]
    fn invalid_field_message_mentions_field_and_token() {
        let error = CronError::InvalidField {
            field: "minute",
            token: "99".to_owned(),
            reason: "value out of range".to_owned(),
        };
        assert_eq!(
            error.to_string(),
            "invalid minute field, token \"99\": value out of range"
        );
    }

    #[test]
    fn out_of_range_message_mentions_bounds() {
        let error = CronError::ValueOutOfRange {
            field: "hour",
            value: 24,
            min: 0,
            max: 23,
        };
        assert_eq!(error.to_string(), "hour value 24 is out of range 0..=23");
    }
}
