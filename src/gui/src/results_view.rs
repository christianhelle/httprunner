#[cfg(not(target_arch = "wasm32"))]
use httprunner_core::telemetry;
use httprunner_core::types::AssertionResult;
use serde::{Deserialize, Serialize};
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
    Failure {
        method: String,
        url: String,
        error: String,
    },
    Running {
        message: String,
    },
}

pub struct ResultsView {
    pub(crate) results: Arc<Mutex<Vec<ExecutionResult>>>,
    pub(crate) is_running: Arc<Mutex<bool>>,
    compact_mode: bool,
}

impl ResultsView {
    pub fn new() -> Self {
        Self {
            results: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(Mutex::new(false)),
            compact_mode: true, // Default to compact mode
        }
    }

    pub fn get_results(&self) -> Vec<ExecutionResult> {
        if let Ok(results) = self.results.lock() {
            // Filter out Running results as they are transient
            results
                .iter()
                .filter(|r| !matches!(r, ExecutionResult::Running { .. }))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn restore_results(&mut self, saved_results: Vec<ExecutionResult>) {
        if let Ok(mut results) = self.results.lock() {
            *results = saved_results;
        }
    }

    pub fn set_compact_mode(&mut self, compact: bool) {
        self.compact_mode = compact;
    }

    pub fn is_compact_mode(&self) -> bool {
        self.compact_mode
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

                    // Use the incremental processor which handles all features
                    let result = httprunner_core::processor::process_http_file_incremental(
                        path_str,
                        env.as_deref(),
                        false, // insecure
                        delay_ms,
                        |_idx, total, process_result| {
                            total_count = total;

                            use httprunner_core::processor::RequestProcessingResult;
                            match process_result {
                                RequestProcessingResult::Skipped { request, reason } => {
                                    skipped_count += 1;
                                    if let Ok(mut r) = results.lock() {
                                        r.push(ExecutionResult::Failure {
                                            method: format!("⏭️ {}", request.method),
                                            url: request.url,
                                            error: format!("Skipped: {}", reason),
                                        });
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
                                            r.push(ExecutionResult::Failure {
                                                method: request.method,
                                                url: request.url,
                                                error: result
                                                    .error_message
                                                    .unwrap_or_else(|| "Unknown error".to_string()),
                                            });
                                        }
                                    }
                                }
                                RequestProcessingResult::Failed { request, error } => {
                                    failed_count += 1;
                                    if let Ok(mut r) = results.lock() {
                                        r.push(ExecutionResult::Failure {
                                            method: request.method,
                                            url: request.url,
                                            error,
                                        });
                                    }
                                }
                            }
                            // Continue processing all requests for "Run All"
                            true
                        },
                    );

                    if let Err(e) = result {
                        // Track parse error
                        telemetry::track_error_message(&format!("Parse error: {}", e));

                        if let Ok(mut r) = results.lock() {
                            r.clear();
                            r.push(ExecutionResult::Failure {
                                method: "PARSE".to_string(),
                                url: path.display().to_string(),
                                error: format!("Failed to parse file: {}", e),
                            });
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
                    r.push(ExecutionResult::Failure {
                        method: "READ".to_string(),
                        url: path.display().to_string(),
                        error: "Failed to convert path to string".to_string(),
                    });
                }
            }));

            if let Err(panic) = run_result {
                let panic_message = panic_to_string(&panic);
                telemetry::track_error_message(&format!("Execution panic: {}", panic_message));
                if let Ok(mut r) = results.lock() {
                    r.clear();
                    r.push(ExecutionResult::Failure {
                        method: "PANIC".to_string(),
                        url: path.display().to_string(),
                        error: format!("Background execution panicked: {}", panic_message),
                    });
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

        thread::spawn(move || {
            let run_result = catch_unwind(AssertUnwindSafe(|| {
                if let Some(path_str) = path.to_str() {
                    // Use the incremental processor to properly handle all features
                    // We process all requests up to the selected index to maintain context
                    // but only show the result of the selected request
                    let mut target_result: Option<ExecutionResult> = None;

                    let result = httprunner_core::processor::process_http_file_incremental(
                        path_str,
                        env.as_deref(),
                        false, // insecure
                        delay_ms,
                        |idx, _total, process_result| {
                            // Only capture the result for the target index
                            if idx == index {
                                use httprunner_core::processor::RequestProcessingResult;
                                target_result = Some(match process_result {
                                    RequestProcessingResult::Skipped { request, reason } => {
                                        ExecutionResult::Failure {
                                            method: format!("⏭️ {}", request.method),
                                            url: request.url,
                                            error: format!("Skipped: {}", reason),
                                        }
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
                                            ExecutionResult::Failure {
                                                method: request.method,
                                                url: request.url,
                                                error: result
                                                    .error_message
                                                    .unwrap_or_else(|| "Unknown error".to_string()),
                                            }
                                        }
                                    }
                                    RequestProcessingResult::Failed { request, error } => {
                                        ExecutionResult::Failure {
                                            method: request.method,
                                            url: request.url,
                                            error,
                                        }
                                    }
                                });
                                // Stop processing after capturing the target result
                                false
                            } else {
                                // Continue processing to maintain context
                                true
                            }
                        },
                    );

                    if let Err(e) = result {
                        if let Ok(mut r) = results.lock() {
                            r.clear();
                            r.push(ExecutionResult::Failure {
                                method: "PARSE".to_string(),
                                url: path.display().to_string(),
                                error: format!("Failed to parse file: {}", e),
                            });
                        }
                    } else if let Some(result) = target_result {
                        if let Ok(mut r) = results.lock() {
                            r.clear();
                            r.push(result);
                        }
                    } else if let Ok(mut r) = results.lock() {
                        r.clear();
                        r.push(ExecutionResult::Failure {
                            method: "INDEX".to_string(),
                            url: path.display().to_string(),
                            error: format!("Request index {} not found", index),
                        });
                    }
                } else if let Ok(mut r) = results.lock() {
                    r.clear();
                    r.push(ExecutionResult::Failure {
                        method: "PATH".to_string(),
                        url: path.display().to_string(),
                        error: "Failed to convert path to string".to_string(),
                    });
                }
            }));

            if let Err(panic) = run_result {
                let panic_message = panic_to_string(&panic);
                telemetry::track_error_message(&format!("Execution panic: {}", panic_message));
                if let Ok(mut r) = results.lock() {
                    r.clear();
                    r.push(ExecutionResult::Failure {
                        method: "PANIC".to_string(),
                        url: path.display().to_string(),
                        error: format!("Background execution panicked: {}", panic_message),
                    });
                }
            }

            if let Ok(mut running) = is_running.lock() {
                *running = false;
            }
        });

        true
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
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
                    ExecutionResult::Failure { method, url, error } => {
                        if self.compact_mode {
                            self.show_compact_failure(ui, method, url, error);
                        } else {
                            self.show_verbose_failure(ui, method, url, error);
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

    fn show_verbose_failure(&self, ui: &mut egui::Ui, method: &str, url: &str, error: &str) {
        ui.colored_label(egui::Color32::from_rgb(200, 0, 0), "❌ FAILED");
        ui.monospace(format!("{} {}", method, url));
        ui.colored_label(egui::Color32::from_rgb(200, 0, 0), error);
        ui.separator();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_start_run_prevents_overlapping_execution() {
        let mut results_view = ResultsView::new();

        assert!(results_view.try_start_run("First".to_string()));
        assert!(!results_view.try_start_run("Second".to_string()));
        assert!(results_view.is_running());
    }
}
