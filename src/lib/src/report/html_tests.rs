use super::*;
use crate::types::{
    Assertion, AssertionResult, AssertionType, Condition, ConditionType, Header, HttpFileResults,
    HttpRequest, HttpResult, ProcessorResults, RequestContext,
};
use std::collections::HashMap;
use std::fs;

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
fn generate_html_creates_file() {
    let results = ProcessorResults {
        success: true,
        files: vec![],
    };

    let filename = generate_html(&results).unwrap();
    assert!(filename.starts_with("httprunner-report-"));
    assert!(filename.ends_with(".html"));

    assert!(std::path::Path::new(&filename).exists());

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_has_proper_structure() {
    let results = ProcessorResults {
        success: true,
        files: vec![],
    };

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("<!DOCTYPE html>"));
    assert!(content.contains("<html lang=\"en\">"));
    assert!(content.contains("<head>"));
    assert!(content.contains("<meta charset=\"UTF-8\">"));
    assert!(content.contains("<title>HTTP File Runner - Test Report</title>"));
    assert!(content.contains("<style>"));
    assert!(content.contains("</style>"));
    assert!(content.contains("<body>"));
    assert!(content.contains("</body>"));
    assert!(content.contains("</html>"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_includes_css_styles() {
    let results = ProcessorResults {
        success: true,
        files: vec![],
    };

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains(":root"));
    assert!(content.contains("--primary-color"));
    assert!(content.contains("--success-color"));
    assert!(content.contains("--error-color"));
    assert!(content.contains("prefers-color-scheme: dark"));
    assert!(content.contains(".container"));
    assert!(content.contains(".stats-grid"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_includes_overall_summary() {
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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("<h1>🚀 HTTP File Runner - Test Report</h1>"));
    assert!(content.contains("<h2>Overall Summary</h2>"));
    assert!(content.contains("Total Requests"));
    assert!(content.contains(">1<"));
    assert!(content.contains("Passed"));
    assert!(content.contains("✅"));
    assert!(content.contains("Failed"));
    assert!(content.contains("Skipped"));
    assert!(content.contains("Success Rate"));
    assert!(content.contains("100.0%"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_calculates_success_rate_correctly() {
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
        RequestContext {
            name: "req3".to_string(),
            request: sample_request("req3", "GET", "https://example.com"),
            result: Some(sample_result(500, false, 100)),
        },
    ];

    let file_results = HttpFileResults {
        filename: "test.http".to_string(),
        success_count: 1,
        failed_count: 2,
        skipped_count: 0,
        result_contexts: contexts,
    };

    let results = ProcessorResults {
        success: false,
        files: vec![file_results],
    };

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains(">3<"));
    assert!(content.contains("33.3%"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_handles_zero_requests() {
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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains(">0<"));
    assert!(content.contains("0.0%"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_escapes_special_characters() {
    let mut request = sample_request("test<script>", "GET", "https://example.com?foo=bar&baz=qux");
    request.body = Some("<html><body>test</body></html>".to_string());

    let context = RequestContext {
        name: "test<script>".to_string(),
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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("test&lt;script&gt;"));
    assert!(content.contains("https://example.com?foo=bar&amp;baz=qux"));
    assert!(content.contains("&lt;html&gt;&lt;body&gt;test&lt;/body&gt;&lt;/html&gt;"));

    assert!(!content.contains("test<script>") || content.contains("&lt;script&gt;"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_includes_request_headers() {
    let mut request = sample_request("test", "POST", "https://api.example.com");
    request.headers = vec![
        Header {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
        Header {
            name: "Authorization".to_string(),
            value: "Bearer token<secret>".to_string(),
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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("<h5>Headers</h5>"));
    assert!(content.contains("<table class=\"data-table\">"));
    assert!(content.contains("Content-Type"));
    assert!(content.contains("application/json"));
    assert!(content.contains("Authorization"));
    assert!(content.contains("Bearer token&lt;secret&gt;"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_includes_request_body() {
    let mut request = sample_request("test", "POST", "https://api.example.com");
    request.body = Some(r#"{"name":"John","age":30,"email":"john@example.com"}"#.to_string());

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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("<h5>Request Body</h5>"));
    assert!(content.contains("<pre class=\"code-block\"><code>"));
    // JSON will be HTML-escaped
    assert!(content.contains(r#"{&quot;name&quot;:&quot;John&quot;,&quot;age&quot;:30,&quot;email&quot;:&quot;john@example.com&quot;}"#));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_includes_response_details() {
    let mut result = sample_result(200, true, 250);

    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("Server".to_string(), "nginx/1.18".to_string());
    result.response_headers = Some(headers);
    result.response_body = Some(r#"{"status":"ok","data":{"id":123}}"#.to_string());

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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("<h4>Response Details</h4>"));
    assert!(content.contains("Status:"));
    assert!(content.contains("✅"));
    assert!(content.contains("200"));
    assert!(content.contains("Duration:"));
    assert!(content.contains("250ms"));
    assert!(content.contains("<h5>Response Headers</h5>"));
    assert!(content.contains("Content-Type"));
    assert!(content.contains("application/json"));
    assert!(content.contains("<h5>Response Body</h5>"));
    // JSON will be HTML-escaped
    assert!(
        content.contains(
            r#"{&quot;status&quot;:&quot;ok&quot;,&quot;data&quot;:{&quot;id&quot;:123}}"#
        )
    );

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_includes_assertions() {
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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("<h4>Assertion Results</h4>"));
    assert!(content.contains("<table class=\"data-table\">"));
    assert!(content.contains("Status Code"));
    assert!(content.contains(">200<"));
    assert!(content.contains("✅"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_includes_failed_assertions() {
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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("Response Body"));
    assert!(content.contains("success"));
    assert!(content.contains("error"));
    assert!(content.contains("❌"));
    assert!(content.contains("class=\"failed\""));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_handles_skipped_requests() {
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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("⏭️ Request was skipped"));
    assert!(content.contains("class=\"skipped\""));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_includes_timeouts() {
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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("Timeout:"));
    assert!(content.contains("5000ms"));
    assert!(content.contains("Connection Timeout:"));
    assert!(content.contains("3000ms"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_includes_dependencies() {
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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("Depends On:"));
    assert!(content.contains("<code>login</code>"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_includes_conditions() {
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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("<h5>Conditions</h5>"));
    assert!(content.contains("@if"));
    assert!(content.contains("@if-not"));
    assert!(content.contains("login.response.Status"));
    assert!(content.contains("auth.response.BodyJsonPath"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_handles_multiple_files() {
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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    // Check for counts - need to be more flexible with HTML formatting
    assert!(content.contains(">4<") || content.contains("4</div>"));
    assert!(content.contains("✅"));
    assert!(content.contains("❌"));
    assert!(content.contains("📄 File: <code>test1.http</code>"));
    assert!(content.contains("📄 File: <code>test2.http</code>"));
    assert!(content.contains("req1"));
    assert!(content.contains("req4"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_handles_error_messages() {
    let mut result = sample_result(500, false, 100);
    result.error_message = Some("Internal Server Error: Database timeout".to_string());

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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("Status:"));
    assert!(content.contains("❌"));
    assert!(content.contains("500"));
    assert!(content.contains("Error:"));
    assert!(content.contains("Internal Server Error: Database timeout"));
    assert!(content.contains("class=\"error\""));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_handles_all_assertion_types() {
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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("Status Code"));
    assert!(content.contains("Response Body"));
    assert!(content.contains("Response Headers"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_includes_file_stats() {
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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("Passed: 5"));
    assert!(content.contains("Failed: 2"));
    assert!(content.contains("Skipped: 1"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_has_responsive_meta_tag() {
    let results = ProcessorResults {
        success: true,
        files: vec![],
    };

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(
        content
            .contains("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")
    );

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_includes_timestamp() {
    let results = ProcessorResults {
        success: true,
        files: vec![],
    };

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("Generated:"));
    // Should contain a timestamp in format YYYY-MM-DD HH:MM:SS
    assert!(content.contains("-"));
    assert!(content.contains(":"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_formats_method_correctly() {
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

        let filename = generate_html(&results).unwrap();
        let content = fs::read_to_string(&filename).unwrap();

        assert!(content.contains(&format!("<strong>Method:</strong> <code>{}</code>", method)));

        fs::remove_file(filename).ok();
    }
}

#[test]
fn generate_html_handles_special_chars_in_filename() {
    let file_results = HttpFileResults {
        filename: "test<file>name.http".to_string(),
        success_count: 0,
        failed_count: 0,
        skipped_count: 0,
        result_contexts: vec![],
    };

    let results = ProcessorResults {
        success: true,
        files: vec![file_results],
    };

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("📄 File: <code>test&lt;file&gt;name.http</code>"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_handles_empty_response_body() {
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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("<h5>Response Body</h5>"));
    assert!(content.contains("<pre class=\"code-block\"><code></code></pre>"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_handles_large_numbers() {
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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("999999ms"));
    assert!(content.contains("888888ms"));
    assert!(content.contains("123456ms"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_has_stat_cards() {
    let results = ProcessorResults {
        success: true,
        files: vec![],
    };

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("<div class=\"stats-grid\">"));
    assert!(content.contains("<div class=\"stat-card\">"));
    assert!(content.contains("<div class=\"stat-label\">"));
    assert!(content.contains("<div class=\"stat-value\">"));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_uses_semantic_classes() {
    let result_success = sample_result(200, true, 100);
    let result_failed = sample_result(500, false, 100);

    let contexts = vec![
        RequestContext {
            name: "success".to_string(),
            request: sample_request("success", "GET", "https://example.com"),
            result: Some(result_success),
        },
        RequestContext {
            name: "failed".to_string(),
            request: sample_request("failed", "GET", "https://example.com"),
            result: Some(result_failed),
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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    // Check for status class in status span
    assert!(content.contains("class=\"status success\"") || content.contains("class=\"success\""));
    assert!(content.contains("class=\"status failed\"") || content.contains("class=\"failed\""));

    fs::remove_file(filename).ok();
}

#[test]
fn generate_html_handles_quotes_in_values() {
    let mut request = sample_request("test", "POST", "https://example.com");
    request.headers = vec![Header {
        name: "X-Custom".to_string(),
        value: r#"value with "quotes" and 'apostrophes'"#.to_string(),
    }];

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

    let filename = generate_html(&results).unwrap();
    let content = fs::read_to_string(&filename).unwrap();

    assert!(content.contains("&quot;quotes&quot;"));
    assert!(content.contains("&#39;apostrophes&#39;"));

    fs::remove_file(filename).ok();
}
