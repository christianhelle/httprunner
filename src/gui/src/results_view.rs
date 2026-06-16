use httprunner_core::processor::RequestProcessingResult;
#[cfg(not(target_arch = "wasm32"))]
use httprunner_core::telemetry;
use httprunner_core::types::AssertionResult;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[cfg(not(target_arch = "wasm32"))]
use std::any::Any;
#[cfg(not(target_arch = "wasm32"))]
use std::panic::{AssertUnwindSafe, catch_unwind};
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

#[cfg(not(target_arch = "wasm32"))]
use std::thread;

/// Parameters for displaying verbose success results
struct VerboseSuccessParams<'a> {
    result_idx: usize,
    method: &'a str,
    url: &'a str,
    status: u16,
    duration_ms: u64,
    request_body: &'a Option<String>,
    response_body: &'a str,
    assertion_results: &'a [AssertionResult],
}

/// Detail captured for a failed result. For failures that originate from an
/// executed request, the optional fields preserve the full HTTP detail so the
/// verbose view (e.g. after a fail-fast halt) can render status, bodies and
/// assertion results. Non-execution failures (parse/read/panic/index/skip)
/// only populate `method`, `url` and `error`.
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct FailureResult {
    pub method: String,
    pub url: String,
    pub error: String,
    pub status: Option<u16>,
    pub duration_ms: Option<u64>,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub assertion_results: Vec<AssertionResult>,
}

impl FailureResult {
    /// Build a failure that only carries an error message, for failures with no
    /// HTTP response detail to preserve.
    pub fn simple(method: String, url: String, error: String) -> Self {
        Self {
            method,
            url,
            error,
            ..Default::default()
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ExecutionResult {
    Success {
        method: String,
        url: String,
        status: u16,
        duration_ms: u64,
        request_body: Option<String>,
        response_body: String,
        assertion_results: Vec<AssertionResult>,
    },
    Failure(FailureResult),
    Running {
        message: String,
    },
}

pub struct ResultsView {
    pub(crate) results: Arc<Mutex<Vec<ExecutionResult>>>,
    pub(crate) is_running: Arc<Mutex<bool>>,
    compact_mode: bool,
    /// When enabled, a run stops at the first failing request and the view
    /// auto-switches to verbose. In-memory only (never persisted).
    fail_fast: bool,
    /// Set by a run thread when it halts on a failure so the next render can
    /// force the results display into verbose mode.
    switch_to_verbose: Arc<AtomicBool>,
}

impl ResultsView {
    pub fn new() -> Self {
        Self {
            results: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(Mutex::new(false)),
            compact_mode: true, // Default to compact mode
            fail_fast: false,
            switch_to_verbose: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_compact_mode(&mut self, compact: bool) {
        self.compact_mode = compact;
    }

    pub fn is_compact_mode(&self) -> bool {
        self.compact_mode
    }

    pub fn is_fail_fast(&self) -> bool {
        self.fail_fast
    }

    pub fn set_fail_fast(&mut self, fail_fast: bool) {
        self.fail_fast = fail_fast;
    }

    /// Shared flag used by async (WASM) run paths to request a verbose switch
    /// when a run halts on a failure under fail-fast.
    pub(crate) fn switch_to_verbose_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.switch_to_verbose)
    }

    pub fn is_running(&self) -> bool {
        self.is_running.lock().map(|guard| *guard).unwrap_or(false)
    }

    pub(crate) fn try_start_run(&mut self, message: String) -> bool {
        if let Ok(mut running) = self.is_running.lock() {
            if *running {
                return false;
            }
            *running = true;
        } else {
            return false;
        }

        if let Ok(mut results) = self.results.lock() {
            results.clear();
            results.push(ExecutionResult::Running { message });
        }

        true
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_file(&mut self, path: &Path, environment: Option<&str>, delay_ms: u64) -> bool {
        let path = path.to_path_buf();
        let env = environment.map(|s| s.to_string());
        if !self.try_start_run(format!("Parsing {}...", path.display())) {
            return false;
        }

        let results = Arc::clone(&self.results);
        let is_running = Arc::clone(&self.is_running);
        let fail_fast = self.fail_fast;
        let switch_to_verbose = Arc::clone(&self.switch_to_verbose);

        // Track feature usage
        telemetry::track_feature_usage("run_file");

        thread::spawn(move || {
            let run_result = catch_unwind(AssertUnwindSafe(|| {
                let execution_start = std::time::Instant::now();

                if let Some(path_str) = path.to_str() {
                    // Clear the parsing message
                    if let Ok(mut r) = results.lock() {
                        r.clear();
                    }

                    let mut success_count = 0usize;
                    let mut failed_count = 0usize;
                    let mut skipped_count = 0usize;
                    let mut total_count = 0usize;

                    // Use the incremental processor which handles all features.
                    // Forward verbose = fail_fast to the executor so that, when
                    // fail-fast is on, the failing request captures full detail.
                    let result =
                        httprunner_core::processor::process_http_file_incremental_with_executor(
                            path_str,
                            env.as_deref(),
                            false, // insecure
                            delay_ms,
                            |_idx, total, process_result| {
                                total_count = total;

                                let should_continue =
                                    should_continue_after(&process_result, fail_fast);

                                use httprunner_core::processor::RequestProcessingResult;
                                match process_result {
                                    RequestProcessingResult::Skipped { request, reason } => {
                                        skipped_count += 1;
                                        if let Ok(mut r) = results.lock() {
                                            r.push(ExecutionResult::Failure(
                                                FailureResult::simple(
                                                    format!("⏭️ {}", request.method),
                                                    request.url,
                                                    format!("Skipped: {}", reason),
                                                ),
                                            ));
                                        }
                                    }
                                    RequestProcessingResult::Executed { request, result } => {
                                        let request_body = request.body.clone();
                                        if result.success {
                                            success_count += 1;
                                            if let Ok(mut r) = results.lock() {
                                                r.push(ExecutionResult::Success {
                                                    method: request.method,
                                                    url: request.url,
                                                    status: result.status_code,
                                                    duration_ms: result.duration_ms,
                                                    request_body,
                                                    response_body: result
                                                        .response_body
                                                        .unwrap_or_default(),
                                                    assertion_results: result.assertion_results,
                                                });
                                            }
                                        } else {
                                            failed_count += 1;
                                            if let Ok(mut r) = results.lock() {
                                                r.push(ExecutionResult::Failure(FailureResult {
                                                    method: request.method,
                                                    url: request.url,
                                                    error: result.error_message.unwrap_or_else(
                                                        || "Unknown error".to_string(),
                                                    ),
                                                    status: Some(result.status_code),
                                                    duration_ms: Some(result.duration_ms),
                                                    request_body,
                                                    response_body: result.response_body,
                                                    assertion_results: result.assertion_results,
                                                }));
                                            }
                                        }
                                    }
                                    RequestProcessingResult::Failed { request, error } => {
                                        failed_count += 1;
                                        if let Ok(mut r) = results.lock() {
                                            r.push(ExecutionResult::Failure(
                                                FailureResult::simple(
                                                    request.method,
                                                    request.url,
                                                    error,
                                                ),
                                            ));
                                        }
                                    }
                                }

                                // Fail-fast: halt on the first failing result and
                                // request the view to switch to verbose.
                                if !should_continue {
                                    switch_to_verbose.store(true, Ordering::SeqCst);
                                }
                                should_continue
                            },
                            &|req, _verbose, insecure| {
                                httprunner_core::runner::execute_http_request(
                                    req, fail_fast, insecure,
                                )
                            },
                        );

                    if let Err(e) = result {
                        // Track parse error
                        telemetry::track_error_message(&format!("Parse error: {}", e));

                        if let Ok(mut r) = results.lock() {
                            r.clear();
                            r.push(ExecutionResult::Failure(FailureResult::simple(
                                "PARSE".to_string(),
                                path.display().to_string(),
                                format!("Failed to parse file: {}", e),
                            )));
                        }
                    } else {
                        // Track execution completion
                        let total_duration = execution_start.elapsed().as_millis() as u64;

                        // Track parse metrics (approximate, since parsing is now integrated)
                        telemetry::track_parse_complete(total_count, 0);

                        telemetry::track_execution_complete(
                            success_count,
                            failed_count,
                            skipped_count,
                            total_duration,
                        );
                    }
                } else if let Ok(mut r) = results.lock() {
                    r.clear();
                    r.push(ExecutionResult::Failure(FailureResult::simple(
                        "READ".to_string(),
                        path.display().to_string(),
                        "Failed to convert path to string".to_string(),
                    )));
                }
            }));

            if let Err(panic) = run_result {
                let panic_message = panic_to_string(&panic);
                telemetry::track_error_message(&format!("Execution panic: {}", panic_message));
                if let Ok(mut r) = results.lock() {
                    r.clear();
                    r.push(ExecutionResult::Failure(FailureResult::simple(
                        "PANIC".to_string(),
                        path.display().to_string(),
                        format!("Background execution panicked: {}", panic_message),
                    )));
                }
            }

            if let Ok(mut running) = is_running.lock() {
                *running = false;
            }
        });

        true
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_single_request(
        &mut self,
        path: &Path,
        index: usize,
        environment: Option<&str>,
        delay_ms: u64,
    ) -> bool {
        let path = path.to_path_buf();
        let env = environment.map(|s| s.to_string());
        if !self.try_start_run(format!(
            "Running request {} from {}...",
            index + 1,
            path.display()
        )) {
            return false;
        }

        let results = Arc::clone(&self.results);
        let is_running = Arc::clone(&self.is_running);
        let fail_fast = self.fail_fast;
        let switch_to_verbose = Arc::clone(&self.switch_to_verbose);

        thread::spawn(move || {
            let run_result = catch_unwind(AssertUnwindSafe(|| {
                if let Some(path_str) = path.to_str() {
                    // Use the incremental processor to properly handle all features
                    // We process all requests up to the selected index to maintain context
                    // but only show the result of the selected request
                    let mut target_result: Option<ExecutionResult> = None;

                    let result =
                        httprunner_core::processor::process_http_file_incremental_with_executor(
                            path_str,
                            env.as_deref(),
                            false, // insecure
                            delay_ms,
                            |idx, _total, process_result| {
                                // Only capture the result for the target index
                                if idx == index {
                                    use httprunner_core::processor::RequestProcessingResult;
                                    let is_failure = !should_continue_after(&process_result, true);
                                    target_result = Some(match process_result {
                                        RequestProcessingResult::Skipped { request, reason } => {
                                            ExecutionResult::Failure(FailureResult::simple(
                                                format!("⏭️ {}", request.method),
                                                request.url,
                                                format!("Skipped: {}", reason),
                                            ))
                                        }
                                        RequestProcessingResult::Executed { request, result } => {
                                            let request_body = request.body.clone();
                                            if result.success {
                                                ExecutionResult::Success {
                                                    method: request.method,
                                                    url: request.url,
                                                    status: result.status_code,
                                                    duration_ms: result.duration_ms,
                                                    request_body,
                                                    response_body: result
                                                        .response_body
                                                        .unwrap_or_default(),
                                                    assertion_results: result.assertion_results,
                                                }
                                            } else {
                                                ExecutionResult::Failure(FailureResult {
                                                    method: request.method,
                                                    url: request.url,
                                                    error: result.error_message.unwrap_or_else(
                                                        || "Unknown error".to_string(),
                                                    ),
                                                    status: Some(result.status_code),
                                                    duration_ms: Some(result.duration_ms),
                                                    request_body,
                                                    response_body: result.response_body,
                                                    assertion_results: result.assertion_results,
                                                })
                                            }
                                        }
                                        RequestProcessingResult::Failed { request, error } => {
                                            ExecutionResult::Failure(FailureResult::simple(
                                                request.method,
                                                request.url,
                                                error,
                                            ))
                                        }
                                    });
                                    // Fail-fast: surface the failing request in verbose.
                                    if fail_fast && is_failure {
                                        switch_to_verbose.store(true, Ordering::SeqCst);
                                    }
                                    // Stop processing after capturing the target result
                                    false
                                } else {
                                    // Continue processing to maintain context
                                    true
                                }
                            },
                            &|req, _verbose, insecure| {
                                httprunner_core::runner::execute_http_request(
                                    req, fail_fast, insecure,
                                )
                            },
                        );

                    if let Err(e) = result {
                        if let Ok(mut r) = results.lock() {
                            r.clear();
                            r.push(ExecutionResult::Failure(FailureResult::simple(
                                "PARSE".to_string(),
                                path.display().to_string(),
                                format!("Failed to parse file: {}", e),
                            )));
                        }
                    } else if let Some(result) = target_result {
                        if let Ok(mut r) = results.lock() {
                            r.clear();
                            r.push(result);
                        }
                    } else if let Ok(mut r) = results.lock() {
                        r.clear();
                        r.push(ExecutionResult::Failure(FailureResult::simple(
                            "INDEX".to_string(),
                            path.display().to_string(),
                            format!("Request index {} not found", index + 1),
                        )));
                    }
                } else if let Ok(mut r) = results.lock() {
                    r.clear();
                    r.push(ExecutionResult::Failure(FailureResult::simple(
                        "PATH".to_string(),
                        path.display().to_string(),
                        "Failed to convert path to string".to_string(),
                    )));
                }
            }));

            if let Err(panic) = run_result {
                let panic_message = panic_to_string(&panic);
                telemetry::track_error_message(&format!("Execution panic: {}", panic_message));
                if let Ok(mut r) = results.lock() {
                    r.clear();
                    r.push(ExecutionResult::Failure(FailureResult::simple(
                        "PANIC".to_string(),
                        path.display().to_string(),
                        format!("Background execution panicked: {}", panic_message),
                    )));
                }
            }

            if let Ok(mut running) = is_running.lock() {
                *running = false;
            }
        });

        true
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        // If a run halted on a failure with fail-fast enabled, force verbose so
        // the failed request's full detail is shown.
        if self.switch_to_verbose.swap(false, Ordering::SeqCst) {
            self.compact_mode = false;
        }

        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.compact_mode, "📋 Compact")
                .on_hover_text("Show compact results (Ctrl+D to toggle)")
                .clicked()
            {
                self.compact_mode = true;
            }
            if ui
                .selectable_label(!self.compact_mode, "📄 Verbose")
                .on_hover_text("Show verbose results (Ctrl+D to toggle)")
                .clicked()
            {
                self.compact_mode = false;
            }

            ui.separator();

            ui.checkbox(&mut self.fail_fast, "Fail-fast")
                .on_hover_text("Stop the run at the first failed request and show it in verbose");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("(Ctrl+D to toggle compact/verbose)")
                        .small()
                        .color(egui::Color32::from_rgb(128, 128, 128)),
                );
            });
        });

        ui.separator();

        if self.is_running() {
            ui.spinner();
        }

        if let Ok(results) = self.results.lock() {
            if results.is_empty() {
                ui.label("No results yet. Select and run a request.");
                return;
            }

            for (result_idx, result) in results.iter().enumerate() {
                match result {
                    ExecutionResult::Success {
                        method,
                        url,
                        status,
                        duration_ms,
                        request_body,
                        response_body,
                        assertion_results,
                    } => {
                        if self.compact_mode {
                            self.show_compact_success(
                                ui,
                                method,
                                url,
                                *status,
                                *duration_ms,
                                assertion_results,
                            );
                        } else {
                            self.show_verbose_success(
                                ui,
                                VerboseSuccessParams {
                                    result_idx,
                                    method,
                                    url,
                                    status: *status,
                                    duration_ms: *duration_ms,
                                    request_body,
                                    response_body,
                                    assertion_results,
                                },
                            );
                        }
                    }
                    ExecutionResult::Failure(failure) => {
                        if self.compact_mode {
                            self.show_compact_failure(
                                ui,
                                &failure.method,
                                &failure.url,
                                &failure.error,
                            );
                        } else {
                            self.show_verbose_failure(ui, result_idx, failure);
                        }
                    }
                    ExecutionResult::Running { message } => {
                        ui.colored_label(egui::Color32::from_rgb(0, 100, 200), "⏳ RUNNING");
                        ui.label(message);
                        ui.separator();
                    }
                }
            }
        }
    }

    fn show_compact_success(
        &self,
        ui: &mut egui::Ui,
        method: &str,
        url: &str,
        status: u16,
        duration_ms: u64,
        assertion_results: &[AssertionResult],
    ) {
        ui.horizontal_wrapped(|ui| {
            ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "✅");
            ui.monospace(format!("{} {}", method, url));
            ui.label(format!("| {} | {} ms", status, duration_ms));
        });

        // Show assertion results in compact form
        if !assertion_results.is_empty() {
            for assertion_result in assertion_results {
                let assertion_type_str = match assertion_result.assertion.assertion_type {
                    httprunner_core::types::AssertionType::Status => "Status Code",
                    httprunner_core::types::AssertionType::Body => "Response Body",
                    httprunner_core::types::AssertionType::Headers => "Response Headers",
                };

                if assertion_result.passed {
                    ui.horizontal(|ui| {
                        ui.label("  ");
                        ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "✅");
                        ui.label(format!(
                            "{}: Expected '{}'",
                            assertion_type_str, assertion_result.assertion.expected_value
                        ));
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label("  ");
                        ui.colored_label(egui::Color32::from_rgb(200, 0, 0), "❌");
                        ui.label(format!(
                            "{}: {}",
                            assertion_type_str,
                            assertion_result
                                .error_message
                                .as_ref()
                                .unwrap_or(&"Failed".to_string())
                        ));
                    });

                    if let Some(ref actual) = assertion_result.actual_value {
                        ui.horizontal(|ui| {
                            ui.label("      ");
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 200, 0),
                                format!(
                                    "Expected: '{}', Actual: '{}'",
                                    assertion_result.assertion.expected_value, actual
                                ),
                            );
                        });
                    }
                }
            }
        }
        ui.separator();
    }

    fn show_compact_failure(&self, ui: &mut egui::Ui, method: &str, url: &str, error: &str) {
        ui.horizontal_wrapped(|ui| {
            ui.colored_label(egui::Color32::from_rgb(200, 0, 0), "❌");
            ui.monospace(format!("{} {}", method, url));
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("  ");
            ui.colored_label(egui::Color32::from_rgb(200, 0, 0), error);
        });
        ui.separator();
    }

    fn show_verbose_success(&self, ui: &mut egui::Ui, params: VerboseSuccessParams) {
        ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "✅ SUCCESS");
        ui.monospace(format!("{} {}", params.method, params.url));
        ui.label(format!("Status: {}", params.status));
        ui.label(format!("Duration: {} ms", params.duration_ms));

        // Verbose mode display order: 1. Assertion Results -> 2. Request Body -> 3. Response Body

        // 1. Display assertion results if any
        if !params.assertion_results.is_empty() {
            ui.separator();
            ui.label("🔍 Assertion Results:");

            for assertion_result in params.assertion_results {
                let assertion_type_str = match assertion_result.assertion.assertion_type {
                    httprunner_core::types::AssertionType::Status => "Status Code",
                    httprunner_core::types::AssertionType::Body => "Response Body",
                    httprunner_core::types::AssertionType::Headers => "Response Headers",
                };

                if assertion_result.passed {
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "  ✅");
                        ui.label(format!(
                            "{}: Expected '{}'",
                            assertion_type_str, assertion_result.assertion.expected_value
                        ));
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(200, 0, 0), "  ❌");
                        ui.label(format!(
                            "{}: {}",
                            assertion_type_str,
                            assertion_result
                                .error_message
                                .as_ref()
                                .unwrap_or(&"Failed".to_string())
                        ));
                    });

                    if let Some(ref actual) = assertion_result.actual_value {
                        ui.horizontal(|ui| {
                            ui.label("     ");
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 200, 0),
                                format!(
                                    "Expected: '{}'",
                                    assertion_result.assertion.expected_value
                                ),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("     ");
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 200, 0),
                                format!("Actual: '{}'", actual),
                            );
                        });
                    }
                }
            }
        }

        // 2. Display request body if present (skip if empty or whitespace only)
        if let Some(request_body) = params.request_body
            && !request_body.trim().is_empty()
        {
            ui.separator();
            ui.label("Request Body:");
            egui::ScrollArea::vertical()
                .id_salt(format!("request_body_{}", params.result_idx))
                .max_height(150.0)
                .show(ui, |ui| {
                    ui.monospace(request_body);
                });
        }

        // 3. Display response body (only if not empty or whitespace only)
        if !params.response_body.trim().is_empty() {
            ui.separator();
            ui.label("Response:");
            egui::ScrollArea::vertical()
                .id_salt(format!("response_body_{}", params.result_idx))
                .max_height(300.0)
                .show(ui, |ui| {
                    ui.monospace(params.response_body);
                });
        }
        ui.separator();
    }

    fn show_verbose_failure(&self, ui: &mut egui::Ui, result_idx: usize, failure: &FailureResult) {
        ui.colored_label(egui::Color32::from_rgb(200, 0, 0), "❌ FAILED");
        ui.monospace(format!("{} {}", failure.method, failure.url));
        if let Some(status) = failure.status {
            ui.label(format!("Status: {}", status));
        }
        if let Some(duration_ms) = failure.duration_ms {
            ui.label(format!("Duration: {} ms", duration_ms));
        }
        ui.colored_label(egui::Color32::from_rgb(200, 0, 0), &failure.error);

        // 1. Display assertion results if any
        if !failure.assertion_results.is_empty() {
            ui.separator();
            ui.label("🔍 Assertion Results:");
            self.show_verbose_assertions(ui, &failure.assertion_results);
        }

        // 2. Display request body if present (skip if empty or whitespace only)
        if let Some(request_body) = &failure.request_body
            && !request_body.trim().is_empty()
        {
            ui.separator();
            ui.label("Request Body:");
            egui::ScrollArea::vertical()
                .id_salt(format!("failure_request_body_{}", result_idx))
                .max_height(150.0)
                .show(ui, |ui| {
                    ui.monospace(request_body);
                });
        }

        // 3. Display response body if present (skip if empty or whitespace only)
        if let Some(response_body) = &failure.response_body
            && !response_body.trim().is_empty()
        {
            ui.separator();
            ui.label("Response:");
            egui::ScrollArea::vertical()
                .id_salt(format!("failure_response_body_{}", result_idx))
                .max_height(300.0)
                .show(ui, |ui| {
                    ui.monospace(response_body);
                });
        }
        ui.separator();
    }

    fn show_verbose_assertions(&self, ui: &mut egui::Ui, assertion_results: &[AssertionResult]) {
        for assertion_result in assertion_results {
            let assertion_type_str = match assertion_result.assertion.assertion_type {
                httprunner_core::types::AssertionType::Status => "Status Code",
                httprunner_core::types::AssertionType::Body => "Response Body",
                httprunner_core::types::AssertionType::Headers => "Response Headers",
            };

            if assertion_result.passed {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "  ✅");
                    ui.label(format!(
                        "{}: Expected '{}'",
                        assertion_type_str, assertion_result.assertion.expected_value
                    ));
                });
            } else {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(200, 0, 0), "  ❌");
                    ui.label(format!(
                        "{}: {}",
                        assertion_type_str,
                        assertion_result
                            .error_message
                            .as_ref()
                            .unwrap_or(&"Failed".to_string())
                    ));
                });

                if let Some(ref actual) = assertion_result.actual_value {
                    ui.horizontal(|ui| {
                        ui.label("     ");
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 200, 0),
                            format!("Expected: '{}'", assertion_result.assertion.expected_value),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("     ");
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 200, 0),
                            format!("Actual: '{}'", actual),
                        );
                    });
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn panic_to_string(panic: &Box<dyn Any + Send>) -> String {
    if let Some(message) = panic.downcast_ref::<String>() {
        return message.clone();
    }

    if let Some(message) = panic.downcast_ref::<&str>() {
        return (*message).to_string();
    }

    "unknown panic".to_string()
}

/// Decide whether a run should continue after processing `result`.
///
/// With fail-fast enabled, the run stops on the first failing request: an
/// executed request whose result was not successful, or a processing failure.
/// Skipped requests never trigger fail-fast.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn should_continue_after(result: &RequestProcessingResult, fail_fast: bool) -> bool {
    !(fail_fast && request_result_is_failure(result))
}

#[cfg(not(target_arch = "wasm32"))]
fn request_result_is_failure(result: &RequestProcessingResult) -> bool {
    match result {
        RequestProcessingResult::Executed { result, .. } => !result.success,
        RequestProcessingResult::Failed { .. } => true,
        RequestProcessingResult::Skipped { .. } => false,
    }
}

/// Async (WASM) counterpart of [`should_continue_after`].
pub(crate) fn should_continue_after_async(
    result: &RequestProcessingResult,
    fail_fast: bool,
) -> bool {
    !(fail_fast && async_result_is_failure(result))
}

fn async_result_is_failure(result: &RequestProcessingResult) -> bool {
    match result {
        RequestProcessingResult::Executed { result, .. } => !result.success,
        RequestProcessingResult::Failed { .. } => true,
        RequestProcessingResult::Skipped { .. } => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httprunner_core::types::{HttpRequest, HttpResult};

    fn sample_request() -> HttpRequest {
        HttpRequest {
            name: None,
            method: "GET".to_string(),
            url: "https://example.com".to_string(),
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

    fn sample_result(success: bool) -> HttpResult {
        HttpResult {
            request_name: None,
            status_code: if success { 200 } else { 500 },
            success,
            error_message: None,
            duration_ms: 1,
            response_headers: None,
            response_body: None,
            assertion_results: vec![],
        }
    }

    #[test]
    fn try_start_run_prevents_overlapping_execution() {
        let mut results_view = ResultsView::new();

        assert!(results_view.try_start_run("First".to_string()));
        assert!(!results_view.try_start_run("Second".to_string()));
        assert!(results_view.is_running());
    }

    #[test]
    fn should_continue_after_stops_on_failed_execution_when_fail_fast() {
        let result = RequestProcessingResult::Executed {
            request: sample_request(),
            result: sample_result(false),
        };
        assert!(!should_continue_after(&result, true));
        assert!(should_continue_after(&result, false));
    }

    #[test]
    fn should_continue_after_stops_on_processing_failure_when_fail_fast() {
        let result = RequestProcessingResult::Failed {
            request: sample_request(),
            error: "boom".to_string(),
        };
        assert!(!should_continue_after(&result, true));
        assert!(should_continue_after(&result, false));
    }

    #[test]
    fn should_continue_after_continues_on_success() {
        let result = RequestProcessingResult::Executed {
            request: sample_request(),
            result: sample_result(true),
        };
        assert!(should_continue_after(&result, true));
        assert!(should_continue_after(&result, false));
    }

    #[test]
    fn should_continue_after_never_stops_on_skip() {
        let result = RequestProcessingResult::Skipped {
            request: sample_request(),
            reason: "dependency".to_string(),
        };
        assert!(should_continue_after(&result, true));
        assert!(should_continue_after(&result, false));
    }

    #[test]
    fn should_continue_after_async_matches_sync_semantics() {
        let failed = RequestProcessingResult::Executed {
            request: sample_request(),
            result: sample_result(false),
        };
        let skipped = RequestProcessingResult::Skipped {
            request: sample_request(),
            reason: "dependency".to_string(),
        };
        let processing_failed = RequestProcessingResult::Failed {
            request: sample_request(),
            error: "boom".to_string(),
        };
        let ok = RequestProcessingResult::Executed {
            request: sample_request(),
            result: sample_result(true),
        };

        assert!(!should_continue_after_async(&failed, true));
        assert!(!should_continue_after_async(&processing_failed, true));
        assert!(should_continue_after_async(&skipped, true));
        assert!(should_continue_after_async(&ok, true));
        // fail-fast disabled never stops
        assert!(should_continue_after_async(&failed, false));
    }

    #[test]
    fn failure_result_simple_has_no_optional_fields() {
        let f = FailureResult::simple(
            "GET".to_string(),
            "https://example.com".to_string(),
            "connection refused".to_string(),
        );
        assert_eq!(f.method, "GET");
        assert_eq!(f.url, "https://example.com");
        assert_eq!(f.error, "connection refused");
        assert!(f.status.is_none());
        assert!(f.duration_ms.is_none());
        assert!(f.request_body.is_none());
        assert!(f.response_body.is_none());
        assert!(f.assertion_results.is_empty());
    }

    #[test]
    fn failure_result_full_construction_preserves_all_fields() {
        let f = FailureResult {
            method: "POST".to_string(),
            url: "https://api.example.com/data".to_string(),
            error: "500 Internal Server Error".to_string(),
            status: Some(500),
            duration_ms: Some(42),
            request_body: Some("{\"key\":\"value\"}".to_string()),
            response_body: Some("server error detail".to_string()),
            assertion_results: vec![],
        };
        assert_eq!(f.status, Some(500));
        assert_eq!(f.duration_ms, Some(42));
        assert_eq!(f.request_body.as_deref(), Some("{\"key\":\"value\"}"));
        assert_eq!(f.response_body.as_deref(), Some("server error detail"));
    }

    #[test]
    fn results_view_fail_fast_defaults_to_false_and_can_be_toggled() {
        let mut rv = ResultsView::new();
        assert!(!rv.is_fail_fast());
        rv.set_fail_fast(true);
        assert!(rv.is_fail_fast());
        rv.set_fail_fast(false);
        assert!(!rv.is_fail_fast());
    }

    #[test]
    fn results_view_compact_mode_defaults_to_true_and_can_be_toggled() {
        let mut rv = ResultsView::new();
        assert!(rv.is_compact_mode());
        rv.set_compact_mode(false);
        assert!(!rv.is_compact_mode());
        rv.set_compact_mode(true);
        assert!(rv.is_compact_mode());
    }
}
