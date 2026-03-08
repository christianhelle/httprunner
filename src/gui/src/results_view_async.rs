use crate::app::AppEvent;
use crate::results_view::{ExecutionResult, from_processing_result};
use httprunner_core::types::Variable;
use tokio::sync::mpsc::UnboundedSender;

pub fn start_run_content(
    content: String,
    env_variables: Vec<Variable>,
    delay_ms: u64,
    sender: UnboundedSender<AppEvent>,
) {
    let _ = sender.send(AppEvent::ExecutionStarted {
        message: "Parsing and running requests from content...".to_string(),
    });

    wasm_bindgen_futures::spawn_local(async move {
        let _ = sender.send(AppEvent::ExecutionCleared);

        let result = httprunner_core::processor::process_http_content_incremental_async(
            &content,
            env_variables,
            delay_ms,
            |_idx, _total, process_result| {
                let _ = sender.send(AppEvent::ExecutionPush(from_processing_result(
                    process_result,
                )));
                true
            },
        )
        .await;

        if let Err(error) = result {
            let _ = sender.send(AppEvent::ExecutionReplace(vec![ExecutionResult::Failure {
                method: "PARSE".to_string(),
                url: "content".to_string(),
                error: format!("Failed to parse content: {}", error),
            }]));
        }

        let _ = sender.send(AppEvent::ExecutionFinished);
    });
}

pub fn start_run_single_request_content(
    content: String,
    index: usize,
    env_variables: Vec<Variable>,
    delay_ms: u64,
    sender: UnboundedSender<AppEvent>,
) {
    let _ = sender.send(AppEvent::ExecutionStarted {
        message: format!("Running request {}...", index + 1),
    });

    wasm_bindgen_futures::spawn_local(async move {
        let mut target_result: Option<ExecutionResult> = None;

        let result = httprunner_core::processor::process_http_content_incremental_async(
            &content,
            env_variables,
            delay_ms,
            |idx, _total, process_result| {
                if idx == index {
                    target_result = Some(from_processing_result(process_result));
                    false
                } else {
                    true
                }
            },
        )
        .await;

        if let Err(error) = result {
            let _ = sender.send(AppEvent::ExecutionReplace(vec![ExecutionResult::Failure {
                method: "PARSE".to_string(),
                url: "content".to_string(),
                error: format!("Failed to parse content: {}", error),
            }]));
        } else if let Some(result) = target_result {
            let _ = sender.send(AppEvent::ExecutionReplace(vec![result]));
        } else {
            let _ = sender.send(AppEvent::ExecutionReplace(vec![ExecutionResult::Failure {
                method: "INDEX".to_string(),
                url: "content".to_string(),
                error: format!("Request index {} not found", index + 1),
            }]));
        }

        let _ = sender.send(AppEvent::ExecutionFinished);
    });
}
