use httprunner_lib::types::AssertionResult;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

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

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_file(&mut self, path: &Path, environment: Option<&str>) {
        let path = path.to_path_buf();
        let env = environment.map(|s| s.to_string());
        let results = Arc::clone(&self.results);
        let is_running = Arc::clone(&self.is_running);

        // Clear previous results
        if let Ok(mut r) = results.lock() {
            r.clear();
            r.push(ExecutionResult::Running {
                message: format!("Parsing {}...", path.display()),
            });
        }

        if let Ok(mut running) = is_running.lock() {
            *running = true;
        }

        thread::spawn(move || {
            if let Some(path_str) = path.to_str() {
                // Parse the file first to get all requests
                match httprunner_lib::parser::parse_http_file(path_str, env.as_deref()) {
                    Ok(requests) => {
                        let total = requests.len();

                        // Clear running message
                        if let Ok(mut r) = results.lock() {
                            r.clear();
                        }

                        // Execute each request individually for immediate feedback
                        for (idx, request) in requests.into_iter().enumerate() {
                            // Show progress for current request
                            if let Ok(mut r) = results.lock() {
                                r.push(ExecutionResult::Running {
                                    message: format!(
                                        "Running {}/{}: {} {}",
                                        idx + 1,
                                        total,
                                        request.method,
                                        request.url
                                    ),
                                });
                            }

                            // Execute the request
                            let result = execute_request(request);

                            // Remove running message and add result
                            if let Ok(mut r) = results.lock() {
                                // Remove the running message we just added
                                if let Some(last) = r.last()
                                    && matches!(last, ExecutionResult::Running { .. })
                                {
                                    r.pop();
                                }
                                r.push(result);
                            }
                        }
                    }
                    Err(e) => {
                        if let Ok(mut r) = results.lock() {
                            r.clear();
                            r.push(ExecutionResult::Failure {
                                method: "PARSE".to_string(),
                                url: path.display().to_string(),
                                error: format!("Failed to parse file: {}", e),
                            });
                        }
                    }
                }
            } else if let Ok(mut r) = results.lock() {
                r.clear();
                r.push(ExecutionResult::Failure {
                    method: "READ".to_string(),
                    url: path.display().to_string(),
                    error: "Failed to convert path to string".to_string(),
                });
            }

            if let Ok(mut running) = is_running.lock() {
                *running = false;
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_single_request(&mut self, path: &Path, index: usize, environment: Option<&str>) {
        let path = path.to_path_buf();
        let env = environment.map(|s| s.to_string());
        let results = Arc::clone(&self.results);
        let is_running = Arc::clone(&self.is_running);

        // Clear previous results
        if let Ok(mut r) = results.lock() {
            r.clear();
            r.push(ExecutionResult::Running {
                message: format!("Running request {} from {}...", index + 1, path.display()),
            });
        }

        if let Ok(mut running) = is_running.lock() {
            *running = true;
        }

        thread::spawn(move || {
            // Parse the file
            if let Some(path_str) = path.to_str() {
                if let Ok(requests) =
                    httprunner_lib::parser::parse_http_file(path_str, env.as_deref())
                {
                    if let Some(request) = requests.get(index) {
                        let result = execute_request(request.clone());
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
                        method: "PARSE".to_string(),
                        url: path.display().to_string(),
                        error: "Failed to parse .http file".to_string(),
                    });
                }
            } else if let Ok(mut r) = results.lock() {
                r.clear();
                r.push(ExecutionResult::Failure {
                    method: "PATH".to_string(),
                    url: path.display().to_string(),
                    error: "Invalid file path".to_string(),
                });
            }

            if let Ok(mut running) = is_running.lock() {
                *running = false;
            }
        });
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

        if let Ok(is_running) = self.is_running.lock()
            && *is_running
        {
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
        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "✅");
            ui.monospace(format!("{} {}", method, url));
            ui.label(format!("| {} | {} ms", status, duration_ms));
        });

        // Show assertion results in compact form
        if !assertion_results.is_empty() {
            for assertion_result in assertion_results {
                let assertion_type_str = match assertion_result.assertion.assertion_type {
                    httprunner_lib::types::AssertionType::Status => "Status Code",
                    httprunner_lib::types::AssertionType::Body => "Response Body",
                    httprunner_lib::types::AssertionType::Headers => "Response Headers",
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
        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::from_rgb(200, 0, 0), "❌");
            ui.monospace(format!("{} {}", method, url));
        });
        ui.horizontal(|ui| {
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

        // Display assertion results if any
        if !params.assertion_results.is_empty() {
            ui.separator();
            ui.label("🔍 Assertion Results:");

            for assertion_result in params.assertion_results {
                let assertion_type_str = match assertion_result.assertion.assertion_type {
                    httprunner_lib::types::AssertionType::Status => "Status Code",
                    httprunner_lib::types::AssertionType::Body => "Response Body",
                    httprunner_lib::types::AssertionType::Headers => "Response Headers",
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

        ui.separator();
        ui.label("Response:");
        egui::ScrollArea::vertical()
            .id_salt(format!("response_body_{}", params.result_idx))
            .max_height(300.0)
            .show(ui, |ui| {
                ui.monospace(params.response_body);
            });
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
fn execute_request(request: httprunner_lib::HttpRequest) -> ExecutionResult {
    use std::time::Instant;

    let start = Instant::now();

    match httprunner_lib::runner::execute_http_request(&request, false, false) {
        Ok(result) => {
            let duration_ms = start.elapsed().as_millis() as u64;

            if result.success {
                ExecutionResult::Success {
                    method: request.method,
                    url: request.url,
                    status: result.status_code,
                    duration_ms,
                    response_body: result.response_body.unwrap_or_default(),
                    assertion_results: result.assertion_results.clone(),
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
        Err(e) => ExecutionResult::Failure {
            method: request.method,
            url: request.url,
            error: e.to_string(),
        },
    }
}
