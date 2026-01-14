use super::condition_parser::*;
use crate::types::ConditionType;

#[test]
fn test_parse_condition_status() {
    let result = parse_condition("login.response.status 200", false).unwrap();

    assert_eq!(result.request_name, "login");
    assert!(matches!(result.condition_type, ConditionType::Status));
    assert_eq!(result.expected_value, "200");
    assert!(!result.negate);
}

#[test]
fn test_parse_condition_status_negated() {
    let result = parse_condition("login.response.status 404", true).unwrap();

    assert_eq!(result.request_name, "login");
    assert!(matches!(result.condition_type, ConditionType::Status));
    assert_eq!(result.expected_value, "404");
    assert!(result.negate);
}

#[test]
fn test_parse_condition_body_jsonpath() {
    let result = parse_condition("login.response.body.$.token secret123", false).unwrap();

    assert_eq!(result.request_name, "login");
    match &result.condition_type {
        ConditionType::BodyJsonPath(path) => {
            assert_eq!(path, "$.token");
        }
        _ => panic!("Expected BodyJsonPath"),
    }
    assert_eq!(result.expected_value, "secret123");
}

#[test]
fn test_parse_condition_body_jsonpath_nested() {
    let result =
        parse_condition("user.response.body.$.profile.email test@example.com", false).unwrap();

    assert_eq!(result.request_name, "user");
    match &result.condition_type {
        ConditionType::BodyJsonPath(path) => {
            assert_eq!(path, "$.profile.email");
        }
        _ => panic!("Expected BodyJsonPath"),
    }
    assert_eq!(result.expected_value, "test@example.com");
}

#[test]
fn test_parse_condition_expected_value_with_spaces() {
    let result = parse_condition("req.response.status 200 OK", false).unwrap();

    assert_eq!(result.expected_value, "200 OK");
}

#[test]
fn test_parse_condition_invalid_too_few_parts() {
    let result = parse_condition("login.response", false);
    assert!(result.is_none());
}

#[test]
fn test_parse_condition_invalid_missing_expected_value() {
    let result = parse_condition("login.response.status", false);
    assert!(result.is_none());
}

#[test]
fn test_parse_condition_invalid_reference_format() {
    let result = parse_condition("login 200", false);
    assert!(result.is_none());
}

#[test]
fn test_parse_condition_complex_request_name() {
    let result = parse_condition("createUser.response.status 201", false).unwrap();

    assert_eq!(result.request_name, "createUser");
}

#[test]
fn test_parse_condition_numeric_expected_value() {
    let result = parse_condition("api.response.body.$.count 42", false).unwrap();

    assert_eq!(result.expected_value, "42");
}

#[test]
fn test_parse_condition_boolean_expected_value() {
    let result = parse_condition("check.response.body.$.active true", false).unwrap();

    assert_eq!(result.expected_value, "true");
}

#[test]
fn test_parse_condition_empty_string() {
    let result = parse_condition("", false);
    assert!(result.is_none());
}

#[test]
fn test_parse_condition_only_whitespace() {
    let result = parse_condition("   ", false);
    assert!(result.is_none());
}

#[test]
fn test_parse_condition_array_jsonpath() {
    let result = parse_condition("list.response.body.$.items[0].id 123", false).unwrap();

    match &result.condition_type {
        ConditionType::BodyJsonPath(path) => {
            assert_eq!(path, "$.items[0].id");
        }
        _ => panic!("Expected BodyJsonPath"),
    }
}

#[test]
fn test_parse_condition_multiple_spaces_in_value() {
    let result = parse_condition("req.response.status   200   OK  ", false).unwrap();

    // split_whitespace() normalizes multiple spaces
    assert_eq!(result.expected_value, "200 OK");
}

#[test]
fn test_parse_condition_special_chars_in_value() {
    let result = parse_condition("req.response.body.$.msg Hello, World!", false).unwrap();

    assert_eq!(result.expected_value, "Hello, World!");
}

#[test]
fn test_parse_condition_invalid_source() {
    // Using "request" instead of "response" should fail
    let result = parse_condition("login.request.status 200", false);
    assert!(result.is_none());
}

#[test]
fn test_parse_condition_invalid_target() {
    // Using "headers" instead of "status" or "body" should fail
    let result = parse_condition("login.response.headers 200", false);
    assert!(result.is_none());
}
