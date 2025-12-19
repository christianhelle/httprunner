use crate::colors;
use crate::conditions;
use crate::log::Log;
use crate::parser;
use crate::request_variables;
use crate::runner;
use crate::types::HttpFileResults;
use crate::types::{AssertionType, HttpRequest, ProcessorResults, RequestContext};
use anyhow::Result;

/// Attempts to parse and pretty-print JSON. Returns the formatted JSON on success,
/// or the original string if it's not valid JSON.
fn format_json_if_valid(text: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(json) => match serde_json::to_string_pretty(&json) {
            Ok(pretty) => pretty,
            Err(_) => text.to_string(),
        },
        Err(_) => text.to_string(),
    }
}

/// Helper function to format request name for logging
fn format_request_name(name: &Option<String>) -> String {
    name.as_ref()
        .map(|n| format!("{}: ", n))
        .unwrap_or_default()
}

/// Helper function to add a skipped request to the context
fn add_skipped_request_context(
    request_contexts: &mut Vec<RequestContext>,
    processed_request: HttpRequest,
    request_count: u32,
) {
    let context_name = processed_request
        .name
        .clone()
        .unwrap_or_else(|| format!("request_{}", request_count));

    request_contexts.push(RequestContext {
        name: context_name,
        request: processed_request,
        result: None,
    });
}

/// Process one or more HTTP files, executing their requests and logging results.
///
/// This function parses each provided HTTP file, evaluates request dependencies and conditions,
/// substitutes request variables from prior results, executes requests, evaluates assertions,
/// and records per-request contexts and summary logs. If `insecure` is `true`, TLS certificate
/// verification is skipped for HTTP requests. If `pretty_json` is `true`, JSON payloads in
/// verbose output will be formatted for better readability.
///
/// # Returns
///
/// `true` if no request failures were encountered across all processed files, `false` otherwise.
///
/// # Examples
///
/// ```
/// let ok = process_http_files(&[], false, None, None, false, false).unwrap();
/// assert!(ok);
/// ```
pub fn process_http_files(
    files: &[String],
    verbose: bool,
    log_filename: Option<&str>,
    environment: Option<&str>,
    insecure: bool,
    pretty_json: bool,
) -> Result<ProcessorResults> {
    let mut log = Log::new(log_filename)?;
    let mut http_file_results = Vec::<HttpFileResults>::new();

    let mut total_success_count = 0u32;
    let mut total_failed_count = 0u32;
    let mut total_skipped_count = 0u32;
    let mut files_processed = 0u32;

    for http_file in files {
        log.writeln(&format!(
            "{} HTTP File Runner - Processing file: {}",
            colors::blue("🚀"),
            http_file
        ));
        log.writeln(&"=".repeat(50));

        let requests = match parser::parse_http_file(http_file, environment) {
            Ok(reqs) => reqs,
            Err(e) => {
                log.writeln(&format!("{} Error parsing file: {}", colors::red("❌"), e));
                continue;
            }
        };

        if requests.is_empty() {
            log.writeln(&format!(
                "{} No HTTP requests found in file",
                colors::yellow("⚠️")
            ));
            continue;
        }

        log.writeln(&format!("Found {} HTTP request(s)\n", requests.len()));
        files_processed += 1;

        let mut success_count = 0u32;
        let mut failed_count = 0u32;
        let mut skipped_count = 0u32;
        let mut request_count = 0u32;
        let mut request_contexts: Vec<RequestContext> = Vec::new();

        for request in requests {
            request_count += 1;

            // Clone request for processing
            let mut processed_request = request.clone();

            // Check if dependencies are met
            if let Some(dep_name) = processed_request.depends_on.as_ref()
                && !conditions::check_dependency(&Some(dep_name.clone()), &request_contexts)
            {
                let name_str = format_request_name(&processed_request.name);

                log.writeln(&format!(
                    "{} {} {} {} - Skipped: dependency '{}' not met (must succeed with HTTP 2xx)",
                    colors::yellow("⏭️"),
                    name_str,
                    processed_request.method,
                    processed_request.url,
                    dep_name
                ));

                add_skipped_request_context(
                    &mut request_contexts,
                    processed_request,
                    request_count,
                );
                skipped_count += 1;
                continue;
            }

            // Check if conditions are met
            if !processed_request.conditions.is_empty() {
                if verbose {
                    // Use verbose evaluation to get detailed results
                    match conditions::evaluate_conditions_verbose(
                        &processed_request.conditions,
                        &request_contexts,
                    ) {
                        Ok((conditions_met, evaluation_results)) => {
                            // Log condition evaluation details in assertion-like format
                            log.writeln(&format!("\n{} Condition Evaluation:", colors::blue("🔍")));

                            for (condition, eval_result) in processed_request
                                .conditions
                                .iter()
                                .zip(evaluation_results.iter())
                            {
                                let directive = if eval_result.negated {
                                    "@if-not"
                                } else {
                                    "@if"
                                };
                                let request_ref = if condition.request_name.is_empty() {
                                    "<unnamed>"
                                } else {
                                    condition.request_name.as_str()
                                };

                                if eval_result.condition_met {
                                    log.writeln(&format!(
                                        "{}   ✅ {}: {}.response.{}",
                                        colors::green(""),
                                        directive,
                                        request_ref,
                                        eval_result.condition_type
                                    ));
                                    log.writeln(&format!(
                                        "{}      Expected: {} \"{}\"",
                                        colors::green(""),
                                        if eval_result.negated { "!=" } else { "==" },
                                        eval_result.expected_value
                                    ));
                                    log.writeln(&format!(
                                        "{}      Actual: \"{}\"",
                                        colors::green(""),
                                        eval_result.actual_value.as_deref().unwrap_or("<unknown>")
                                    ));
                                } else {
                                    log.writeln(&format!(
                                        "{}   ❌ {}: {}.response.{}",
                                        colors::red(""),
                                        directive,
                                        request_ref,
                                        eval_result.condition_type
                                    ));
                                    log.writeln(&format!(
                                        "{}      Expected: {} \"{}\"",
                                        colors::yellow(""),
                                        if eval_result.negated { "!=" } else { "==" },
                                        eval_result.expected_value
                                    ));
                                    log.writeln(&format!(
                                        "{}      Actual: \"{}\"",
                                        colors::yellow(""),
                                        eval_result.actual_value.as_deref().unwrap_or("<unknown>")
                                    ));
                                }
                            }

                            if !conditions_met {
                                let name_str = format_request_name(&processed_request.name);
                                log.writeln(&format!(
                                    "\n{} {} {} {} - Skipped: conditions not met\n",
                                    colors::yellow("⏭️"),
                                    name_str,
                                    processed_request.method,
                                    processed_request.url
                                ));

                                add_skipped_request_context(
                                    &mut request_contexts,
                                    processed_request,
                                    request_count,
                                );
                                skipped_count += 1;
                                continue;
                            }
                            log.writeln(""); // Empty line for readability
                        }
                        Err(e) => {
                            let name_str = format_request_name(&processed_request.name);
                            log.writeln(&format!(
                                "{} {} {} {} - Error evaluating conditions: {}\n",
                                colors::red("❌"),
                                name_str,
                                processed_request.method,
                                processed_request.url,
                                e
                            ));

                            add_skipped_request_context(
                                &mut request_contexts,
                                processed_request,
                                request_count,
                            );
                            skipped_count += 1;
                            continue;
                        }
                    }
                } else {
                    // Non-verbose mode: simple evaluation
                    match conditions::evaluate_conditions(
                        &processed_request.conditions,
                        &request_contexts,
                    ) {
                        Ok(conditions_met) => {
                            if !conditions_met {
                                let name_str = format_request_name(&processed_request.name);

                                log.writeln(&format!(
                                    "{} {} {} {} - Skipped: conditions not met",
                                    colors::yellow("⏭️"),
                                    name_str,
                                    processed_request.method,
                                    processed_request.url
                                ));

                                add_skipped_request_context(
                                    &mut request_contexts,
                                    processed_request,
                                    request_count,
                                );
                                skipped_count += 1;
                                continue;
                            }
                        }
                        Err(e) => {
                            let name_str = format_request_name(&processed_request.name);

                            log.writeln(&format!(
                                "{} {} {} {} - Error evaluating conditions: {}",
                                colors::red("❌"),
                                name_str,
                                processed_request.method,
                                processed_request.url,
                                e
                            ));

                            add_skipped_request_context(
                                &mut request_contexts,
                                processed_request,
                                request_count,
                            );
                            skipped_count += 1;
                            continue;
                        }
                    }
                }
            }

            // Substitute request variables
            substitute_request_variables_in_request(&mut processed_request, &request_contexts)?;

            if verbose {
                log.writeln(&format!("\n{} Request Details:", colors::blue("📤")));
                if let Some(ref name) = processed_request.name {
                    log.writeln(&format!("Name: {}", name));
                }
                log.writeln(&format!("Method: {}", processed_request.method));
                log.writeln(&format!("URL: {}", processed_request.url));

                if !processed_request.headers.is_empty() {
                    log.writeln("Headers:");
                    for header in &processed_request.headers {
                        log.writeln(&format!("  {}: {}", header.name, header.value));
                    }
                }

                if let Some(ref body) = processed_request.body {
                    let formatted_body = if pretty_json {
                        format_json_if_valid(body)
                    } else {
                        body.clone()
                    };
                    log.writeln(&format!("Body:\n{}", formatted_body));
                }
                log.writeln(&"-".repeat(30));
            }

            let result = match runner::execute_http_request(&processed_request, verbose, insecure) {
                Ok(res) => res,
                Err(e) => {
                    log.writeln(&format!(
                        "{} {} {} - Error: {}",
                        colors::red("❌"),
                        processed_request.method,
                        processed_request.url,
                        e
                    ));

                    let context_name = processed_request
                        .name
                        .clone()
                        .unwrap_or_else(|| format!("request_{}", request_count));

                    request_contexts.push(RequestContext {
                        name: context_name,
                        request: processed_request,
                        result: None,
                    });
                    failed_count += 1;
                    continue;
                }
            };

            if result.success {
                success_count += 1;
                let name_prefix = result
                    .request_name
                    .as_ref()
                    .map(|n| format!("{}: ", n))
                    .unwrap_or_default();

                log.writeln(&format!(
                    "{} {} {} {} - Status: {} - {}ms",
                    colors::green("✅"),
                    name_prefix,
                    processed_request.method,
                    processed_request.url,
                    result.status_code,
                    result.duration_ms
                ));
            } else {
                failed_count += 1;
                let name_prefix = result
                    .request_name
                    .as_ref()
                    .map(|n| format!("{}: ", n))
                    .unwrap_or_default();

                if let Some(ref msg) = result.error_message {
                    log.writeln(&format!(
                        "{} {} {} {} - Status: {} - {}ms - Error: {}",
                        colors::red("❌"),
                        name_prefix,
                        processed_request.method,
                        processed_request.url,
                        result.status_code,
                        result.duration_ms,
                        msg
                    ));
                } else {
                    log.writeln(&format!(
                        "{} {} {} {} - Status: {} - {}ms",
                        colors::red("❌"),
                        name_prefix,
                        processed_request.method,
                        processed_request.url,
                        result.status_code,
                        result.duration_ms
                    ));
                }
            }

            let context_name = processed_request
                .name
                .clone()
                .unwrap_or_else(|| format!("request_{}", request_count));

            request_contexts.push(RequestContext {
                name: context_name,
                request: processed_request.clone(),
                result: Some(result),
            });

            if verbose
                && let Some(ctx) = request_contexts.last()
                && let Some(ref result) = ctx.result
            {
                log.writeln(&format!("\n{} Response Details:", colors::blue("📥")));
                log.writeln(&format!("Status: {}", result.status_code));
                log.writeln(&format!("Duration: {}ms", result.duration_ms));

                if let Some(ref headers) = result.response_headers {
                    log.writeln("Headers:");
                    for (name, value) in headers {
                        log.writeln(&format!("  {}: {}", name, value));
                    }
                }

                if let Some(ref body) = result.response_body {
                    let formatted_body = if pretty_json {
                        format_json_if_valid(body)
                    } else {
                        body.clone()
                    };
                    log.writeln(&format!("Body:\n{}", formatted_body));
                }
                log.writeln(&"-".repeat(30));
            }

            if !processed_request.assertions.is_empty()
                && let Some(ctx) = request_contexts.last()
                && let Some(ref result) = ctx.result
            {
                log.writeln(&format!("\n{} Assertion Results:", colors::blue("🔍")));
                for assertion_result in &result.assertion_results {
                    let assertion_type_str = match assertion_result.assertion.assertion_type {
                        AssertionType::Status => "Status Code",
                        AssertionType::Body => "Response Body",
                        AssertionType::Headers => "Response Headers",
                    };

                    if assertion_result.passed {
                        log.writeln(&format!(
                            "{}   ✅ {}: Expected '{}'",
                            colors::green(""),
                            assertion_type_str,
                            assertion_result.assertion.expected_value
                        ));
                    } else {
                        log.writeln(&format!(
                            "{}   ❌ {}: {}",
                            colors::red(""),
                            assertion_type_str,
                            assertion_result
                                .error_message
                                .as_ref()
                                .unwrap_or(&"Failed".to_string())
                        ));
                        if let Some(ref actual) = assertion_result.actual_value {
                            log.writeln(&format!(
                                "{}      Expected: '{}'",
                                colors::yellow(""),
                                assertion_result.assertion.expected_value
                            ));
                            log.writeln(&format!(
                                "{}      Actual: '{}'",
                                colors::yellow(""),
                                actual
                            ));
                        }
                    }
                }
                log.writeln(&"-".repeat(30));
            }
        }

        log.writeln(&format!("\n{}", "=".repeat(50)));
        log.writeln(&format!(
            "File Summary: {}, {}, {}\n",
            colors::green(&format!("{} Passed", success_count)),
            colors::red(&format!("{} Failed", failed_count)),
            colors::yellow(&format!("{} Skipped", skipped_count))
        ));

        total_success_count += success_count;
        total_failed_count += failed_count;
        total_skipped_count += skipped_count;

        http_file_results.push(HttpFileResults {
            filename: http_file.clone(),
            success_count,
            failed_count,
            skipped_count,
            result_contexts: request_contexts,
        });
    }

    if files_processed > 1 {
        log.writeln(&format!("{} Overall Summary:", colors::blue("🎯")));
        log.writeln(&format!("Files processed: {}", files_processed));
        log.writeln(&format!(
            "Total requests: {}, {}, {}\n",
            colors::green(&format!("{} Passed", total_success_count)),
            colors::red(&format!("{} Failed", total_failed_count)),
            colors::yellow(&format!("{} Skipped", total_skipped_count))
        ));
    }

    Ok(ProcessorResults {
        success: total_failed_count == 0,
        files: http_file_results,
    })
}

fn substitute_request_variables_in_request(
    request: &mut HttpRequest,
    context: &[RequestContext],
) -> Result<()> {
    // Substitute in URL
    request.url = request_variables::substitute_request_variables(&request.url, context)?;

    // Substitute in headers
    for header in &mut request.headers {
        header.name = request_variables::substitute_request_variables(&header.name, context)?;
        header.value = request_variables::substitute_request_variables(&header.value, context)?;
    }

    // Substitute in body
    if let Some(ref body) = request.body {
        request.body = Some(request_variables::substitute_request_variables(
            body, context,
        )?);
    }

    // Substitute in assertion expected values
    for assertion in &mut request.assertions {
        assertion.expected_value =
            request_variables::substitute_request_variables(&assertion.expected_value, context)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Assertion, AssertionType, Header, HttpRequest, HttpResult, RequestContext};
    use std::collections::HashMap;

    fn sample_request(name: Option<&str>) -> HttpRequest {
        HttpRequest {
            name: name.map(|n| n.to_string()),
            method: "GET".into(),
            url: "https://example.com".into(),
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

    fn sample_context() -> RequestContext {
        let request = sample_request(Some("login"));

        let mut headers = HashMap::new();
        headers.insert("X-Auth".into(), "secret".into());

        let result = HttpResult {
            request_name: Some("login".into()),
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 20,
            response_headers: Some(headers),
            response_body: Some(r#"{"tenant":"acme","token":"abc123"}"#.into()),
            assertion_results: vec![],
        };

        RequestContext {
            name: "login".into(),
            request,
            result: Some(result),
        }
    }

    #[test]
    fn format_request_name_includes_suffix() {
        assert_eq!(format_request_name(&Some("req".into())), "req: ");
        assert_eq!(format_request_name(&None), "");
    }

    #[test]
    fn add_skipped_request_context_assigns_default_name() {
        let mut contexts = Vec::new();
        add_skipped_request_context(&mut contexts, sample_request(None), 1);
        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0].name, "request_1");
        assert!(contexts[0].result.is_none());
    }

    #[test]
    fn substitute_request_variables_updates_all_fields() {
        let context = sample_context();
        let mut request = HttpRequest {
            name: Some("use-substitution".into()),
            method: "POST".into(),
            url: "https://api/{{login.response.body.$.tenant}}/users".into(),
            headers: vec![Header {
                name: "{{login.response.body.$.tenant}}-Header".into(),
                value: "Bearer {{login.response.headers.X-Auth}}".into(),
            }],
            body: Some("{\"token\":\"{{login.response.body.$.token}}\"}".into()),
            assertions: vec![Assertion {
                assertion_type: AssertionType::Body,
                expected_value: "{{login.response.body.$.token}}".into(),
            }],
            variables: vec![],
            timeout: None,
            connection_timeout: None,
            depends_on: None,
            conditions: vec![],
        };

        substitute_request_variables_in_request(&mut request, &[context]).unwrap();

        assert_eq!(request.url, "https://api/acme/users");
        assert_eq!(request.headers[0].name, "acme-Header");
        assert_eq!(request.headers[0].value, "Bearer secret");
        assert_eq!(request.body.as_deref(), Some("{\"token\":\"abc123\"}"));
        assert_eq!(request.assertions[0].expected_value, "abc123");
    }

    #[test]
    fn format_json_if_valid_formats_valid_json() {
        let compact_json = r#"{"name":"John","age":30,"active":true}"#;
        let result = format_json_if_valid(compact_json);

        // Should be formatted with proper indentation
        assert!(result.contains("{\n"));
        assert!(result.contains("  \"name\": \"John\""));
        assert!(result.contains("  \"age\": 30"));
        assert!(result.contains("  \"active\": true"));
    }

    #[test]
    fn format_json_if_valid_handles_nested_objects() {
        let compact_json = r#"{"user":{"name":"Jane","address":{"city":"NYC"}}}"#;
        let result = format_json_if_valid(compact_json);

        // Should format nested structures
        assert!(result.contains("{\n"));
        assert!(result.contains("  \"user\": {"));
        assert!(result.contains("    \"name\": \"Jane\""));
        assert!(result.contains("    \"address\": {"));
        assert!(result.contains("      \"city\": \"NYC\""));
    }

    #[test]
    fn format_json_if_valid_handles_arrays() {
        let compact_json = r#"{"items":[1,2,3],"names":["Alice","Bob"]}"#;
        let result = format_json_if_valid(compact_json);

        // Should format arrays
        assert!(result.contains("\"items\": ["));
        assert!(result.contains("\"names\": ["));
        assert!(result.contains("\"Alice\""));
        assert!(result.contains("\"Bob\""));
    }

    #[test]
    fn format_json_if_valid_preserves_non_json() {
        let plain_text = "This is not JSON";
        let result = format_json_if_valid(plain_text);

        // Should return original text unchanged
        assert_eq!(result, plain_text);
    }

    #[test]
    fn format_json_if_valid_handles_empty_object() {
        let empty_json = "{}";
        let result = format_json_if_valid(empty_json);

        // Should handle empty objects
        assert_eq!(result, "{}");
    }

    #[test]
    fn format_json_if_valid_handles_empty_array() {
        let empty_array = "[]";
        let result = format_json_if_valid(empty_array);

        // Should handle empty arrays
        assert_eq!(result, "[]");
    }

    #[test]
    fn format_json_if_valid_handles_malformed_json() {
        let malformed = r#"{"name":"John","age":}"#;
        let result = format_json_if_valid(malformed);

        // Should return original text for malformed JSON
        assert_eq!(result, malformed);
    }

    #[test]
    fn format_json_if_valid_handles_json_with_special_chars() {
        let json_with_escapes = r#"{"message":"Line1\nLine2\tTabbed","quote":"He said \"Hello\""}"#;
        let result = format_json_if_valid(json_with_escapes);

        // Should properly format JSON with escape sequences
        assert!(result.contains("\"message\": \"Line1\\nLine2\\tTabbed\""));
        assert!(result.contains("\"quote\": \"He said \\\"Hello\\\"\""));
    }

    #[test]
    fn format_json_if_valid_handles_json_with_null() {
        let json_with_null = r#"{"name":"John","email":null}"#;
        let result = format_json_if_valid(json_with_null);

        // Should handle null values
        assert!(result.contains("\"email\": null"));
    }

    #[test]
    fn format_json_if_valid_handles_already_formatted_json() {
        let formatted_json = "{\n  \"name\": \"John\",\n  \"age\": 30\n}";
        let result = format_json_if_valid(formatted_json);

        // Should reformat already formatted JSON
        assert!(result.contains("\"name\": \"John\""));
        assert!(result.contains("\"age\": 30"));
    }

    #[test]
    fn format_json_if_valid_handles_xml() {
        let xml = "<root><item>value</item></root>";
        let result = format_json_if_valid(xml);

        // Should return XML unchanged as it's not JSON
        assert_eq!(result, xml);
    }

    #[test]
    fn format_json_if_valid_handles_html() {
        let html = "<html><body>Hello World</body></html>";
        let result = format_json_if_valid(html);

        // Should return HTML unchanged as it's not JSON
        assert_eq!(result, html);
    }
}
