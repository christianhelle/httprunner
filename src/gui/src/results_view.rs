#[cfg(not(target_arch = "wasm32"))]
use crate::app::AppEvent;
#[cfg(not(target_arch = "wasm32"))]
use httprunner_core::telemetry;
use httprunner_core::types::AssertionResult;
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::thread;
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::mpsc::UnboundedSender;

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
}

pub(crate) fn from_processing_result(
    process_result: httprunner_core::processor::RequestProcessingResult,
) -> ExecutionResult {
    use httprunner_core::processor::RequestProcessingResult;

    match process_result {
        RequestProcessingResult::Skipped { request, reason } => ExecutionResult::Failure {
            method: format!("⏭️ {}", request.method),
            url: request.url,
            error: format!("Skipped: {}", reason),
        },
        RequestProcessingResult::Executed { request, result } => {
            let request_body = request.body.clone();
            if result.success {
                ExecutionResult::Success {
                    method: request.method,
                    url: request.url,
                    status: result.status_code,
                    duration_ms: result.duration_ms,
                    request_body,
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
        RequestProcessingResult::Failed { request, error } => ExecutionResult::Failure {
            method: request.method,
            url: request.url,
            error,
        },
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn start_run_file(
    path: &Path,
    environment: Option<&str>,
    delay_ms: u64,
    sender: UnboundedSender<AppEvent>,
) {
    let path = path.to_path_buf();
    let environment = environment.map(str::to_string);

    telemetry::track_feature_usage("run_file");
    let _ = sender.send(AppEvent::ExecutionStarted {
        message: format!("Parsing {}...", path.display()),
    });

    thread::spawn(move || {
        let execution_start = std::time::Instant::now();

        if let Some(path_str) = path.to_str() {
            let _ = sender.send(AppEvent::ExecutionCleared);

            let mut success_count = 0usize;
            let mut failed_count = 0usize;
            let mut skipped_count = 0usize;
            let mut total_count = 0usize;

            let result = httprunner_core::processor::process_http_file_incremental(
                path_str,
                environment.as_deref(),
                false,
                delay_ms,
                |_idx, total, process_result| {
                    total_count = total;
                    match &process_result {
                        httprunner_core::processor::RequestProcessingResult::Skipped { .. } => {
                            skipped_count += 1;
                        }
                        httprunner_core::processor::RequestProcessingResult::Executed {
                            result,
                            ..
                        } => {
                            if result.success {
                                success_count += 1;
                            } else {
                                failed_count += 1;
                            }
                        }
                        httprunner_core::processor::RequestProcessingResult::Failed { .. } => {
                            failed_count += 1;
                        }
                    }

                    let _ = sender.send(AppEvent::ExecutionPush(from_processing_result(
                        process_result,
                    )));
                    true
                },
            );

            if let Err(error) = result {
                telemetry::track_error_message(&format!("Parse error: {}", error));
                let _ = sender.send(AppEvent::ExecutionReplace(vec![ExecutionResult::Failure {
                    method: "PARSE".to_string(),
                    url: path.display().to_string(),
                    error: format!("Failed to parse file: {}", error),
                }]));
            } else {
                let total_duration = execution_start.elapsed().as_millis() as u64;
                telemetry::track_parse_complete(total_count, 0);
                telemetry::track_execution_complete(
                    success_count,
                    failed_count,
                    skipped_count,
                    total_duration,
                );
            }
        } else {
            let _ = sender.send(AppEvent::ExecutionReplace(vec![ExecutionResult::Failure {
                method: "READ".to_string(),
                url: path.display().to_string(),
                error: "Failed to convert path to string".to_string(),
            }]));
        }

        let _ = sender.send(AppEvent::ExecutionFinished);
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn start_run_single_request(
    path: &Path,
    index: usize,
    environment: Option<&str>,
    delay_ms: u64,
    sender: UnboundedSender<AppEvent>,
) {
    let path = path.to_path_buf();
    let environment = environment.map(str::to_string);

    let _ = sender.send(AppEvent::ExecutionStarted {
        message: format!("Running request {} from {}...", index + 1, path.display()),
    });

    thread::spawn(move || {
        if let Some(path_str) = path.to_str() {
            let mut target_result: Option<ExecutionResult> = None;

            let result = httprunner_core::processor::process_http_file_incremental(
                path_str,
                environment.as_deref(),
                false,
                delay_ms,
                |idx, _total, process_result| {
                    if idx == index {
                        target_result = Some(from_processing_result(process_result));
                        false
                    } else {
                        true
                    }
                },
            );

            if let Err(error) = result {
                let _ = sender.send(AppEvent::ExecutionReplace(vec![ExecutionResult::Failure {
                    method: "PARSE".to_string(),
                    url: path.display().to_string(),
                    error: format!("Failed to parse file: {}", error),
                }]));
            } else if let Some(result) = target_result {
                let _ = sender.send(AppEvent::ExecutionReplace(vec![result]));
            } else {
                let _ = sender.send(AppEvent::ExecutionReplace(vec![ExecutionResult::Failure {
                    method: "INDEX".to_string(),
                    url: path.display().to_string(),
                    error: format!("Request index {} not found", index + 1),
                }]));
            }
        } else {
            let _ = sender.send(AppEvent::ExecutionReplace(vec![ExecutionResult::Failure {
                method: "PATH".to_string(),
                url: path.display().to_string(),
                error: "Failed to convert path to string".to_string(),
            }]));
        }

        let _ = sender.send(AppEvent::ExecutionFinished);
    });
}
