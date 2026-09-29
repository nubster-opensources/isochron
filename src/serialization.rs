//! Serde support for [`CronSchedule`], behind the `serde` feature.
//!
//! A schedule serializes as the canonical cron expression rendered by its
//! [`Display`](core::fmt::Display) implementation, in every format, and
//! deserializes through [`CronSchedule::parse`] and nothing else.
//!
//! Serialization delegates to `Display` rather than reading the fields again.
//! That is what makes "equal schedules serialize identically" true by
//! construction: `Display` already reads exactly the components that
//! `PartialEq` compares, so nothing here can drift away from equality.

use core::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::CronSchedule;

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl Serialize for CronSchedule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Reads a canonical cron expression, whatever format carries it.
struct CanonicalExpression;

impl Visitor<'_> for CanonicalExpression {
    type Value = CronSchedule;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a cron expression in Vixie cron format")
    }

    fn visit_str<E>(self, expression: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        CronSchedule::parse(expression).map_err(de::Error::custom)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for CronSchedule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(CanonicalExpression)
    }
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::CronSchedule;

    fn parsed(expression: &str) -> CronSchedule {
        CronSchedule::parse(expression).expect("the test expression is valid")
    }

    fn as_json(schedule: &CronSchedule) -> String {
        serde_json::to_string(schedule).expect("a schedule serializes")
    }

    #[test]
    fn serializing_emits_what_display_renders() {
        let five_fields = parsed("0 9 * * 1-5");
        let six_fields = parsed("30 0 9 * * 1-5");

        assert_eq!(as_json(&five_fields), format!("\"{five_fields}\""));
        assert_eq!(as_json(&six_fields), format!("\"{six_fields}\""));
    }

    #[test]
    fn deserializing_restores_every_compared_component() {
        let original = parsed("30 15 8-17 3,6,9 6 2-4");
        let restored: CronSchedule =
            serde_json::from_str(&as_json(&original)).expect("the canonical form parses back");

        assert_eq!(restored.second, original.second);
        assert_eq!(restored.minute, original.minute);
        assert_eq!(restored.hour, original.hour);
        assert_eq!(restored.month, original.month);
        assert_eq!(restored.days, original.days);
    }

    // A six-field spelling whose seconds are zero renders as five fields, so the round
    // trip cannot bring `has_seconds` back. The loss is already part of the canonical
    // contract of #15, and it changes no occurrence: this test pins both halves of that
    // claim, so a future change that made the loss observable would fail here.
    #[test]
    fn a_round_trip_drops_has_seconds_when_the_seconds_are_zero() {
        let original = parsed("0 0 12 * * *");
        assert!(original.has_seconds);

        let restored: CronSchedule =
            serde_json::from_str(&as_json(&original)).expect("the canonical form parses back");

        assert!(!restored.has_seconds);
        assert_eq!(restored, original);

        let after = datetime!(2026-01-01 00:00:00 UTC);
        assert_eq!(restored.next_after(after), original.next_after(after));
    }

    #[test]
    fn a_round_trip_keeps_has_seconds_when_the_seconds_are_not_zero() {
        let original = parsed("30 0 12 * * *");
        let restored: CronSchedule =
            serde_json::from_str(&as_json(&original)).expect("the canonical form parses back");

        assert!(restored.has_seconds);
        assert_eq!(restored, original);
    }

    #[test]
    fn deserializing_reports_the_reason_the_parser_gave() {
        // Five fields are `minute hour day-of-month month day-of-week`, so the out of range
        // value has to sit first for the parser to name the minute field.
        let failure = serde_json::from_str::<CronSchedule>("\"99 0 * * *\"")
            .expect_err("an out of range minute is refused");
        let message = failure.to_string();

        assert!(message.contains("minute"), "message was: {message}");
        assert!(message.contains("out of range"), "message was: {message}");
    }

    #[test]
    fn deserializing_never_yields_a_schedule_from_an_invalid_expression() {
        for expression in ["\"\"", "\"0 0 *\"", "\"0 0 * * 8\"", "\"not a cron\""] {
            assert!(
                serde_json::from_str::<CronSchedule>(expression).is_err(),
                "expected a refusal for {expression}"
            );
        }
    }
}
