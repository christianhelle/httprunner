// WASM-specific async execution for results view
use crate::results_view::{ExecutionResult, ResultsView};
use httprunner_lib::parser;
use std::sync::{Arc, Mutex};

impl ResultsView {
    pub fn run_content_async(
        &mut self,
        content: String,
        environment: Option<&str>,
        ctx: &egui::Context,
    ) {
        let results: Arc<Mutex<Vec<ExecutionResult>>> = Arc::clone(&self.results);
        let is_running: Arc<Mutex<bool>> = Arc::clone(&self.is_running);
        let ctx = ctx.clone();
        let env = environment.map(|s| s.to_string());

        // Clear previous results
        if let Ok(mut r) = results.lock() {
            r.clear();
            r.push(ExecutionResult::Running {
                message: "Parsing and running requests from content...".to_string(),
            });
        }

        if let Ok(mut running) = is_running.lock() {
            *running = true;
        }

        wasm_bindgen_futures::spawn_local(async move {

            // Parse the content
            let parse_result = parser::parse_http_content(&content, env.as_deref());

            match parse_result {
                Ok(requests) => {
                    if let Ok(mut r) = results.lock() {
                        r.clear();
                    }

                    let total = requests.len();
                    for (idx, request) in requests.into_iter().enumerate() {
                        // Show running status
                        if let Ok(mut r) = results.lock() {
                            r.push(ExecutionResult::Running {
                                message: format!(
                                    "Executing {}/{}: {} {}",
                                    idx + 1,
                                    total,
                                    request.method,
                                    request.url
                                ),
                            });
                        }
                        ctx.request_repaint();

                        #[cfg(target_arch = "wasm32")]
                        {
                            use web_sys::console;
                            console::log_1(&"Calling execute_request_async...".into());
                        }

                        let result = execute_request_async(request).await;

                        #[cfg(target_arch = "wasm32")]
                        {
                            use web_sys::console;
                            console::log_1(
                                &format!("Request {}/{} complete", idx + 1, total).into(),
                            );
                        }

                        if let Ok(mut r) = results.lock() {
                            // Remove running message
                            if let Some(last) = r.last() {
                                if matches!(last, ExecutionResult::Running { .. }) {
                                    r.pop();
                                }
                            }
                            r.push(result);
                        }
                        ctx.request_repaint();
                    }
                }
                Err(e) => {
                    if let Ok(mut r) = results.lock() {
                        r.clear();
                        r.push(ExecutionResult::Failure {
                            method: "PARSE".to_string(),
                            url: "".to_string(),
                            error: format!("Failed to parse content: {}", e),
                        });
                    }
                    ctx.request_repaint();
                }
            }

            if let Ok(mut running) = is_running.lock() {
                *running = false;
            }
            ctx.request_repaint();
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_file_async(&mut self, path: &Path, environment: Option<&str>, ctx: &egui::Context) {
        let path = path.to_path_buf();
        let env = environment.map(|s| s.to_string());
        let results: Arc<Mutex<Vec<ExecutionResult>>> = Arc::clone(&self.results);
        let is_running: Arc<Mutex<bool>> = Arc::clone(&self.is_running);
        let ctx = ctx.clone();

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

        // Spawn async task for WASM
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(path_str) = path.to_str() {
                // Parse the file
                match httprunner_lib::parser::parse_http_file(path_str, env.as_deref()) {
                    Ok(requests) => {
                        if let Ok(mut r) = results.lock() {
                            r.clear();
                        }

                        for request in requests {
                            let result = execute_request_async(request).await;
                            if let Ok(mut r) = results.lock() {
                                r.push(result);
                            }
                            ctx.request_repaint();
                        }
                    }
                    Err(e) => {
                        if let Ok(mut r) = results.lock() {
                            r.clear();
                            r.push(ExecutionResult::Failure {
                                method: "PARSE".to_string(),
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
            ctx.request_repaint();
        });
    }

    pub fn run_single_request_async(
        &mut self,
        content: &str,
        index: usize,
        environment: Option<&str>,
        ctx: &egui::Context,
    ) {
        let content = content.to_string();
        let env = environment.map(|s| s.to_string());
        let results: Arc<Mutex<Vec<ExecutionResult>>> = Arc::clone(&self.results);
        let is_running: Arc<Mutex<bool>> = Arc::clone(&self.is_running);
        let ctx = ctx.clone();

        // Clear previous results
        if let Ok(mut r) = results.lock() {
            r.clear();
            r.push(ExecutionResult::Running {
                message: format!("Running request {}...", index + 1),
            });
        }

        if let Ok(mut running) = is_running.lock() {
            *running = true;
        }

        wasm_bindgen_futures::spawn_local(async move {
            // Parse the content
            if let Ok(requests) =
                httprunner_lib::parser::parse_http_content(&content, env.as_deref())
            {
                if let Some(request) = requests.get(index) {
                    let result = execute_request_async(request.clone()).await;
                    if let Ok(mut r) = results.lock() {
                        r.clear();
                        r.push(result);
                    }
                } else if let Ok(mut r) = results.lock() {
                    r.clear();
                    r.push(ExecutionResult::Failure {
                        method: "INDEX".to_string(),
                        url: "content".to_string(),
                        error: format!("Request index {} not found", index),
                    });
                }
            } else if let Ok(mut r) = results.lock() {
                r.clear();
                r.push(ExecutionResult::Failure {
                    method: "PARSE".to_string(),
                    url: "content".to_string(),
                    error: "Failed to parse HTTP content".to_string(),
                });
            }

            if let Ok(mut running) = is_running.lock() {
                *running = false;
            }
            ctx.request_repaint();
        });
    }
}

async fn execute_request_async(request: httprunner_lib::HttpRequest) -> ExecutionResult {
    #[cfg(not(target_arch = "wasm32"))]
    use std::time::Instant;
    #[cfg(target_arch = "wasm32")]
    use web_time::Instant;

    let start = Instant::now();

    // Execute the request using the async runner
    match httprunner_lib::execute_http_request_async(&request, false, false).await {
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
