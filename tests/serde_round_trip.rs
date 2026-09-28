//! Integration tests for the `serde` feature: round trips across JSON and postcard,
//! the canonical-string contract, and the errors raised on invalid input.

#![cfg(feature = "serde")]

use isochron::CronSchedule;

/// Canonical expressions, chosen to exercise five and six field forms, a full field,
/// a range, a comma list, and a step-derived list.
fn canonical_corpus() -> Vec<&'static str> {
    vec![
        "0 0 * * *",
        "0 0 13 * *",
        "0 0 * * 1-5",
        "*/15 * * * *",
        "0 9 1,15 * *",
        "30 0 0 1 1 *",
    ]
}

#[test]
fn json_round_trip_preserves_value_and_canonical_string() {
    for canonical in canonical_corpus() {
        let original = CronSchedule::parse(canonical).unwrap();

        let json = serde_json::to_string(&original).unwrap();
        let restored: CronSchedule = serde_json::from_str(&json).unwrap();

        assert_eq!(restored, original);
        assert_eq!(restored.to_string(), canonical);
    }
}

#[test]
fn postcard_round_trip_preserves_value_and_canonical_string() {
    for canonical in canonical_corpus() {
        let original = CronSchedule::parse(canonical).unwrap();

        let bytes = postcard::to_allocvec(&original).unwrap();
        let restored: CronSchedule = postcard::from_bytes(&bytes).unwrap();

        assert_eq!(restored, original);
        assert_eq!(restored.to_string(), canonical);
    }
}

#[test]
fn value_survives_a_crossing_from_json_to_postcard() {
    let original = CronSchedule::parse("0 0 * * 1-5").unwrap();

    let json = serde_json::to_string(&original).unwrap();
    let via_json: CronSchedule = serde_json::from_str(&json).unwrap();

    let bytes = postcard::to_allocvec(&via_json).unwrap();
    let via_postcard: CronSchedule = postcard::from_bytes(&bytes).unwrap();

    assert_eq!(via_postcard, original);
}

#[test]
fn json_representation_is_a_bare_string_not_an_object_or_array() {
    let schedule = CronSchedule::parse("0 0 13 * *").unwrap();

    let value = serde_json::to_value(&schedule).unwrap();

    match value {
        serde_json::Value::String(text) => assert_eq!(text, "0 0 13 * *"),
        other => panic!("expected a bare JSON string, got {other:?}"),
    }
}

#[test]
fn equivalent_expressions_serialize_to_identical_bytes() {
    let written_as_names = CronSchedule::parse("0 0 * * MON-FRI").unwrap();
    let written_as_numbers = CronSchedule::parse("0 0 * * 1-5").unwrap();

    let json_from_names = serde_json::to_string(&written_as_names).unwrap();
    let json_from_numbers = serde_json::to_string(&written_as_numbers).unwrap();
    assert_eq!(json_from_names, json_from_numbers);

    let postcard_from_names = postcard::to_allocvec(&written_as_names).unwrap();
    let postcard_from_numbers = postcard::to_allocvec(&written_as_numbers).unwrap();
    assert_eq!(postcard_from_names, postcard_from_numbers);
}

#[test]
fn a_non_canonical_source_resolves_to_its_absorbed_canonical_form_when_reserialized() {
    let schedule = CronSchedule::parse("0 0 13 * 0-6").unwrap();

    assert_eq!(schedule.to_string(), "0 0 * * *");

    let json = serde_json::to_string(&schedule).unwrap();
    assert_eq!(json, "\"0 0 * * *\"");
    assert_ne!(json, "\"0 0 13 * 0-6\"");
}

#[test]
fn deserializing_an_invalid_expression_yields_an_error_never_a_schedule() {
    let out_of_range_minute: Result<CronSchedule, _> = serde_json::from_str("\"99 0 * * *\"");
    assert!(out_of_range_minute.is_err());

    let wrong_field_count: Result<CronSchedule, _> = serde_json::from_str("\"0 0 * *\"");
    assert!(wrong_field_count.is_err());

    let empty_expression: Result<CronSchedule, _> = serde_json::from_str("\"\"");
    assert!(empty_expression.is_err());
}

#[test]
fn deserialization_error_message_carries_the_parser_diagnostic() {
    let result: Result<CronSchedule, _> = serde_json::from_str("\"99 0 * * *\"");

    let message = result.unwrap_err().to_string();

    assert!(message.contains("minute"));
    assert!(message.contains("99"));
    assert!(message.contains("out of range"));
}
