use super::formatter::{format_json_if_valid, format_request_name};
use super::substitution::{
    substitute_functions_in_request, substitute_request_variables_in_request,
};
use crate::colors;
use crate::conditions;
use crate::logging::Log;
use crate::parser;
use crate::runner;
use crate::types::{
    AssertionType, HttpFileResults, HttpRequest, HttpResult, ProcessorResults, RequestContext,
};
use anyhow::Result;

/// Configuration for processing HTTP files.
pub struct ProcessorConfig<'a> {
    pub files: &'a [String],
    pub verbose: bool,
    pub log_filename: Option<&'a str>,
    pub environment: Option<&'a str>,
    pub insecure: bool,
    pub pretty_json: bool,
    pub silent: bool,
}

impl<'a> ProcessorConfig<'a> {
    pub fn new(files: &'a [String]) -> Self {
        Self {
            files,
            verbose: false,
            log_filename: None,
            environment: None,
            insecure: false,
            pretty_json: false,
            silent: false,
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn with_log_filename(mut self, log_filename: Option<&'a str>) -> Self {
        self.log_filename = log_filename;
        self
    }

    pub fn with_environment(mut self, environment: Option<&'a str>) -> Self {
        self.environment = environment;
        self
    }

    pub fn with_insecure(mut self, insecure: bool) -> Self {
        self.insecure = insecure;
        self
    }

    pub fn with_pretty_json(mut self, pretty_json: bool) -> Self {
        self.pretty_json = pretty_json;
        self
    }

    pub fn with_silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }
}

/// Counters for tracking request processing results.
struct RequestCounters {
    success: u32,
    failed: u32,
    skipped: u32,
    total: u32,
}

impl RequestCounters {
    fn new() -> Self {
        Self {
            success: 0,
            failed: 0,
            skipped: 0,
            total: 0,
        }
    }

    fn increment_total(&mut self) {
        self.total += 1;
    }

    fn record_success(&mut self) {
        self.success += 1;
    }

    fn record_failure(&mut self) {
        self.failed += 1;
    }

    fn record_skip(&mut self) {
        self.skipped += 1;
    }
}

/// Aggregate counters across multiple files.
struct TotalCounters {
    success: u32,
    failed: u32,
    skipped: u32,
    files_processed: u32,
}

impl TotalCounters {
    fn new() -> Self {
        Self {
            success: 0,
            failed: 0,
            skipped: 0,
            files_processed: 0,
        }
    }

    fn add_file_results(&mut self, counters: &RequestCounters) {
        self.success += counters.success;
        self.failed += counters.failed;
        self.skipped += counters.skipped;
        self.files_processed += 1;
    }
}

/// Generate a context name for a request.
fn get_context_name(request: &HttpRequest, request_count: u32) -> String {
    request
        .name
        .clone()
        .unwrap_or_else(|| format!("request_{}", request_count))
}

/// Add a request context for a skipped request.
fn add_skipped_request_context(
    request_contexts: &mut Vec<RequestContext>,
    processed_request: HttpRequest,
    request_count: u32,
) {
    let context_name = get_context_name(&processed_request, request_count);
    request_contexts.push(RequestContext {
        name: context_name,
        request: processed_request,
        result: None,
    });
}

/// Add a request context with a result.
fn add_request_context_with_result(
    request_contexts: &mut Vec<RequestContext>,
    processed_request: HttpRequest,
    result: Option<HttpResult>,
    request_count: u32,
) {
    let context_name = get_context_name(&processed_request, request_count);
    request_contexts.push(RequestContext {
        name: context_name,
        request: processed_request,
        result,
    });
}

/// Check if a request should be skipped due to a failed dependency.
fn should_skip_due_to_dependency(
    processed_request: &HttpRequest,
    request_contexts: &[RequestContext],
    log: &mut Log,
) -> bool {
    if let Some(dep_name) = processed_request.depends_on.as_ref()
        && !conditions::check_dependency(&Some(dep_name.clone()), request_contexts)
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

        return true;
    }
    false
}

/// Log verbose condition evaluation details.
fn log_condition_evaluation_verbose(
    processed_request: &HttpRequest,
    request_contexts: &[RequestContext],
    log: &mut Log,
) -> Result<bool> {
    let (conditions_met, evaluation_results) =
        conditions::evaluate_conditions_verbose(&processed_request.conditions, request_contexts)?;

    log.writeln(&format!("\n{} Condition Evaluation:", colors::blue("🔍")));

    for (condition, eval_result) in processed_request
        .conditions
        .iter()
        .zip(evaluation_results.iter())
    {
        log_single_condition_result(condition, eval_result, log);
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
    } else {
        log.writeln(""); // Empty line for readability
    }

    Ok(conditions_met)
}

/// Log a single condition evaluation result.
fn log_single_condition_result(
    condition: &crate::types::Condition,
    eval_result: &conditions::ConditionEvaluationResult,
    log: &mut Log,
) {
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

    let (color_fn, status_icon): (fn(&str) -> String, &str) = if eval_result.condition_met {
        (colors::green, "✅")
    } else {
        (colors::red, "❌")
    };

    log.writeln(&format!(
        "{}   {} {}: {}.response.{}",
        color_fn(""),
        status_icon,
        directive,
        request_ref,
        eval_result.condition_type
    ));

    let value_color = if eval_result.condition_met {
        colors::green
    } else {
        colors::yellow
    };

    log.writeln(&format!(
        "{}      Expected: {} \"{}\"",
        value_color(""),
        if eval_result.negated { "!=" } else { "==" },
        eval_result.expected_value
    ));
    log.writeln(&format!(
        "{}      Actual: \"{}\"",
        value_color(""),
        eval_result.actual_value.as_deref().unwrap_or("<unknown>")
    ));
}

/// Check if a request should be skipped due to unmet conditions.
fn should_skip_due_to_conditions(
    processed_request: &HttpRequest,
    request_contexts: &[RequestContext],
    log: &mut Log,
    verbose: bool,
) -> bool {
    if processed_request.conditions.is_empty() {
        return false;
    }

    if verbose {
        match log_condition_evaluation_verbose(processed_request, request_contexts, log) {
            Ok(conditions_met) => !conditions_met,
            Err(e) => {
                log_condition_error(processed_request, &e, log);
                true
            }
        }
    } else {
        match conditions::evaluate_conditions(&processed_request.conditions, request_contexts) {
            Ok(conditions_met) => {
                if !conditions_met {
                    log_conditions_not_met(processed_request, log);
                }
                !conditions_met
            }
            Err(e) => {
                log_condition_error(processed_request, &e, log);
                true
            }
        }
    }
}

/// Log that conditions were not met.
fn log_conditions_not_met(processed_request: &HttpRequest, log: &mut Log) {
    let name_str = format_request_name(&processed_request.name);
    log.writeln(&format!(
        "{} {} {} {} - Skipped: conditions not met",
        colors::yellow("⏭️"),
        name_str,
        processed_request.method,
        processed_request.url
    ));
}

/// Log a condition evaluation error.
fn log_condition_error(processed_request: &HttpRequest, error: &anyhow::Error, log: &mut Log) {
    let name_str = format_request_name(&processed_request.name);
    log.writeln(&format!(
        "{} {} {} {} - Error evaluating conditions: {}",
        colors::red("❌"),
        name_str,
        processed_request.method,
        processed_request.url,
        error
    ));
}

/// Log request details in verbose mode.
fn log_request_details(request: &HttpRequest, log: &mut Log, pretty_json: bool) {
    log.writeln(&format!("\n{} Request Details:", colors::blue("📤")));
    if let Some(ref name) = request.name {
        log.writeln(&format!("Name: {}", name));
    }
    log.writeln(&format!("Method: {}", request.method));
    log.writeln(&format!("URL: {}", request.url));

    if !request.headers.is_empty() {
        log.writeln("Headers:");
        for header in &request.headers {
            log.writeln(&format!("  {}: {}", header.name, header.value));
        }
    }

    if let Some(ref body) = request.body {
        let formatted_body = if pretty_json {
            format_json_if_valid(body)
        } else {
            body.clone()
        };
        log.writeln(&format!("Body:\n{}", formatted_body));
    }
    log.writeln(&"-".repeat(30));
}

/// Log the result of executing a request.
fn log_execution_result(result: &HttpResult, processed_request: &HttpRequest, log: &mut Log) {
    let name_prefix = result
        .request_name
        .as_ref()
        .map(|n| format!("{}: ", n))
        .unwrap_or_default();

    if result.success {
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
        let error_suffix = result
            .error_message
            .as_ref()
            .map(|msg| format!(" - Error: {}", msg))
            .unwrap_or_default();

        log.writeln(&format!(
            "{} {} {} {} - Status: {} - {}ms{}",
            colors::red("❌"),
            name_prefix,
            processed_request.method,
            processed_request.url,
            result.status_code,
            result.duration_ms,
            error_suffix
        ));
    }
}

/// Log response details in verbose mode.
fn log_response_details(result: &HttpResult, log: &mut Log, pretty_json: bool) {
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

/// Log assertion results.
fn log_assertion_results(result: &HttpResult, log: &mut Log) {
    log.writeln(&format!("\n{} Assertion Results:", colors::blue("🔍")));
    for assertion_result in &result.assertion_results {
        log_single_assertion_result(assertion_result, log);
    }
    log.writeln(&"-".repeat(30));
}

/// Log a single assertion result.
fn log_single_assertion_result(
    assertion_result: &crate::types::AssertionResult,
    log: &mut Log,
) {
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
            log.writeln(&format!("{}      Actual: '{}'", colors::yellow(""), actual));
        }
    }
}

/// Log the file header when starting to process a file.
fn log_file_header(http_file: &str, log: &mut Log) {
    log.writeln(&format!(
        "{} HTTP File Runner - Processing file: {}",
        colors::blue("🚀"),
        http_file
    ));
    log.writeln(&"=".repeat(50));
}

/// Log the file summary after processing all requests in a file.
fn log_file_summary(counters: &RequestCounters, log: &mut Log) {
    log.writeln(&format!("\n{}", "=".repeat(50)));
    log.writeln(&format!(
        "File Summary: {}, {}, {}\n",
        colors::green(&format!("{} Passed", counters.success)),
        colors::red(&format!("{} Failed", counters.failed)),
        colors::yellow(&format!("{} Skipped", counters.skipped))
    ));
}

/// Log the overall summary when processing multiple files.
fn log_overall_summary(totals: &TotalCounters, log: &mut Log) {
    if totals.files_processed > 1 {
        log.writeln(&format!("{} Overall Summary:", colors::blue("🎯")));
        log.writeln(&format!("Files processed: {}", totals.files_processed));
        log.writeln(&format!(
            "Total requests: {}, {}, {}\n",
            colors::green(&format!("{} Passed", totals.success)),
            colors::red(&format!("{} Failed", totals.failed)),
            colors::yellow(&format!("{} Skipped", totals.skipped))
        ));
    }
}

/// Log an execution error.
fn log_execution_error(processed_request: &HttpRequest, error: &anyhow::Error, log: &mut Log) {
    log.writeln(&format!(
        "{} {} {} - Error: {}",
        colors::red("❌"),
        processed_request.method,
        processed_request.url,
        error
    ));
}

/// Result of processing a single request.
enum RequestProcessResult {
    /// Request was skipped
    Skipped,
    /// Request execution failed with an error
    ExecutionError,
    /// Request completed (success or failure determined by result.success)
    Completed(HttpResult),
}

/// Process a single HTTP request.
fn process_single_request<F>(
    request: HttpRequest,
    request_contexts: &[RequestContext],
    config: &ProcessorConfig,
    executor: &F,
    log: &mut Log,
) -> Result<(RequestProcessResult, HttpRequest)>
where
    F: Fn(&HttpRequest, bool, bool) -> Result<HttpResult>,
{
    let mut processed_request = request;

    // Check dependencies
    if should_skip_due_to_dependency(&processed_request, request_contexts, log) {
        return Ok((RequestProcessResult::Skipped, processed_request));
    }

    // Check conditions
    if should_skip_due_to_conditions(&processed_request, request_contexts, log, config.verbose) {
        return Ok((RequestProcessResult::Skipped, processed_request));
    }

    // Apply substitutions
    substitute_request_variables_in_request(&mut processed_request, request_contexts)?;
    substitute_functions_in_request(&mut processed_request)?;

    // Log request details if verbose
    if config.verbose {
        log_request_details(&processed_request, log, config.pretty_json);
    }

    // Execute the request
    match executor(&processed_request, config.verbose, config.insecure) {
        Ok(result) => Ok((RequestProcessResult::Completed(result), processed_request)),
        Err(e) => {
            log_execution_error(&processed_request, &e, log);
            Ok((RequestProcessResult::ExecutionError, processed_request))
        }
    }
}

/// Process all requests in a single HTTP file.
fn process_single_file<F>(
    http_file: &str,
    config: &ProcessorConfig,
    executor: &F,
    log: &mut Log,
) -> Option<HttpFileResults>
where
    F: Fn(&HttpRequest, bool, bool) -> Result<HttpResult>,
{
    log_file_header(http_file, log);

    let requests = match parser::parse_http_file(http_file, config.environment) {
        Ok(reqs) => reqs,
        Err(e) => {
            log.writeln(&format!("{} Error parsing file: {}", colors::red("❌"), e));
            return None;
        }
    };

    if requests.is_empty() {
        log.writeln(&format!(
            "{} No HTTP requests found in file",
            colors::yellow("⚠️")
        ));
        return None;
    }

    log.writeln(&format!("Found {} HTTP request(s)\n", requests.len()));

    let mut counters = RequestCounters::new();
    let mut request_contexts: Vec<RequestContext> = Vec::new();

    for request in requests {
        counters.increment_total();

        let (result, processed_request) = match process_single_request(
            request,
            &request_contexts,
            config,
            executor,
            log,
        ) {
            Ok((result, req)) => (result, req),
            Err(e) => {
                log.writeln(&format!("{} Internal error: {}", colors::red("❌"), e));
                counters.record_failure();
                continue;
            }
        };

        match result {
            RequestProcessResult::Skipped => {
                add_skipped_request_context(
                    &mut request_contexts,
                    processed_request,
                    counters.total,
                );
                counters.record_skip();
            }
            RequestProcessResult::ExecutionError => {
                add_request_context_with_result(
                    &mut request_contexts,
                    processed_request,
                    None,
                    counters.total,
                );
                counters.record_failure();
            }
            RequestProcessResult::Completed(http_result) => {
                if http_result.success {
                    counters.record_success();
                } else {
                    counters.record_failure();
                }

                log_execution_result(&http_result, &processed_request, log);

                // Log verbose details
                if config.verbose {
                    log_response_details(&http_result, log, config.pretty_json);
                }

                // Log assertion results
                if !processed_request.assertions.is_empty() {
                    log_assertion_results(&http_result, log);
                }

                add_request_context_with_result(
                    &mut request_contexts,
                    processed_request,
                    Some(http_result),
                    counters.total,
                );
            }
        }
    }

    log_file_summary(&counters, log);

    Some(HttpFileResults {
        filename: http_file.to_string(),
        success_count: counters.success,
        failed_count: counters.failed,
        skipped_count: counters.skipped,
        result_contexts: request_contexts,
    })
}

/// Process HTTP files with the default executor.
pub fn process_http_files(
    files: &[String],
    verbose: bool,
    log_filename: Option<&str>,
    environment: Option<&str>,
    insecure: bool,
    pretty_json: bool,
) -> Result<ProcessorResults> {
    let config = ProcessorConfig::new(files)
        .with_verbose(verbose)
        .with_log_filename(log_filename)
        .with_environment(environment)
        .with_insecure(insecure)
        .with_pretty_json(pretty_json);

    process_http_files_with_config(&config, &|request, verbose, insecure| {
        runner::execute_http_request(request, verbose, insecure)
    })
}

/// Process HTTP files with the default executor in silent mode.
pub fn process_http_files_with_silent(
    files: &[String],
    verbose: bool,
    log_filename: Option<&str>,
    environment: Option<&str>,
    insecure: bool,
    pretty_json: bool,
    silent: bool,
) -> Result<ProcessorResults> {
    let config = ProcessorConfig::new(files)
        .with_verbose(verbose)
        .with_log_filename(log_filename)
        .with_environment(environment)
        .with_insecure(insecure)
        .with_pretty_json(pretty_json)
        .with_silent(silent);

    process_http_files_with_config(&config, &|request, verbose, insecure| {
        runner::execute_http_request(request, verbose, insecure)
    })
}

/// Process HTTP files with a custom executor.
pub fn process_http_files_with_executor<F>(
    files: &[String],
    verbose: bool,
    log_filename: Option<&str>,
    environment: Option<&str>,
    insecure: bool,
    pretty_json: bool,
    executor: &F,
) -> Result<ProcessorResults>
where
    F: Fn(&HttpRequest, bool, bool) -> Result<HttpResult>,
{
    let config = ProcessorConfig::new(files)
        .with_verbose(verbose)
        .with_log_filename(log_filename)
        .with_environment(environment)
        .with_insecure(insecure)
        .with_pretty_json(pretty_json);

    process_http_files_with_config(&config, executor)
}

/// Process HTTP files with a custom executor and full configuration.
pub fn process_http_files_with_config<F>(
    config: &ProcessorConfig,
    executor: &F,
) -> Result<ProcessorResults>
where
    F: Fn(&HttpRequest, bool, bool) -> Result<HttpResult>,
{
    let mut log = Log::new_with_silent(config.log_filename, config.silent)?;
    let mut http_file_results = Vec::<HttpFileResults>::new();
    let mut totals = TotalCounters::new();

    for http_file in config.files {
        if let Some(file_results) = process_single_file(http_file, config, executor, &mut log) {
            totals.add_file_results(&RequestCounters {
                success: file_results.success_count,
                failed: file_results.failed_count,
                skipped: file_results.skipped_count,
                total: 0, // Not used in add_file_results
            });
            http_file_results.push(file_results);
        }
    }

    log_overall_summary(&totals, &mut log);

    Ok(ProcessorResults {
        success: totals.failed == 0,
        files: http_file_results,
    })
}
