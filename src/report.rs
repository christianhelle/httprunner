use crate::types::{AssertionType, ProcessorResults};
use chrono::Local;
use std::fs;
use std::io::Write;

fn escape_markdown(s: &str) -> String {
    s.replace('|', "\\|")
}

pub fn generate_markdown(results: ProcessorResults) -> Result<String, std::io::Error> {
    let timestamp = Local::now().format("%Y%m%d-%H%M%S-%f");
    let filename = format!("httprunner-report-{}.md", timestamp);

    let mut report = String::new();

    // Header
    report.push_str("# HTTP File Runner - Test Report\n\n");
    report.push_str(&format!(
        "**Generated:** {}\n\n",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    // Overall summary
    let total_success: u32 = results.files.iter().map(|f| f.success_count).sum();
    let total_failed: u32 = results.files.iter().map(|f| f.failed_count).sum();
    let total_skipped: u32 = results.files.iter().map(|f| f.skipped_count).sum();
    let total_requests = total_success + total_failed + total_skipped;

    report.push_str("## Overall Summary\n\n");
    report.push_str(&format!("- **Total Requests:** {}\n", total_requests));
    report.push_str(&format!("- **Passed:** ✅ {}\n", total_success));
    report.push_str(&format!("- **Failed:** ❌ {}\n", total_failed));
    report.push_str(&format!("- **Skipped:** ⏭️ {}\n", total_skipped));
    report.push_str(&format!(
        "- **Success Rate:** {:.1}%\n\n",
        if total_requests > 0 {
            (total_success as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        }
    ));

    // Process each file
    for file_results in &results.files {
        report.push_str("---\n\n");
        report.push_str(&format!(
            "## File: `{}`\n\n",
            escape_markdown(&file_results.filename)
        ));

        report.push_str(&format!(
            "- **Passed:** {} | **Failed:** {} | **Skipped:** {}\n\n",
            file_results.success_count, file_results.failed_count, file_results.skipped_count
        ));

        // Process each request
        for context in &file_results.result_contexts {
            report.push_str(&format!(
                "### Request: {}\n\n",
                escape_markdown(&context.name)
            ));

            // Request details
            report.push_str("#### Request Details\n\n");
            report.push_str(&format!("- **Method:** `{}`\n", context.request.method));
            report.push_str(&format!(
                "- **URL:** `{}`\n",
                escape_markdown(&context.request.url)
            ));

            if let Some(timeout) = context.request.timeout {
                report.push_str(&format!("- **Timeout:** {}ms\n", timeout));
            }
            if let Some(conn_timeout) = context.request.connection_timeout {
                report.push_str(&format!("- **Connection Timeout:** {}ms\n", conn_timeout));
            }
            if let Some(ref depends_on) = context.request.depends_on {
                report.push_str(&format!(
                    "- **Depends On:** `{}`\n",
                    escape_markdown(depends_on)
                ));
            }

            // Headers
            if !context.request.headers.is_empty() {
                report.push_str("\n**Headers:**\n\n");
                report.push_str("| Header | Value |\n");
                report.push_str("|--------|-------|\n");
                for header in &context.request.headers {
                    report.push_str(&format!(
                        "| {} | {} |\n",
                        escape_markdown(&header.name),
                        escape_markdown(&header.value)
                    ));
                }
                report.push('\n');
            }

            // Request body
            if let Some(ref body) = context.request.body {
                report.push_str("**Request Body:**\n\n");
                report.push_str("```\n");
                report.push_str(body);
                report.push_str("\n```\n\n");
            }

            // Conditions
            if !context.request.conditions.is_empty() {
                report.push_str("**Conditions:**\n\n");
                for condition in &context.request.conditions {
                    let directive = if condition.negate { "@if-not" } else { "@if" };
                    let cond_type = format!("{:?}", condition.condition_type);
                    report.push_str(&format!(
                        "- {} `{}.response.{}` == `{}`\n",
                        directive,
                        escape_markdown(&condition.request_name),
                        cond_type,
                        escape_markdown(&condition.expected_value)
                    ));
                }
                report.push('\n');
            }

            // Result details
            if let Some(ref result) = context.result {
                report.push_str("#### Response Details\n\n");

                let status_icon = if result.success { "✅" } else { "❌" };
                report.push_str(&format!(
                    "- **Status:** {} {}\n",
                    status_icon, result.status_code
                ));
                report.push_str(&format!("- **Duration:** {}ms\n", result.duration_ms));

                if let Some(ref error_msg) = result.error_message {
                    report.push_str(&format!("- **Error:** {}\n", escape_markdown(error_msg)));
                }

                // Response headers
                if let Some(ref headers) = result.response_headers
                    && !headers.is_empty()
                {
                    report.push_str("\n**Response Headers:**\n\n");
                    report.push_str("| Header | Value |\n");
                    report.push_str("|--------|-------|\n");
                    for (name, value) in headers {
                        report.push_str(&format!(
                            "| {} | {} |\n",
                            escape_markdown(name),
                            escape_markdown(value)
                        ));
                    }
                    report.push('\n');
                }

                // Response body
                if let Some(ref body) = result.response_body {
                    report.push_str("**Response Body:**\n\n");
                    report.push_str("```\n");
                    report.push_str(body);
                    report.push_str("\n```\n\n");
                }

                // Assertions
                if !result.assertion_results.is_empty() {
                    report.push_str("#### Assertion Results\n\n");
                    report.push_str("| Type | Expected | Actual | Result |\n");
                    report.push_str("|------|----------|--------|--------|\n");

                    for assertion_result in &result.assertion_results {
                        let assertion_type_str = match assertion_result.assertion.assertion_type {
                            AssertionType::Status => "Status Code",
                            AssertionType::Body => "Response Body",
                            AssertionType::Headers => "Response Headers",
                        };

                        let result_icon = if assertion_result.passed {
                            "✅"
                        } else {
                            "❌"
                        };
                        let actual_val = assertion_result
                            .actual_value
                            .as_ref()
                            .map(|v| escape_markdown(v))
                            .unwrap_or_else(|| "N/A".to_string());

                        report.push_str(&format!(
                            "| {} | {} | {} | {} |\n",
                            assertion_type_str,
                            escape_markdown(&assertion_result.assertion.expected_value),
                            actual_val,
                            result_icon
                        ));
                    }
                    report.push('\n');
                }
            } else {
                report.push_str("#### Response Details\n\n");
                report.push_str("⏭️ **Request was skipped**\n\n");
            }
        }
    }

    // Write to file
    let mut file = fs::File::create(&filename)?;
    file.write_all(report.as_bytes())?;

    Ok(filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        Assertion, AssertionResult, AssertionType, Condition, ConditionType, Header,
        HttpFileResults, HttpRequest, HttpResult, RequestContext,
    };
    use std::collections::HashMap;

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
    fn escape_markdown_escapes_pipe_character() {
        assert_eq!(escape_markdown("hello|world"), "hello\\|world");
        assert_eq!(escape_markdown("no pipes here"), "no pipes here");
        assert_eq!(escape_markdown("|||"), "\\|\\|\\|");
    }

    #[test]
    fn generate_markdown_creates_file() {
        let results = ProcessorResults {
            success: true,
            files: vec![],
        };

        let filename = generate_markdown(results).unwrap();
        assert!(filename.starts_with("httprunner-report-"));
        assert!(filename.ends_with(".md"));

        // Verify file exists
        assert!(std::path::Path::new(&filename).exists());

        // Clean up
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

        let filename = generate_markdown(results).unwrap();
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

        let filename = generate_markdown(results).unwrap();
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

        let filename = generate_markdown(results).unwrap();
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

        let filename = generate_markdown(results).unwrap();
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

        let filename = generate_markdown(results).unwrap();
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

        let filename = generate_markdown(results).unwrap();
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

        let filename = generate_markdown(results).unwrap();
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

        let filename = generate_markdown(results).unwrap();
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

        let filename = generate_markdown(results).unwrap();
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

        let filename = generate_markdown(results).unwrap();
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

        let filename = generate_markdown(results).unwrap();
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

        let filename = generate_markdown(results).unwrap();
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

        let filename = generate_markdown(results).unwrap();
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

        let filename = generate_markdown(results).unwrap();
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

        let filename = generate_markdown(results).unwrap();
        let content = fs::read_to_string(&filename).unwrap();

        assert!(content.contains("| Status Code |"));
        assert!(content.contains("| Response Body |"));
        assert!(content.contains("| Response Headers |"));

        fs::remove_file(filename).ok();
    }
}
