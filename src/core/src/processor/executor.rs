use super::formatter::format_request_name;
use super::incremental_loop::{RequestReporter, SyncSleep, block_on, run_requests};
use super::output;
use crate::colors;
use crate::logging::Log;
use crate::parser;
use crate::redaction::{sanitize_request_for_output, sanitize_result_for_output};
use crate::runner;
use crate::types::{HttpFileResults, HttpRequest, HttpResult, ProcessorResults};
use anyhow::Result;

pub struct ProcessorConfig<'a> {
    pub files: &'a [String],
    pub verbose: bool,
    pub log_filename: Option<&'a str>,
    pub environment: Option<&'a str>,
    pub insecure: bool,
    pub pretty_json: bool,
    pub silent: bool,
    pub delay_ms: u64,
    pub include_secrets: bool,
    pub fail_fast: bool,
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
            delay_ms: 0,
            include_secrets: false,
            fail_fast: false,
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

    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    pub fn with_include_secrets(mut self, include_secrets: bool) -> Self {
        self.include_secrets = include_secrets;
        self
    }

    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }
}

/// Reporter adapter for the CLI batch path: logs each outcome (reusing the
/// `output` helpers) and aggregates pass/fail/skip counts. Fail-fast is signalled
/// by returning `false` and recorded in `halted`.
struct BatchReporter<'a, 'b> {
    config: &'a ProcessorConfig<'b>,
    log: &'a mut Log,
    counters: output::RequestCounters,
    halted: bool,
}

impl<'a, 'b> BatchReporter<'a, 'b> {
    fn new(config: &'a ProcessorConfig<'b>, log: &'a mut Log) -> Self {
        Self {
            config,
            log,
            counters: output::RequestCounters::new(),
            halted: false,
        }
    }
}

impl RequestReporter for BatchReporter<'_, '_> {
    fn request_started(&mut self, _idx: usize, _total: usize, request: &HttpRequest) {
        if self.config.verbose {
            let sanitized_request =
                sanitize_request_for_output(request, self.config.include_secrets);
            output::log_request_details(&sanitized_request, self.log, self.config.pretty_json);
        }
    }

    fn dependency_skipped(
        &mut self,
        _idx: usize,
        _total: usize,
        request: &HttpRequest,
        dep_name: &str,
    ) -> bool {
        self.counters.record_skip();
        let name_str = format_request_name(&request.name);
        self.log.writeln(&format!(
            "{} {} {} {} - Skipped: dependency '{}' not met (must succeed with HTTP 2xx)",
            colors::yellow("⏭️"),
            name_str,
            request.method,
            request.url,
            dep_name
        ));
        true
    }

    fn conditions_skipped(&mut self, _idx: usize, _total: usize, request: &HttpRequest) -> bool {
        self.counters.record_skip();
        output::log_conditions_not_met(request, self.log);
        true
    }

    fn condition_error(
        &mut self,
        _idx: usize,
        _total: usize,
        request: &HttpRequest,
        error: &anyhow::Error,
    ) -> bool {
        self.counters.record_skip();
        output::log_condition_error(request, error, self.log);
        true
    }

    fn substitution_error(
        &mut self,
        _idx: usize,
        _total: usize,
        _request: &HttpRequest,
        error: &anyhow::Error,
    ) -> bool {
        self.counters.record_failure();
        self.log
            .writeln(&format!("{} Internal error: {}", colors::red("❌"), error));
        if self.config.fail_fast {
            self.halted = true;
            return false;
        }
        true
    }

    fn executed(
        &mut self,
        _idx: usize,
        _total: usize,
        request: &HttpRequest,
        result: &HttpResult,
    ) -> bool {
        if result.success {
            self.counters.record_success();
        } else {
            self.counters.record_failure();
        }

        let sanitized_request = sanitize_request_for_output(request, self.config.include_secrets);
        output::log_execution_result(result, &sanitized_request, self.log);

        if self.config.verbose {
            let sanitized_result = sanitize_result_for_output(result, self.config.include_secrets);
            output::log_response_details(&sanitized_result, self.log, self.config.pretty_json);
        }

        if !request.assertions.is_empty() {
            let sanitized_result = sanitize_result_for_output(result, self.config.include_secrets);
            output::log_assertion_results(&sanitized_result, self.log);
        }

        let failed = !result.success;

        output::log_fail_fast_verbose(
            &sanitized_request,
            Some(result),
            self.config.include_secrets,
            self.config.pretty_json,
            self.config.fail_fast,
            self.config.verbose,
            self.log,
        );

        if failed && self.config.fail_fast {
            self.halted = true;
            return false;
        }
        true
    }

    fn execution_error(
        &mut self,
        _idx: usize,
        _total: usize,
        request: &HttpRequest,
        error: &anyhow::Error,
    ) -> bool {
        self.counters.record_failure();
        output::log_execution_error(request, error, self.log, self.config.include_secrets);
        output::log_fail_fast_verbose(
            request,
            None,
            self.config.include_secrets,
            self.config.pretty_json,
            self.config.fail_fast,
            self.config.verbose,
            self.log,
        );
        if self.config.fail_fast {
            self.halted = true;
            return false;
        }
        true
    }
}

fn process_single_file<F>(
    http_file: &str,
    config: &ProcessorConfig,
    executor: &F,
    log: &mut Log,
) -> Result<(HttpFileResults, bool)>
where
    F: Fn(&HttpRequest, bool, bool) -> Result<HttpResult>,
{
    output::log_file_header(http_file, log);

    let requests = match parser::parse_http_file(http_file, config.environment) {
        Ok(reqs) => reqs,
        Err(e) => {
            log.writeln(&format!("{} Error parsing file: {}", colors::red("❌"), e));
            return Err(e);
        }
    };

    if requests.is_empty() {
        log.writeln(&format!(
            "{} No HTTP requests found in file",
            colors::yellow("⚠️")
        ));
        return Err(anyhow::anyhow!("No HTTP requests found in file"));
    }

    log.writeln(&format!("Found {} HTTP request(s)\n", requests.len()));

    // When fail_fast is enabled we force full response capture for every request
    // (verbose || fail_fast) so the failed request always has body/headers
    // available, even though we only print verbose detail for the failing request.
    let capture = config.verbose || config.fail_fast;
    let wrapped = move |request: HttpRequest, _verbose: bool, insecure: bool| {
        async move { executor(&request, capture, insecure) }
    };

    let mut reporter = BatchReporter::new(config, log);
    let result_contexts = block_on(run_requests(
        &mut reporter,
        requests,
        config.insecure,
        config.delay_ms,
        &wrapped,
        SyncSleep,
    ))?;

    let BatchReporter {
        counters, halted, ..
    } = reporter;

    // Suppress the per-file summary when halting due to fail-fast so the output
    // ends on the failed request's detail.
    if !halted {
        output::log_file_summary(&counters, log);
    }

    Ok((
        HttpFileResults {
            filename: http_file.to_string(),
            success_count: counters.success,
            failed_count: counters.failed,
            skipped_count: counters.skipped,
            result_contexts,
        },
        halted,
    ))
}

/// The default executor: performs real blocking HTTP requests.
pub fn default_executor(
    request: &HttpRequest,
    verbose: bool,
    insecure: bool,
) -> Result<HttpResult> {
    runner::execute_http_request(request, verbose, insecure)
}

pub fn process_http_files<F>(
    config: &ProcessorConfig,
    executor: &F,
) -> Result<ProcessorResults>
where
    F: Fn(&HttpRequest, bool, bool) -> Result<HttpResult>,
{
    let mut log = Log::new_with_silent(config.log_filename, config.silent)?;
    let mut http_file_results = Vec::<HttpFileResults>::new();
    let mut totals = output::TotalCounters::new();
    let mut halted = false;

    if config.insecure {
        log.writeln(&format!(
            "{} TLS certificate validation is disabled (--insecure). Do not use against production endpoints.",
            colors::yellow("⚠️")
        ));
    }

    for http_file in config.files {
        match process_single_file(http_file, config, executor, &mut log) {
            Ok((file_results, file_halted)) => {
                totals.add_file_results(&output::RequestCounters {
                    success: file_results.success_count,
                    failed: file_results.failed_count,
                    skipped: file_results.skipped_count,
                });
                http_file_results.push(file_results);
                if file_halted {
                    // Fail-fast halt: skip all remaining files.
                    halted = true;
                    break;
                }
            }
            Err(_) => {
                // Parse error - count the entire file as failed
                totals.increment_files_failed();
                if config.fail_fast {
                    // Parse/processing errors also trigger a fail-fast halt.
                    halted = true;
                    break;
                }
            }
        }
    }

    // Suppress the overall summary when halting due to fail-fast.
    if !halted {
        output::log_overall_summary(&totals, &mut log);
    }

    Ok(ProcessorResults {
        success: totals.failed == 0,
        files: http_file_results,
    })
}
