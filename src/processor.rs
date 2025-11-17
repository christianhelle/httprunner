use crate::colors;
use crate::conditions;
use crate::log::Log;
use crate::parser;
use crate::request_variables;
use crate::runner;
use crate::types::{AssertionType, HttpRequest, RequestContext};
use anyhow::Result;

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
/// verification is skipped for HTTP requests.
///
/// # Returns
///
/// `true` if no request failures were encountered across all processed files, `false` otherwise.
///
/// # Examples
///
/// ```
/// let ok = process_http_files(&[], false, None, None, false).unwrap();
/// assert!(ok);
/// ```
pub fn process_http_files(
    files: &[String],
    verbose: bool,
    log_filename: Option<&str>,
    environment: Option<&str>,
    insecure: bool,
) -> Result<bool> {
    let mut log = Log::new(log_filename)?;

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
                    "{} {} {} {} - Skipped: dependency '{}' not met (must return HTTP 200)",
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
                    log.writeln(&format!("Body:\n{}", body));
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
                    log.writeln(&format!("Body:\n{}", body));
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

    Ok(total_failed_count == 0)
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