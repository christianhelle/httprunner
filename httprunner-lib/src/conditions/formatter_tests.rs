use super::formatter::*;
use crate::types::ConditionType;

#[test]
fn test_format_condition_type_status() {
    let condition = ConditionType::Status;
    assert_eq!(format_condition_type(&condition), "status");
}

#[test]
fn test_format_condition_type_body_json_path_simple() {
    let condition = ConditionType::BodyJsonPath("$.token".to_string());
    assert_eq!(format_condition_type(&condition), "body.$.token");
}

#[test]
fn test_format_condition_type_body_json_path_nested() {
    let condition = ConditionType::BodyJsonPath("$.user.profile.email".to_string());
    assert_eq!(
        format_condition_type(&condition),
        "body.$.user.profile.email"
    );
}

#[test]
fn test_format_condition_type_body_json_path_array() {
    let condition = ConditionType::BodyJsonPath("$.users[0].id".to_string());
    assert_eq!(format_condition_type(&condition), "body.$.users[0].id");
}

#[test]
fn test_format_condition_type_body_json_path_empty() {
    let condition = ConditionType::BodyJsonPath("".to_string());
    assert_eq!(format_condition_type(&condition), "body.");
}

#[test]
fn test_format_condition_type_body_json_path_complex() {
    let condition = ConditionType::BodyJsonPath("$.data.results[5].metadata.tags[0]".to_string());
    assert_eq!(
        format_condition_type(&condition),
        "body.$.data.results[5].metadata.tags[0]"
    );
}
