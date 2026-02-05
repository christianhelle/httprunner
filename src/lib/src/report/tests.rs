use super::formatter::*;
use super::*;
use crate::types::{
    Assertion, AssertionResult, AssertionType, Condition, ConditionType, Header, HttpFileResults,
    HttpRequest, HttpResult, ProcessorResults, RequestContext,
};
use std::collections::HashMap;
use std::fs;

#[test]
fn escape_markdown_escapes_pipe_character() {
    assert_eq!(escape_markdown("hello|world"), "hello\\|world");
    assert_eq!(escape_markdown("no pipes here"), "no pipes here");
    assert_eq!(escape_markdown("|||"), "\\|\\|\\|");
}

fn sample_request(name: &str, method: &str, url: &str) -> HttpRequest {
    HttpRequest {
        name: Some(name.to_string()),
        method: method.to_string(),
        url: url.to_string(),
        headers: vec![],
        body: None,
        assertions: vec![],
        variables: vec![],
        timeout: None,
        connection_timeout: None,
        depends_on: None,
        conditions: vec![],
        pre_delay_ms: None,
        post_delay_ms: None,
    }
}

fn sample_result(status: u16, success: bool, duration: u64) -> HttpResult {
    HttpResult {
        request_name: Some("test".to_string()),
        status_code: status,
        success,
        error_message: None,
        duration_ms: duration,
        response_headers: None,
        response_body: None,
        assertion_results: vec![],
    }
}

#[test]
fn generate_markdown_creates_file() {
    let results = ProcessorResults {
        success: true,
        files: vec![],
    };

    let filename = generate_markdown(&results).unwrap();
    assert!(filename.starts_with("httprunner-report-"));
    assert!(filename.ends_with(".md"));

    assert!(std::path::Path::new(&filename).exists());

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_includes_overall_summary() {
    let request = sample_request("test_req", "GET", "https://example.com");
    let result = sample_result(200, true, 100);

    let context = RequestContext {
        name: "test_req".to_string(),
        request,
        result: Some(result),
    };

    let file_results = HttpFileResults {
        filename: "test.http".to_string(),
        success_count: 1,
        failed_count: 0,
        skipped_count: 0,
        result_contexts: vec![context],
    };

    let results = ProcessorResults {
        success: true,
        files: vec![file_results],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("# HTTP File Runner - Test Report"));
    assert!(content.contains("## Overall Summary"));
    assert!(content.contains("- **Total Requests:** 1"));
    assert!(content.contains("- **Passed:** ✅ 1"));
    assert!(content.contains("- **Failed:** ❌ 0"));
    assert!(content.contains("- **Skipped:** ⏭️ 0"));
    assert!(content.contains("- **Success Rate:** 100.0%"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_calculates_success_rate_correctly() {
    let contexts = vec![
        RequestContext {
            name: "req1".to_string(),
            request: sample_request("req1", "GET", "https://example.com"),
            result: Some(sample_result(200, true, 100)),
        },
        RequestContext {
            name: "req2".to_string(),
            request: sample_request("req2", "GET", "https://example.com"),
            result: Some(sample_result(404, false, 100)),
        },
    ];

    let file_results = HttpFileResults {
        filename: "test.http".to_string(),
        success_count: 1,
        failed_count: 1,
        skipped_count: 0,
        result_contexts: contexts,
    };

    let results = ProcessorResults {
        success: false,
        files: vec![file_results],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("- **Total Requests:** 2"));
    assert!(content.contains("- **Success Rate:** 50.0%"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_handles_zero_requests() {
    let results = ProcessorResults {
        success: true,
        files: vec![HttpFileResults {
            filename: "empty.http".to_string(),
            success_count: 0,
            failed_count: 0,
            skipped_count: 0,
            result_contexts: vec![],
        }],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("- **Total Requests:** 0"));
    assert!(content.contains("- **Success Rate:** 0.0%"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_includes_request_headers() {
    let mut request = sample_request("test", "POST", "https://api.example.com");
    request.headers = vec![
        Header {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
        Header {
            name: "Authorization".to_string(),
            value: "Bearer token|123".to_string(),
        },
    ];

    let context = RequestContext {
        name: "test".to_string(),
        request,
        result: Some(sample_result(200, true, 100)),
    };

    let file_results = HttpFileResults {
        filename: "test.http".to_string(),
        success_count: 1,
        failed_count: 0,
        skipped_count: 0,
        result_contexts: vec![context],
    };

    let results = ProcessorResults {
        success: true,
        files: vec![file_results],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("**Headers:**"));
    assert!(content.contains("| Header | Value |"));
    assert!(content.contains("| Content-Type | application/json |"));
    assert!(content.contains("| Authorization | Bearer token\\|123 |"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_includes_request_body() {
    let mut request = sample_request("test", "POST", "https://api.example.com");
    request.body = Some(r#"{"name":"John","age":30}"#.to_string());

    let context = RequestContext {
        name: "test".to_string(),
        request,
        result: Some(sample_result(201, true, 150)),
    };

    let file_results = HttpFileResults {
        filename: "test.http".to_string(),
        success_count: 1,
        failed_count: 0,
        skipped_count: 0,
        result_contexts: vec![context],
    };

    let results = ProcessorResults {
        success: true,
        files: vec![file_results],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("**Request Body:**"));
    assert!(content.contains("```"));
    assert!(content.contains(r#"{"name":"John","age":30}"#));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_includes_response_details() {
    let mut result = sample_result(200, true, 250);

    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("Server".to_string(), "nginx|1.18".to_string());
    result.response_headers = Some(headers);
    result.response_body = Some(r#"{"status":"ok"}"#.to_string());

    let context = RequestContext {
        name: "test".to_string(),
        request: sample_request("test", "GET", "https://api.example.com"),
        result: Some(result),
    };

    let file_results = HttpFileResults {
        filename: "test.http".to_string(),
        success_count: 1,
        failed_count: 0,
        skipped_count: 0,
        result_contexts: vec![context],
    };

    let results = ProcessorResults {
        success: true,
        files: vec![file_results],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("#### Response Details"));
    assert!(content.contains("- **Status:** ✅ 200"));
    assert!(content.contains("- **Duration:** 250ms"));
    assert!(content.contains("**Response Headers:**"));
    assert!(content.contains("| Content-Type | application/json |"));
    assert!(content.contains("| Server | nginx\\|1.18 |"));
    assert!(content.contains("**Response Body:**"));
    assert!(content.contains(r#"{"status":"ok"}"#));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_includes_assertions() {
    let assertion = Assertion {
        assertion_type: AssertionType::Status,
        expected_value: "200".to_string(),
    };

    let assertion_result = AssertionResult {
        assertion: assertion.clone(),
        passed: true,
        actual_value: Some("200".to_string()),
        error_message: None,
    };

    let mut result = sample_result(200, true, 100);
    result.assertion_results = vec![assertion_result];

    let mut request = sample_request("test", "GET", "https://example.com");
    request.assertions = vec![assertion];

    let context = RequestContext {
        name: "test".to_string(),
        request,
        result: Some(result),
    };

    let file_results = HttpFileResults {
        filename: "test.http".to_string(),
        success_count: 1,
        failed_count: 0,
        skipped_count: 0,
        result_contexts: vec![context],
    };

    let results = ProcessorResults {
        success: true,
        files: vec![file_results],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("#### Assertion Results"));
    assert!(content.contains("| Type | Expected | Actual | Result |"));
    assert!(content.contains("| Status Code | 200 | 200 | ✅ |"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_includes_failed_assertions() {
    let assertion = Assertion {
        assertion_type: AssertionType::Body,
        expected_value: "success".to_string(),
    };

    let assertion_result = AssertionResult {
        assertion: assertion.clone(),
        passed: false,
        actual_value: Some("error".to_string()),
        error_message: Some("Body mismatch".to_string()),
    };

    let mut result = sample_result(200, false, 100);
    result.assertion_results = vec![assertion_result];

    let mut request = sample_request("test", "GET", "https://example.com");
    request.assertions = vec![assertion];

    let context = RequestContext {
        name: "test".to_string(),
        request,
        result: Some(result),
    };

    let file_results = HttpFileResults {
        filename: "test.http".to_string(),
        success_count: 0,
        failed_count: 1,
        skipped_count: 0,
        result_contexts: vec![context],
    };

    let results = ProcessorResults {
        success: false,
        files: vec![file_results],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("| Response Body | success | error | ❌ |"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_handles_skipped_requests() {
    let request = sample_request("skipped", "GET", "https://example.com");

    let context = RequestContext {
        name: "skipped".to_string(),
        request,
        result: None,
    };

    let file_results = HttpFileResults {
        filename: "test.http".to_string(),
        success_count: 0,
        failed_count: 0,
        skipped_count: 1,
        result_contexts: vec![context],
    };

    let results = ProcessorResults {
        success: true,
        files: vec![file_results],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("⏭️ **Request was skipped**"));
    assert!(content.contains("- **Skipped:** ⏭️ 1"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_includes_timeouts() {
    let mut request = sample_request("test", "GET", "https://example.com");
    request.timeout = Some(5000);
    request.connection_timeout = Some(3000);

    let context = RequestContext {
        name: "test".to_string(),
        request,
        result: Some(sample_result(200, true, 100)),
    };

    let file_results = HttpFileResults {
        filename: "test.http".to_string(),
        success_count: 1,
        failed_count: 0,
        skipped_count: 0,
        result_contexts: vec![context],
    };

    let results = ProcessorResults {
        success: true,
        files: vec![file_results],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("- **Timeout:** 5000ms"));
    assert!(content.contains("- **Connection Timeout:** 3000ms"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_includes_dependencies() {
    let mut request = sample_request("dependent", "POST", "https://example.com");
    request.depends_on = Some("login".to_string());

    let context = RequestContext {
        name: "dependent".to_string(),
        request,
        result: Some(sample_result(200, true, 100)),
    };

    let file_results = HttpFileResults {
        filename: "test.http".to_string(),
        success_count: 1,
        failed_count: 0,
        skipped_count: 0,
        result_contexts: vec![context],
    };

    let results = ProcessorResults {
        success: true,
        files: vec![file_results],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("- **Depends On:** `login`"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_includes_conditions() {
    let mut request = sample_request("conditional", "GET", "https://example.com");
    request.conditions = vec![
        Condition {
            request_name: "login".to_string(),
            condition_type: ConditionType::Status,
            expected_value: "200".to_string(),
            negate: false,
        },
        Condition {
            request_name: "auth".to_string(),
            condition_type: ConditionType::BodyJsonPath("$.token".to_string()),
            expected_value: "valid".to_string(),
            negate: true,
        },
    ];

    let context = RequestContext {
        name: "conditional".to_string(),
        request,
        result: Some(sample_result(200, true, 100)),
    };

    let file_results = HttpFileResults {
        filename: "test.http".to_string(),
        success_count: 1,
        failed_count: 0,
        skipped_count: 0,
        result_contexts: vec![context],
    };

    let results = ProcessorResults {
        success: true,
        files: vec![file_results],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("**Conditions:**"));
    assert!(content.contains("- @if `login.response.Status` == `200`"));
    assert!(content.contains("- @if-not `auth.response.BodyJsonPath"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_handles_multiple_files() {
    let file1 = HttpFileResults {
        filename: "test1.http".to_string(),
        success_count: 2,
        failed_count: 0,
        skipped_count: 0,
        result_contexts: vec![
            RequestContext {
                name: "req1".to_string(),
                request: sample_request("req1", "GET", "https://example.com/1"),
                result: Some(sample_result(200, true, 100)),
            },
            RequestContext {
                name: "req2".to_string(),
                request: sample_request("req2", "GET", "https://example.com/2"),
                result: Some(sample_result(200, true, 150)),
            },
        ],
    };

    let file2 = HttpFileResults {
        filename: "test2.http".to_string(),
        success_count: 1,
        failed_count: 1,
        skipped_count: 0,
        result_contexts: vec![
            RequestContext {
                name: "req3".to_string(),
                request: sample_request("req3", "GET", "https://example.com/3"),
                result: Some(sample_result(200, true, 100)),
            },
            RequestContext {
                name: "req4".to_string(),
                request: sample_request("req4", "GET", "https://example.com/4"),
                result: Some(sample_result(500, false, 100)),
            },
        ],
    };

    let results = ProcessorResults {
        success: false,
        files: vec![file1, file2],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("- **Total Requests:** 4"));
    assert!(content.contains("- **Passed:** ✅ 3"));
    assert!(content.contains("- **Failed:** ❌ 1"));
    assert!(content.contains("## File: `test1.http`"));
    assert!(content.contains("## File: `test2.http`"));
    assert!(content.contains("### Request: req1"));
    assert!(content.contains("### Request: req4"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_handles_error_messages() {
    let mut result = sample_result(500, false, 100);
    result.error_message = Some("Internal Server Error|Timeout".to_string());

    let context = RequestContext {
        name: "failed".to_string(),
        request: sample_request("failed", "GET", "https://example.com"),
        result: Some(result),
    };

    let file_results = HttpFileResults {
        filename: "test.http".to_string(),
        success_count: 0,
        failed_count: 1,
        skipped_count: 0,
        result_contexts: vec![context],
    };

    let results = ProcessorResults {
        success: false,
        files: vec![file_results],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("- **Status:** ❌ 500"));
    assert!(content.contains("- **Error:** Internal Server Error\\|Timeout"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_handles_all_assertion_types() {
    let assertions = vec![
        AssertionResult {
            assertion: Assertion {
                assertion_type: AssertionType::Status,
                expected_value: "200".to_string(),
            },
            passed: true,
            actual_value: Some("200".to_string()),
            error_message: None,
        },
        AssertionResult {
            assertion: Assertion {
                assertion_type: AssertionType::Body,
                expected_value: "success".to_string(),
            },
            passed: true,
            actual_value: Some("success".to_string()),
            error_message: None,
        },
        AssertionResult {
            assertion: Assertion {
                assertion_type: AssertionType::Headers,
                expected_value: "Content-Type: application/json".to_string(),
            },
            passed: true,
            actual_value: Some("application/json".to_string()),
            error_message: None,
        },
    ];

    let mut result = sample_result(200, true, 100);
    result.assertion_results = assertions;

    let context = RequestContext {
        name: "test".to_string(),
        request: sample_request("test", "GET", "https://example.com"),
        result: Some(result),
    };

    let file_results = HttpFileResults {
        filename: "test.http".to_string(),
        success_count: 1,
        failed_count: 0,
        skipped_count: 0,
        result_contexts: vec![context],
    };

    let results = ProcessorResults {
        success: true,
        files: vec![file_results],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("| Status Code |"));
    assert!(content.contains("| Response Body |"));
    assert!(content.contains("| Response Headers |"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_includes_file_stats() {
    let file_results = HttpFileResults {
        filename: "api-tests.http".to_string(),
        success_count: 5,
        failed_count: 2,
        skipped_count: 1,
        result_contexts: vec![],
    };

    let results = ProcessorResults {
        success: false,
        files: vec![file_results],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("- **Passed:** 5 | **Failed:** 2 | **Skipped:** 1"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_includes_timestamp() {
    let results = ProcessorResults {
        success: true,
        files: vec![],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("**Generated:**"));
    // Should contain a timestamp in format YYYY-MM-DD HH:MM:SS
    assert!(content.contains("-"));
    assert!(content.contains(":"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_formats_method_correctly() {
    let methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH"];

    for method in methods {
        let context = RequestContext {
            name: format!("test_{}", method),
            request: sample_request(&format!("test_{}", method), method, "https://example.com"),
            result: Some(sample_result(200, true, 100)),
        };

        let file_results = HttpFileResults {
            filename: "test.http".to_string(),
            success_count: 1,
            failed_count: 0,
            skipped_count: 0,
            result_contexts: vec![context],
        };

        let results = ProcessorResults {
            success: true,
            files: vec![file_results],
        };

        let filename = generate_markdown(&results).unwrap();
        let content = fs::read_to_string(&filename).unwrap();

        assert!(content.contains(&format!("- **Method:** `{}`", method)));

        fs::remove_file(filename).ok();
    }
}

#[test]
fn generate_markdown_handles_special_chars_in_filename() {
    let file_results = HttpFileResults {
        filename: "test|file|name.http".to_string(),
        success_count: 0,
        failed_count: 0,
        skipped_count: 0,
        result_contexts: vec![],
    };

    let results = ProcessorResults {
        success: true,
        files: vec![file_results],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("## File: `test\\|file\\|name.http`"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_handles_empty_response_body() {
    let mut result = sample_result(204, true, 50);
    result.response_body = Some("".to_string());

    let context = RequestContext {
        name: "no_content".to_string(),
        request: sample_request("no_content", "DELETE", "https://example.com"),
        result: Some(result),
    };

    let file_results = HttpFileResults {
        filename: "test.http".to_string(),
        success_count: 1,
        failed_count: 0,
        skipped_count: 0,
        result_contexts: vec![context],
    };

    let results = ProcessorResults {
        success: true,
        files: vec![file_results],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("**Response Body:**"));
    assert!(content.contains("```\n\n```"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_markdown_handles_large_numbers() {
    let mut request = sample_request("slow", "GET", "https://example.com");
    request.timeout = Some(999999);
    request.connection_timeout = Some(888888);

    let result = sample_result(200, true, 123456);

    let context = RequestContext {
        name: "slow".to_string(),
        request,
        result: Some(result),
    };

    let file_results = HttpFileResults {
        filename: "test.http".to_string(),
        success_count: 1,
        failed_count: 0,
        skipped_count: 0,
        result_contexts: vec![context],
    };

    let results = ProcessorResults {
        success: true,
        files: vec![file_results],
    };

    let filename = generate_markdown(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("- **Timeout:** 999999ms"));
    assert!(content.contains("- **Connection Timeout:** 888888ms"));
    assert!(content.contains("- **Duration:** 123456ms"));

    fs::remove_file(filename).ok();
}
