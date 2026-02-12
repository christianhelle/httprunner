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
    assert!(
        result
            .error_message
            .unwrap()
            .contains("Expected status 404, got 200")
    );
}

// Tests for assertion-based success determination with non-2xx status codes.
// These verify that assertions correctly evaluate against non-2xx responses,
// which is critical for scenarios like testing HTTP 400/404/500 responses.

#[test]
fn status_assertion_passes_for_expected_400() {
    let assertion = Assertion {
        assertion_type: AssertionType::Status,
        expected_value: "400".into(),
    };
    let mut result = build_result();
    result.status_code = 400;
    result.success = false;

    let eval = evaluate_assertion(&assertion, &result);
    assert!(eval.passed);
    assert_eq!(eval.actual_value.as_deref(), Some("400"));
    assert!(eval.error_message.is_none());
}

#[test]
fn status_assertion_passes_for_expected_404() {
    let assertion = Assertion {
        assertion_type: AssertionType::Status,
        expected_value: "404".into(),
    };
    let mut result = build_result();
    result.status_code = 404;
    result.success = false;

    let eval = evaluate_assertion(&assertion, &result);
    assert!(eval.passed);
    assert_eq!(eval.actual_value.as_deref(), Some("404"));
    assert!(eval.error_message.is_none());
}

#[test]
fn status_assertion_passes_for_expected_500() {
    let assertion = Assertion {
        assertion_type: AssertionType::Status,
        expected_value: "500".into(),
    };
    let mut result = build_result();
    result.status_code = 500;
    result.success = false;

    let eval = evaluate_assertion(&assertion, &result);
    assert!(eval.passed);
    assert_eq!(eval.actual_value.as_deref(), Some("500"));
}

#[test]
fn status_assertion_fails_when_expected_400_but_got_200() {
    let assertion = Assertion {
        assertion_type: AssertionType::Status,
        expected_value: "400".into(),
    };
    let result = build_result(); // status_code = 200

    let eval = evaluate_assertion(&assertion, &result);
    assert!(!eval.passed);
    assert!(eval.error_message.unwrap().contains("Expected status 400, got 200"));
}

#[test]
fn all_assertions_pass_for_non_2xx_with_matching_status_and_body() {
    let assertions = vec![
        Assertion {
            assertion_type: AssertionType::Status,
            expected_value: "400".into(),
        },
        Assertion {
            assertion_type: AssertionType::Body,
            expected_value: "invalid".into(),
        },
    ];

    let mut result = build_result();
    result.status_code = 400;
    result.success = false;
    result.response_body = Some(r#"{"error":"invalid request"}"#.to_string());

    let results = evaluate_assertions(&assertions, &result);
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.passed));
}

#[test]
fn mixed_assertions_fail_when_body_does_not_match_for_non_2xx() {
    let assertions = vec![
        Assertion {
            assertion_type: AssertionType::Status,
            expected_value: "400".into(),
        },
        Assertion {
            assertion_type: AssertionType::Body,
            expected_value: "specific error".into(),
        },
    ];

    let mut result = build_result();
    result.status_code = 400;
    result.success = false;
    result.response_body = Some(r#"{"error":"different error"}"#.to_string());

    let results = evaluate_assertions(&assertions, &result);
    assert_eq!(results.len(), 2);
    assert!(results[0].passed);  // status matches
    assert!(!results[1].passed); // body does not match
    assert!(!results.iter().all(|r| r.passed));
}

#[test]
fn all_assertions_pass_for_404_with_matching_status_body_and_headers() {
    let assertions = vec![
        Assertion {
            assertion_type: AssertionType::Status,
            expected_value: "404".into(),
        },
        Assertion {
            assertion_type: AssertionType::Body,
            expected_value: "not found".into(),
        },
        Assertion {
            assertion_type: AssertionType::Headers,
            expected_value: "Content-Type: application/json".into(),
        },
    ];

    let mut headers = HashMap::new();
    headers.insert("content-type".into(), "application/json".into());

    let mut result = build_result();
    result.status_code = 404;
    result.success = false;
    result.response_body = Some(r#"{"error":"not found"}"#.to_string());
    result.response_headers = Some(headers);

    let results = evaluate_assertions(&assertions, &result);
    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.passed));
}

#[test]
fn status_assertion_passes_for_expected_422() {
    let assertion = Assertion {
        assertion_type: AssertionType::Status,
        expected_value: "422".into(),
    };
    let mut result = build_result();
    result.status_code = 422;
    result.success = false;

    let eval = evaluate_assertion(&assertion, &result);
    assert!(eval.passed);
}

#[test]
fn status_assertion_passes_for_expected_429() {
    let assertion = Assertion {
        assertion_type: AssertionType::Status,
        expected_value: "429".into(),
    };
    let mut result = build_result();
    result.status_code = 429;
    result.success = false;

    let eval = evaluate_assertion(&assertion, &result);
    assert!(eval.passed);
}

#[test]
fn status_assertion_passes_for_expected_503() {
    let assertion = Assertion {
        assertion_type: AssertionType::Status,
        expected_value: "503".into(),
    };
    let mut result = build_result();
    result.status_code = 503;
    result.success = false;

    let eval = evaluate_assertion(&assertion, &result);
    assert!(eval.passed);
}
