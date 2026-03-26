// WASM-specific async execution for results view
use crate::results_view::{ExecutionResult, ResultsView};
use futures_util::FutureExt;
use httprunner_core::parser;
use std::any::Any;
use std::panic::AssertUnwindSafe;
use std::sync::{Arc, Mutex};

impl ResultsView {
    pub fn run_content_async(
        &mut self,
        content: String,
        environment: Option<&str>,
        ctx: &egui::Context,
    ) -> bool {
        if !self.try_start_run("Parsing and running requests from content...".to_string()) {
            return false;
        }

        let results: Arc<Mutex<Vec<ExecutionResult>>> = Arc::clone(&self.results);
        let is_running: Arc<Mutex<bool>> = Arc::clone(&self.is_running);
        let ctx = ctx.clone();
        let env = environment.map(|s| s.to_string());

        wasm_bindgen_futures::spawn_local(async move {
            let run_result = AssertUnwindSafe(async {
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
                                if let Some(last) = r.last()
                                    && matches!(last, ExecutionResult::Running { .. })
                                {
                                    r.pop();
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
            })
            .catch_unwind()
            .await;

            if let Err(panic) = run_result {
                let panic_message = panic_to_string(&panic);
                if let Ok(mut r) = results.lock() {
                    r.clear();
                    r.push(ExecutionResult::Failure {
                        method: "PANIC".to_string(),
                        url: "content".to_string(),
                        error: format!("Async execution panicked: {}", panic_message),
                    });
                }
            }

            if let Ok(mut running) = is_running.lock() {
                *running = false;
            }
            ctx.request_repaint();
        });

        true
    }

    pub fn run_single_request_async(
        &mut self,
        content: &str,
        index: usize,
        environment: Option<&str>,
        ctx: &egui::Context,
    ) -> bool {
        if !self.try_start_run(format!("Running request {}...", index + 1)) {
            return false;
        }

        let content = content.to_string();
        let env = environment.map(|s| s.to_string());
        let results: Arc<Mutex<Vec<ExecutionResult>>> = Arc::clone(&self.results);
        let is_running: Arc<Mutex<bool>> = Arc::clone(&self.is_running);
        let ctx = ctx.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let run_result = AssertUnwindSafe(async {
                // Parse the content
                if let Ok(requests) =
                    httprunner_core::parser::parse_http_content(&content, env.as_deref())
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
                            error: format!("Request index {} not found", index + 1),
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
            })
            .catch_unwind()
            .await;

            if let Err(panic) = run_result {
                let panic_message = panic_to_string(&panic);
                if let Ok(mut r) = results.lock() {
                    r.clear();
                    r.push(ExecutionResult::Failure {
                        method: "PANIC".to_string(),
                        url: "content".to_string(),
                        error: format!("Async execution panicked: {}", panic_message),
                    });
                }
            }

            if let Ok(mut running) = is_running.lock() {
                *running = false;
            }
            ctx.request_repaint();
        });

        true
    }
}

fn panic_to_string(panic: &Box<dyn Any + Send>) -> String {
    if let Some(message) = panic.downcast_ref::<String>() {
        return message.clone();
    }

    if let Some(message) = panic.downcast_ref::<&str>() {
        return (*message).to_string();
    }

    "unknown panic".to_string()
}

async fn execute_request_async(request: httprunner_core::HttpRequest) -> ExecutionResult {
    match httprunner_core::execute_http_request_async(&request, false, false).await {
        Ok(result) => {
            if result.success {
                ExecutionResult::Success {
                    method: request.method,
                    url: request.url,
                    status: result.status_code,
                    duration_ms: result.duration_ms,
                    request_body: request.body,
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
