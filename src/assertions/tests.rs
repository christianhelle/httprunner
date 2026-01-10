use super::*;
use crate::types::{Assertion, AssertionType, HttpResult};
use std::collections::HashMap;

fn build_result() -> HttpResult {
    HttpResult {
        request_name: Some("sample".into()),
        status_code: 200,
        success: true,
        error_message: None,
        duration_ms: 10,
        response_headers: None,
        response_body: Some(r#"{"message":"ok"}"#.to_string()),
        assertion_results: Vec::new(),
    }
}

#[test]
fn status_assertion_succeeds_on_match() {
    let assertion = Assertion {
        assertion_type: AssertionType::Status,
        expected_value: "200".into(),
    };
    let result = evaluate_assertion(&assertion, &build_result());
    assert!(result.passed);
    assert_eq!(result.actual_value.as_deref(), Some("200"));
}

#[test]
fn status_assertion_fails_on_invalid_expected_value() {
    let assertion = Assertion {
        assertion_type: AssertionType::Status,
        expected_value: "two-hundred".into(),
    };
    let result = evaluate_assertion(&assertion, &build_result());
    assert!(!result.passed);
    assert_eq!(
        result.error_message.as_deref(),
        Some("Invalid expected status code format")
    );
}

#[test]
fn body_assertion_handles_missing_body() {
    let assertion = Assertion {
        assertion_type: AssertionType::Body,
        expected_value: "token".into(),
    };
    let mut result = build_result();
    result.response_body = None;
    let eval = evaluate_assertion(&assertion, &result);
    assert!(!eval.passed);
    assert_eq!(
        eval.error_message.as_deref(),
        Some("No response body available")
    );
}

#[test]
fn headers_assertion_is_case_insensitive() {
    let assertion = Assertion {
        assertion_type: AssertionType::Headers,
        expected_value: "Content-Type: json".into(),
    };

    let mut headers = HashMap::new();
    headers.insert("content-type".into(), "application/json".into());

    let mut result = build_result();
    result.response_headers = Some(headers);

    let eval = evaluate_assertion(&assertion, &result);
    assert!(eval.passed);
}

#[test]
fn headers_assertion_reports_invalid_format() {
    let assertion = Assertion {
        assertion_type: AssertionType::Headers,
        expected_value: "Missing colon".into(),
    };

    let mut headers = HashMap::new();
    headers.insert("X-Test".into(), "value".into());

    let mut result = build_result();
    result.response_headers = Some(headers);

    let eval = evaluate_assertion(&assertion, &result);
    assert!(!eval.passed);
    assert_eq!(
        eval.error_message.as_deref(),
        Some("Invalid header format, expected 'Name: Value'")
    );
}

#[test]
fn test_evaluate_assertions_multiple() {
    let assertions = vec![
        Assertion {
            assertion_type: AssertionType::Status,
            expected_value: "200".into(),
        },
        Assertion {
            assertion_type: AssertionType::Body,
            expected_value: "ok".into(),
        },
    ];
    
    let results = evaluate_assertions(&assertions, &build_result());
    assert_eq!(results.len(), 2);
    assert!(results[0].passed);
    assert!(results[1].passed);
}

#[test]
fn test_body_assertion_success() {
    let assertion = Assertion {
        assertion_type: AssertionType::Body,
        expected_value: "message".into(),
    };
    let result = evaluate_assertion(&assertion, &build_result());
    assert!(result.passed);
}

#[test]
fn test_body_assertion_failure() {
    let assertion = Assertion {
        assertion_type: AssertionType::Body,
        expected_value: "notfound".into(),
    };
    let result = evaluate_assertion(&assertion, &build_result());
    assert!(!result.passed);
    assert!(result.error_message.is_some());
}

#[test]
fn test_headers_assertion_missing_headers() {
    let assertion = Assertion {
        assertion_type: AssertionType::Headers,
        expected_value: "X-Custom: value".into(),
    };
    
    let mut result = build_result();
    result.response_headers = None;
    
    let eval = evaluate_assertion(&assertion, &result);
    assert!(!eval.passed);
    assert_eq!(
        eval.error_message.as_deref(),
        Some("No response headers available")
    );
}

#[test]
fn test_headers_assertion_header_not_found() {
    let assertion = Assertion {
        assertion_type: AssertionType::Headers,
        expected_value: "X-Custom: value".into(),
    };
    
    let mut headers = HashMap::new();
    headers.insert("Content-Type".into(), "application/json".into());
    
    let mut result = build_result();
    result.response_headers = Some(headers);
    
    let eval = evaluate_assertion(&assertion, &result);
    assert!(!eval.passed);
    assert!(eval.error_message.is_some());
}

#[test]
fn test_status_assertion_failure() {
    let assertion = Assertion {
        assertion_type: AssertionType::Status,
        expected_value: "404".into(),
    };
    let result = evaluate_assertion(&assertion, &build_result());
    assert!(!result.passed);
    assert!(result.error_message.is_some());
    assert!(result.error_message.unwrap().contains("Expected status 404, got 200"));
}
