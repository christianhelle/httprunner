use httprunner_lib::types::AssertionResult;
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
}

fn execute_request(request: httprunner_lib::HttpRequest) -> ExecutionResult {
    use std::time::Instant;

    let start = Instant::now();

    // Execute the request using the runner
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
