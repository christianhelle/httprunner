use super::formatter::{format_json_if_valid, format_request_name};
use crate::colors;
use crate::logging::Log;
use crate::redaction::{sanitize_request_for_output, sanitize_result_for_output};
use crate::types::{AssertionType, HttpRequest, HttpResult};

pub(super) struct RequestCounters {
    pub success: u32,
    pub failed: u32,
    pub skipped: u32,
}

impl RequestCounters {
    pub fn new() -> Self {
        Self {
            success: 0,
            failed: 0,
            skipped: 0,
        }
    }

    pub fn record_success(&mut self) {
        self.success += 1;
    }

    pub fn record_failure(&mut self) {
        self.failed += 1;
    }

    pub fn record_skip(&mut self) {
        self.skipped += 1;
    }
}

pub(super) struct TotalCounters {
    pub success: u32,
    pub failed: u32,
    pub skipped: u32,
    pub files_processed: u32,
}

impl TotalCounters {
    pub fn new() -> Self {
        Self {
            success: 0,
            failed: 0,
            skipped: 0,
            files_processed: 0,
        }
    }

    pub fn add_file_results(&mut self, counters: &RequestCounters) {
        self.success += counters.success;
        self.failed += counters.failed;
        self.skipped += counters.skipped;
        self.files_processed += 1;
    }

    pub fn increment_files_failed(&mut self) {
        self.failed += 1;
        self.files_processed += 1;
    }
}

pub fn log_file_header(http_file: &str, log: &mut Log) {
    log.writeln(&format!(
        "{} HTTP File Runner - Processing file: {}",
        colors::blue("🚀"),
        http_file
    ));
    log.writeln(&"=".repeat(50));
}

pub fn log_file_summary(counters: &RequestCounters, log: &mut Log) {
    log.writeln(&format!("\n{}", "=".repeat(50)));
    log.writeln(&format!(
        "File Summary: {}, {}, {}\n",
        colors::green(&format!("{} Passed", counters.success)),
        colors::red(&format!("{} Failed", counters.failed)),
        colors::yellow(&format!("{} Skipped", counters.skipped))
    ));
}

pub fn log_overall_summary(totals: &TotalCounters, log: &mut Log) {
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

pub fn log_request_details(request: &HttpRequest, log: &mut Log, pretty_json: bool) {
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

pub fn log_execution_result(result: &HttpResult, processed_request: &HttpRequest, log: &mut Log) {
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

pub fn log_response_details(result: &HttpResult, log: &mut Log, pretty_json: bool) {
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

pub fn log_assertion_results(result: &HttpResult, log: &mut Log) {
    log.writeln(&format!("\n{} Assertion Results:", colors::blue("🔍")));
    for assertion_result in &result.assertion_results {
        log_single_assertion_result(assertion_result, log);
    }
    log.writeln(&"-".repeat(30));
}

fn log_single_assertion_result(assertion_result: &crate::types::AssertionResult, log: &mut Log) {
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
                .as_deref()
                .unwrap_or("Failed")
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

pub fn log_execution_error(
    processed_request: &HttpRequest,
    error: &anyhow::Error,
    log: &mut Log,
    include_secrets: bool,
) {
    let sanitized_request = sanitize_request_for_output(processed_request, include_secrets);
    log.writeln(&format!(
        "{} {} {} - Error: {}",
        colors::red("❌"),
        sanitized_request.method,
        sanitized_request.url,
        error
    ));
}

pub fn log_conditions_not_met(processed_request: &HttpRequest, log: &mut Log) {
    let name_str = format_request_name(&processed_request.name);
    log.writeln(&format!(
        "{} {} {} {} - Skipped: conditions not met",
        colors::yellow("⏭️"),
        name_str,
        processed_request.method,
        processed_request.url
    ));
}

pub fn log_condition_error(processed_request: &HttpRequest, error: &anyhow::Error, log: &mut Log) {
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

pub fn log_fail_fast_verbose(
    processed_request: &HttpRequest,
    http_result: Option<&HttpResult>,
    include_secrets: bool,
    pretty_json: bool,
    fail_fast: bool,
    verbose: bool,
    log: &mut Log,
) {
    if fail_fast && !verbose {
        let sanitized_request =
            sanitize_request_for_output(processed_request, include_secrets);
        log_request_details(&sanitized_request, log, pretty_json);

        if let Some(result) = http_result {
            let sanitized_result = sanitize_result_for_output(result, include_secrets);
            log_response_details(&sanitized_result, log, pretty_json);
        }
    }
}
