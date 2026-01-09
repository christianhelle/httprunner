use super::json_extractor::*;

#[test]
fn test_extract_json_value_simple_path() {
    let json = r#"{"name":"John","age":30}"#;
    let result = extract_json_value(json, "$.name").unwrap();
    assert_eq!(result, Some("John".to_string()));
}

#[test]
fn test_extract_json_value_nested_path() {
    let json = r#"{"user":{"profile":{"email":"test@example.com"}}}"#;
    let result = extract_json_value(json, "$.user.profile.email").unwrap();
    assert_eq!(result, Some("test@example.com".to_string()));
}

#[test]
fn test_extract_json_value_array_index() {
    let json = r#"{"items":[{"id":1},{"id":2},{"id":3}]}"#;
    let result = extract_json_value(json, "$.items[1].id").unwrap();
    assert_eq!(result, Some("2".to_string()));
}

#[test]
fn test_extract_json_value_nonexistent_path() {
    let json = r#"{"name":"John"}"#;
    let result = extract_json_value(json, "$.nonexistent").unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_extract_json_value_without_dollar_prefix() {
    let json = r#"{"name":"John"}"#;
    let result = extract_json_value(json, "name").unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_extract_json_value_number() {
    let json = r#"{"count":42}"#;
    let result = extract_json_value(json, "$.count").unwrap();
    assert_eq!(result, Some("42".to_string()));
}

#[test]
fn test_extract_json_value_boolean() {
    let json = r#"{"active":true}"#;
    let result = extract_json_value(json, "$.active").unwrap();
    assert_eq!(result, Some("true".to_string()));
}

#[test]
fn test_extract_json_value_null() {
    let json = r#"{"value":null}"#;
    let result = extract_json_value(json, "$.value").unwrap();
    assert_eq!(result, Some("null".to_string()));
}

#[test]
fn test_extract_json_value_array() {
    let json = r#"{"tags":["rust","http","cli"]}"#;
    let result = extract_json_value(json, "$.tags").unwrap();
    assert!(result.is_some());
    let value = result.unwrap();
    assert!(value.contains("rust") && value.contains("http"));
}

#[test]
fn test_extract_json_value_object() {
    let json = r#"{"metadata":{"created":"2024-01-01","author":"test"}}"#;
    let result = extract_json_value(json, "$.metadata").unwrap();
    assert!(result.is_some());
    let value = result.unwrap();
    assert!(value.contains("created") && value.contains("author"));
}

#[test]
fn test_extract_json_value_invalid_json() {
    let json = "not valid json";
    let result = extract_json_value(json, "$.name");
    // The underlying extract_json_property function will error on invalid JSON
    // but since it's called within extract_json_value, it propagates the error
    assert!(result.is_err() || result.unwrap().is_none());
}

#[test]
fn test_extract_json_value_empty_path() {
    let json = r#"{"name":"John"}"#;
    let result = extract_json_value(json, "$").unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_extract_json_value_deep_nesting() {
    let json = r#"{"a":{"b":{"c":{"d":{"e":"deep"}}}}}"#;
    let result = extract_json_value(json, "$.a.b.c.d.e").unwrap();
    assert_eq!(result, Some("deep".to_string()));
}
