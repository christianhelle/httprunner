use httprunner_lib::types::AssertionResult;
use iced::widget::{column, text, Column};
use iced::Element;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

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
    results: Arc<Mutex<Vec<ExecutionResult>>>,
    is_running: Arc<Mutex<bool>>,
}

impl ResultsView {
    pub fn new() -> Self {
        Self {
            results: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(Mutex::new(false)),
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

    pub fn run_file(&mut self, path: &Path, environment: Option<&str>) {
        let path = path.to_path_buf();
        let env = environment.map(|s| s.to_string());
        let results = Arc::clone(&self.results);
        let is_running = Arc::clone(&self.is_running);

        // Clear previous results
        if let Ok(mut r) = results.lock() {
            r.clear();
            r.push(ExecutionResult::Running {
                message: format!("Running all requests from {}...", path.display()),
            });
        }

        if let Ok(mut running) = is_running.lock() {
            *running = true;
        }

        thread::spawn(move || {
            if let Some(path_str) = path.to_str() {
                // Use processor::process_http_files for consistent behavior with CLI
                let files = vec![path_str.to_string()];
                match httprunner_lib::processor::process_http_files(
                    &files,
                    false, // verbose
                    None,  // log_filename
                    env.as_deref(),
                    false, // insecure
                    false, // pretty_json
                ) {
                    Ok(processor_results) => {
                        if let Ok(mut r) = results.lock() {
                            r.clear();
                            // Convert ProcessorResults to ExecutionResults
                            for file_result in processor_results.files {
                                for request_context in file_result.result_contexts {
                                    if let Some(http_result) = request_context.result {
                                        if http_result.success {
                                            r.push(ExecutionResult::Success {
                                                method: request_context.request.method,
                                                url: request_context.request.url,
                                                status: http_result.status_code,
                                                duration_ms: http_result.duration_ms,
                                                response_body: http_result
                                                    .response_body
                                                    .unwrap_or_default(),
                                                assertion_results: http_result
                                                    .assertion_results
                                                    .clone(),
                                            });
                                        } else {
                                            r.push(ExecutionResult::Failure {
                                                method: request_context.request.method,
                                                url: request_context.request.url,
                                                error: http_result
                                                    .error_message
                                                    .unwrap_or_else(|| "Unknown error".to_string()),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if let Ok(mut r) = results.lock() {
                            r.clear();
                            r.push(ExecutionResult::Failure {
                                method: "PROCESS".to_string(),
                                url: path.display().to_string(),
                                error: e.to_string(),
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

    pub fn view(&self) -> Element<'static, crate::app::Message> {
        let mut content: Column<crate::app::Message> = Column::new().spacing(10);

        if let Ok(is_running) = self.is_running.lock()
            && *is_running
        {
            content = content.push(text("⏳ Running..."));
        }

        if let Ok(results) = self.results.lock() {
            if results.is_empty() {
                content = content.push(text("No results yet. Select and run a request."));
                return content.into();
            }

            for result in results.iter() {
                match result {
                    ExecutionResult::Success {
                        method,
                        url,
                        status,
                        duration_ms,
                        response_body,
                        assertion_results,
                    } => {
                        let mut result_section = column![
                            text("✅ SUCCESS").style(text::success),
                            text(format!("{} {}", method, url)),
                            text(format!("Status: {}", status)),
                            text(format!("Duration: {} ms", duration_ms)),
                        ]
                        .spacing(5);

                        if !assertion_results.is_empty() {
                            result_section = result_section.push(text("🔍 Assertion Results:"));
                            for assertion_result in assertion_results {
                                let status_symbol = if assertion_result.passed {
                                    "✅"
                                } else {
                                    "❌"
                                };
                                result_section = result_section.push(text(format!(
                                    "  {} {:?}",
                                    status_symbol, assertion_result.assertion
                                )));
                            }
                        }

                        if !response_body.is_empty() {
                            result_section = result_section.push(text("Response Body:"));
                            let body_preview = if response_body.len() > 500 {
                                format!("{}...", &response_body[..500])
                            } else {
                                response_body.clone()
                            };
                            result_section = result_section.push(text(body_preview));
                        }

                        content = content.push(result_section);
                    }
                    ExecutionResult::Failure {
                        method,
                        url,
                        error,
                    } => {
                        let result_section = column![
                            text("❌ FAILURE").style(text::danger),
                            text(format!("{} {}", method, url)),
                            text(format!("Error: {}", error)),
                        ]
                        .spacing(5);

                        content = content.push(result_section);
                    }
                    ExecutionResult::Running { message } => {
                        content = content.push(text(format!("⏳ {}", message)));
                    }
                }
            }
        }

        content.into()
    }
}

// Helper function to execute a single request
fn execute_request(request: httprunner_lib::HttpRequest) -> ExecutionResult {
    match httprunner_lib::runner::execute_http_request(&request, false, false) {
        Ok(result) => {
            if result.success {
                ExecutionResult::Success {
                    method: request.method,
                    url: request.url,
                    status: result.status_code,
                    duration_ms: result.duration_ms,
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
