// WASM-specific async execution for results view
use crate::results_view::{ExecutionResult, ResultsView};
use futures_util::FutureExt;
use httprunner_core::parser;
use httprunner_core::runner::{
    AsyncRequestFuture, AsyncRequestProcessingResult, process_http_requests_incremental_async,
};
use httprunner_core::types::{HttpRequest, HttpResult};
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
                let parse_result = parser::parse_http_content(&content, env.as_deref());

                match parse_result {
                    Ok(requests) => {
                        if requests.is_empty() {
                            if let Ok(mut r) = results.lock() {
                                r.clear();
                            }
                            ctx.request_repaint();
                            return;
                        }

                        if let Err(error) = process_http_requests_incremental_async(
                            requests,
                            false,
                            0,
                            |_, _, process_result| {
                                if let Ok(mut r) = results.lock() {
                                    if let Some(last) = r.last()
                                        && matches!(last, ExecutionResult::Running { .. })
                                    {
                                        r.pop();
                                    }
                                    r.push(map_process_result(process_result));
                                }
                                ctx.request_repaint();
                                true
                            },
                            &boxed_async_executor,
                        )
                        .await
                        {
                            if let Ok(mut r) = results.lock() {
                                r.clear();
                                r.push(ExecutionResult::Failure {
                                    method: "PROCESS".to_string(),
                                    url: "content".to_string(),
                                    error: format!("Async processing failed: {}", error),
                                });
                            }
                            ctx.request_repaint();
                        }
                    }
                    Err(error) => {
                        if let Ok(mut r) = results.lock() {
                            r.clear();
                            r.push(ExecutionResult::Failure {
                                method: "PARSE".to_string(),
                                url: "content".to_string(),
                                error: format!("Failed to parse content: {}", error),
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
                match parser::parse_http_content(&content, env.as_deref()) {
                    Ok(requests) => {
                        let mut target_result: Option<ExecutionResult> = None;

                        let process_result = process_http_requests_incremental_async(
                            requests,
                            false,
                            0,
                            |idx, _, process_result| {
                                if idx == index {
                                    target_result = Some(map_process_result(process_result));
                                    return false;
                                }

                                true
                            },
                            &boxed_async_executor,
                        )
                        .await;

                        if let Ok(mut r) = results.lock() {
                            r.clear();
                            if let Err(error) = process_result {
                                r.push(ExecutionResult::Failure {
                                    method: "PROCESS".to_string(),
                                    url: "content".to_string(),
                                    error: format!("Async processing failed: {}", error),
                                });
                            } else if let Some(result) = target_result {
                                r.push(result);
                            } else {
                                r.push(ExecutionResult::Failure {
                                    method: "INDEX".to_string(),
                                    url: "content".to_string(),
                                    error: format!("Request index {} not found", index + 1),
                                });
                            }
                        }
                    }
                    Err(error) => {
                        if let Ok(mut r) = results.lock() {
                            r.clear();
                            r.push(ExecutionResult::Failure {
                                method: "PARSE".to_string(),
                                url: "content".to_string(),
                                error: format!("Failed to parse HTTP content: {}", error),
                            });
                        }
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

fn boxed_async_executor(
    request: &HttpRequest,
    verbose: bool,
    insecure: bool,
) -> AsyncRequestFuture<'_> {
    Box::pin(httprunner_core::execute_http_request_async(
        request, verbose, insecure,
    ))
}

fn map_process_result(process_result: AsyncRequestProcessingResult) -> ExecutionResult {
    match process_result {
        AsyncRequestProcessingResult::Skipped { request, reason } => ExecutionResult::Failure {
            method: format!("⏭️ {}", request.method),
            url: request.url,
            error: format!("Skipped: {}", reason),
        },
        AsyncRequestProcessingResult::Executed { request, result } => {
            map_http_result(request, result)
        }
        AsyncRequestProcessingResult::Failed { request, error } => ExecutionResult::Failure {
            method: request.method,
            url: request.url,
            error,
        },
    }
}

fn map_http_result(request: HttpRequest, result: HttpResult) -> ExecutionResult {
    if result.success {
        ExecutionResult::Success {
            method: request.method,
            url: request.url,
            status: result.status_code,
            duration_ms: result.duration_ms,
            request_body: request.body,
            response_body: result.response_body.unwrap_or_default(),
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
