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

/// Configuration for processing HTTP files
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
    /// Create a new ProcessorConfig with the provided files and default settings
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

    /// Enable verbose output
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set log filename
    pub fn with_log_filename(mut self, log_filename: Option<&'a str>) -> Self {
        self.log_filename = log_filename;
        self
    }

    /// Set environment name
    pub fn with_environment(mut self, environment: Option<&'a str>) -> Self {
        self.environment = environment;
        self
    }

    /// Enable insecure HTTPS
    pub fn with_insecure(mut self, insecure: bool) -> Self {
        self.insecure = insecure;
        self
    }

    /// Enable pretty JSON formatting
    pub fn with_pretty_json(mut self, pretty_json: bool) -> Self {
        self.pretty_json = pretty_json;
        self
    }

    /// Enable silent mode (suppress console output)
    pub fn with_silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }
}

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

/// Check if request dependencies are met. Returns true if request should be skipped.
fn should_skip_due_to_dependency(
    processed_request: &HttpRequest,
    request_contexts: &[RequestContext],
    log: &mut Log,
    _request_count: u32,
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

/// Evaluate conditions and log results. Returns true if request should be skipped.
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
        match conditions::evaluate_conditions_verbose(
            &processed_request.conditions,
            request_contexts,
        ) {
            Ok((conditions_met, evaluation_results)) => {
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

                    return true;
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

                return true;
            }
        }
    } else {
        // Non-verbose mode: simple evaluation
        match conditions::evaluate_conditions(&processed_request.conditions, request_contexts) {
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

                    return true;
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

                return true;
            }
        }
    }

    false
}

/// Log request details in verbose mode
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

/// Log execution result
fn log_execution_result(result: &HttpResult, processed_request: &HttpRequest, log: &mut Log) {
    if result.success {
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
}

/// Log response details in verbose mode
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

/// Log assertion results
fn log_assertion_results(result: &HttpResult, log: &mut Log) {
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
                log.writeln(&format!("{}      Actual: '{}'", colors::yellow(""), actual));
            }
        }
    }
    log.writeln(&"-".repeat(30));
}

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

/// Process HTTP files with a configuration struct and custom executor
pub fn process_http_files_with_config<F>(
    config: &ProcessorConfig,
    executor: &F,
) -> Result<ProcessorResults>
where
    F: Fn(&HttpRequest, bool, bool) -> Result<HttpResult>,
{
    let mut log = Log::new_with_silent(config.log_filename, config.silent)?;
    let mut http_file_results = Vec::<HttpFileResults>::new();

    let mut total_success_count = 0;
    let mut total_failed_count = 0;
    let mut total_skipped_count = 0;
    let mut files_processed = 0;

    for http_file in config.files {
        log.writeln(&format!(
            "{} HTTP File Runner - Processing file: {}",
            colors::blue("🚀"),
            http_file
        ));
        log.writeln(&"=".repeat(50));

        let requests = match parser::parse_http_file(http_file, config.environment) {
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

        let mut success_count = 0;
        let mut failed_count = 0;
        let mut skipped_count = 0;
        let mut request_count = 0;
        let mut request_contexts: Vec<RequestContext> = Vec::new();

        for request in requests {
            request_count += 1;

            let mut processed_request = request.clone();

            if should_skip_due_to_dependency(
                &processed_request,
                &request_contexts,
                &mut log,
                request_count,
            ) {
                add_skipped_request_context(
                    &mut request_contexts,
                    processed_request,
                    request_count,
                );
                skipped_count += 1;
                continue;
            }

            if should_skip_due_to_conditions(
                &processed_request,
                &request_contexts,
                &mut log,
                config.verbose,
            ) {
                add_skipped_request_context(
                    &mut request_contexts,
                    processed_request,
                    request_count,
                );
                skipped_count += 1;
                continue;
            }

            substitute_request_variables_in_request(&mut processed_request, &request_contexts)?;
            substitute_functions_in_request(&mut processed_request)?;

            if config.verbose {
                log_request_details(&processed_request, &mut log, config.pretty_json);
            }

            let result = match executor(&processed_request, config.verbose, config.insecure) {
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
            } else {
                failed_count += 1;
            }

            log_execution_result(&result, &processed_request, &mut log);

            let context_name = processed_request
                .name
                .clone()
                .unwrap_or_else(|| format!("request_{}", request_count));

            request_contexts.push(RequestContext {
                name: context_name,
                request: processed_request.clone(),
                result: Some(result),
            });

            if config.verbose
                && let Some(ctx) = request_contexts.last()
                && let Some(ref result) = ctx.result
            {
                log_response_details(result, &mut log, config.pretty_json);
            }

            if !processed_request.assertions.is_empty()
                && let Some(ctx) = request_contexts.last()
                && let Some(ref result) = ctx.result
            {
                log_assertion_results(result, &mut log);
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
