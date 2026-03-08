#[cfg(not(target_arch = "wasm32"))]
use httprunner_core::telemetry;
use httprunner_core::types::AssertionResult;
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use dioxus::prelude::*;

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

/// Run all requests from a file in a background thread.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_file(
    path: PathBuf,
    environment: Option<String>,
    delay_ms: u64,
    mut results: Signal<Vec<ExecutionResult>>,
    mut is_running: Signal<bool>,
) {
    telemetry::track_feature_usage("run_file");

    results.write().clear();
    results.write().push(ExecutionResult::Running {
        message: format!("Parsing {}...", path.display()),
    });
    is_running.set(true);

    // Channel: None = done, Some = result
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Option<ExecutionResult>>();

    std::thread::spawn(move || {
        let execution_start = std::time::Instant::now();
        let mut success_count = 0usize;
        let mut failed_count = 0usize;
        let mut skipped_count = 0usize;
        let mut total_count = 0usize;

        if let Some(path_str) = path.to_str() {
            // Clear the "Parsing..." message
            tx.send(Some(ExecutionResult::Running { message: "__clear__".to_string() })).ok();

            let result = httprunner_core::processor::process_http_file_incremental(
                path_str,
                environment.as_deref(),
                false,
                delay_ms,
                |_idx, total, process_result| {
                    total_count = total;
                    use httprunner_core::processor::RequestProcessingResult;
                    let exec_result = match process_result {
                        RequestProcessingResult::Skipped { request, reason } => {
                            skipped_count += 1;
                            ExecutionResult::Failure {
                                method: format!("⏭️ {}", request.method),
                                url: request.url,
                                error: format!("Skipped: {}", reason),
                            }
                        }
                        RequestProcessingResult::Executed { request, result } => {
                            let request_body = request.body.clone();
                            if result.success {
                                success_count += 1;
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
                                failed_count += 1;
                                ExecutionResult::Failure {
                                    method: request.method,
                                    url: request.url,
                                    error: result.error_message.unwrap_or_else(|| "Unknown error".to_string()),
                                }
                            }
                        }
                        RequestProcessingResult::Failed { request, error } => {
                            failed_count += 1;
                            ExecutionResult::Failure {
                                method: request.method,
                                url: request.url,
                                error,
                            }
                        }
                    };
                    tx.send(Some(exec_result)).ok();
                    true
                },
            );

            if let Err(e) = result {
                telemetry::track_error_message(&format!("Parse error: {}", e));
                tx.send(Some(ExecutionResult::Failure {
                    method: "PARSE".to_string(),
                    url: path.display().to_string(),
                    error: format!("Failed to parse file: {}", e),
                })).ok();
            } else {
                let total_duration = execution_start.elapsed().as_millis() as u64;
                telemetry::track_parse_complete(total_count, 0);
                telemetry::track_execution_complete(success_count, failed_count, skipped_count, total_duration);
            }
        } else {
            tx.send(Some(ExecutionResult::Failure {
                method: "READ".to_string(),
                url: path.display().to_string(),
                error: "Failed to convert path to string".to_string(),
            })).ok();
        }
        tx.send(None).ok(); // done
    });

    // Async task on local thread receives from channel and updates signals
    spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                Some(ExecutionResult::Running { message }) if message == "__clear__" => {
                    results.write().clear();
                }
                Some(r) => results.write().push(r),
                None => break,
            }
        }
        is_running.set(false);
    });
}

/// Run a single request from file in a background thread.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_single_request(
    path: PathBuf,
    index: usize,
    environment: Option<String>,
    delay_ms: u64,
    mut results: Signal<Vec<ExecutionResult>>,
    mut is_running: Signal<bool>,
) {
    results.write().clear();
    results.write().push(ExecutionResult::Running {
        message: format!("Running request {} from {}...", index + 1, path.display()),
    });
    is_running.set(true);

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Option<ExecutionResult>>();

    std::thread::spawn(move || {
        if let Some(path_str) = path.to_str() {
            let mut target_result: Option<ExecutionResult> = None;
            let result = httprunner_core::processor::process_http_file_incremental(
                path_str,
                environment.as_deref(),
                false,
                delay_ms,
                |idx, _total, process_result| {
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
                                        method: request.method, url: request.url,
                                        status: result.status_code, duration_ms: result.duration_ms,
                                        request_body,
                                        response_body: result.response_body.unwrap_or_default(),
                                        assertion_results: result.assertion_results,
                                    }
                                } else {
                                    ExecutionResult::Failure {
                                        method: request.method, url: request.url,
                                        error: result.error_message.unwrap_or_else(|| "Unknown error".to_string()),
                                    }
                                }
                            }
                            RequestProcessingResult::Failed { request, error } => {
                                ExecutionResult::Failure { method: request.method, url: request.url, error }
                            }
                        });
                        false
                    } else { true }
                },
            );

            let final_result = if let Err(e) = result {
                ExecutionResult::Failure {
                    method: "PARSE".to_string(),
                    url: path.display().to_string(),
                    error: format!("Failed to parse file: {}", e),
                }
            } else if let Some(r) = target_result {
                r
            } else {
                ExecutionResult::Failure {
                    method: "INDEX".to_string(),
                    url: path.display().to_string(),
                    error: format!("Request index {} not found", index),
                }
            };
            tx.send(Some(final_result)).ok();
        } else {
            tx.send(Some(ExecutionResult::Failure {
                method: "PATH".to_string(),
                url: path.display().to_string(),
                error: "Failed to convert path to string".to_string(),
            })).ok();
        }
        tx.send(None).ok();
    });

    spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                Some(r) => { results.write().clear(); results.write().push(r); }
                None => break,
            }
        }
        is_running.set(false);
    });
}

/// Run from content string (WASM).
#[cfg(target_arch = "wasm32")]
pub fn run_content_async(
    content: String,
    environment: Option<String>,
    mut results: Signal<Vec<ExecutionResult>>,
    mut is_running: Signal<bool>,
) {
    results.write().clear();
    results.write().push(ExecutionResult::Running {
        message: "Parsing and running requests...".to_string(),
    });
    is_running.set(true);

    spawn(async move {
        match httprunner_core::parser::parse_http_content(&content, environment.as_deref()) {
            Ok(requests) => {
                results.write().clear();
                let total = requests.len();
                for (idx, request) in requests.into_iter().enumerate() {
                    results.write().push(ExecutionResult::Running {
                        message: format!("Executing {}/{}: {} {}", idx + 1, total, request.method, request.url),
                    });
                    let r = execute_request_async(request).await;
                    {
                        let mut w = results.write();
                        if let Some(last) = w.last() {
                            if matches!(last, ExecutionResult::Running { .. }) { w.pop(); }
                        }
                        w.push(r);
                    }
                }
            }
            Err(e) => {
                results.write().clear();
                results.write().push(ExecutionResult::Failure {
                    method: "PARSE".to_string(), url: String::new(),
                    error: format!("Failed to parse content: {}", e),
                });
            }
        }
        is_running.set(false);
    });
}

#[cfg(target_arch = "wasm32")]
pub fn run_single_request_async(
    content: String,
    index: usize,
    environment: Option<String>,
    mut results: Signal<Vec<ExecutionResult>>,
    mut is_running: Signal<bool>,
) {
    results.write().clear();
    results.write().push(ExecutionResult::Running {
        message: format!("Running request {}...", index + 1),
    });
    is_running.set(true);

    spawn(async move {
        match httprunner_core::parser::parse_http_content(&content, environment.as_deref()) {
            Ok(requests) => {
                if let Some(request) = requests.into_iter().nth(index) {
                    let r = execute_request_async(request).await;
                    results.write().clear();
                    results.write().push(r);
                } else {
                    results.write().clear();
                    results.write().push(ExecutionResult::Failure {
                        method: "INDEX".to_string(), url: "content".to_string(),
                        error: format!("Request index {} not found", index + 1),
                    });
                }
            }
            Err(e) => {
                results.write().clear();
                results.write().push(ExecutionResult::Failure {
                    method: "PARSE".to_string(), url: "content".to_string(),
                    error: format!("Failed to parse content: {}", e),
                });
            }
        }
        is_running.set(false);
    });
}

#[cfg(target_arch = "wasm32")]
async fn execute_request_async(request: httprunner_core::HttpRequest) -> ExecutionResult {
    match httprunner_core::execute_http_request_async(&request, false, false).await {
        Ok(result) => {
            if result.success {
                ExecutionResult::Success {
                    method: request.method, url: request.url,
                    status: result.status_code, duration_ms: result.duration_ms,
                    request_body: request.body,
                    response_body: result.response_body.unwrap_or_default(),
                    assertion_results: result.assertion_results,
                }
            } else {
                ExecutionResult::Failure {
                    method: request.method, url: request.url,
                    error: result.error_message.unwrap_or_else(|| "Unknown error".to_string()),
                }
            }
        }
        Err(e) => ExecutionResult::Failure { method: request.method, url: request.url, error: e.to_string() },
    }
}

fn assertion_type_label(t: &httprunner_core::types::AssertionType) -> &'static str {
    match t {
        httprunner_core::types::AssertionType::Status => "Status Code",
        httprunner_core::types::AssertionType::Body => "Response Body",
        httprunner_core::types::AssertionType::Headers => "Response Headers",
    }
}

fn render_assertions(assertion_results: &[AssertionResult]) -> String {
    // Returns an HTML string for embedding via dangerous_inner_html
    let mut html = String::new();
    for ar in assertion_results {
        let atype = assertion_type_label(&ar.assertion.assertion_type);
        if ar.passed {
            html.push_str(&format!(
                "<div class='flex items-center gap-8'><span class='success'>  ✅</span><span>{}: Expected '{}'</span></div>",
                atype, html_escape(&ar.assertion.expected_value)
            ));
        } else {
            html.push_str(&format!(
                "<div><div class='flex items-center gap-8'><span class='failure'>  ❌</span><span>{}: {}</span></div>",
                atype, html_escape(ar.error_message.as_deref().unwrap_or("Failed"))
            ));
            if let Some(ref actual) = ar.actual_value {
                html.push_str(&format!(
                    "<div style='margin-left:24px;color:#eed49f;font-size:12px;'>Expected: '{}', Actual: '{}'</div>",
                    html_escape(&ar.assertion.expected_value), html_escape(actual)
                ));
            }
            html.push_str("</div>");
        }
    }
    html
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"',"&quot;")
}

#[component]
pub fn ResultsView(
    results: Signal<Vec<ExecutionResult>>,
    is_running: Signal<bool>,
    mut compact_mode: Signal<bool>,
) -> Element {
    rsx! {
        div {
            div {
                class: "flex items-center gap-8",
                style: "margin-bottom: 6px;",
                button {
                    class: if compact_mode() { "active" } else { "" },
                    onclick: move |_| compact_mode.set(true),
                    "📋 Compact"
                }
                button {
                    class: if !compact_mode() { "active" } else { "" },
                    onclick: move |_| compact_mode.set(false),
                    "📄 Verbose"
                }
                span {
                    style: "color: #8087a2; font-size: 12px; margin-left: auto;",
                    "(Ctrl+D to toggle)"
                }
            }
            hr {}

            if is_running() {
                div {
                    class: "flex items-center gap-8",
                    style: "padding: 8px 0;",
                    span { class: "spinner" }
                    span { class: "running", "Running..." }
                }
            }

            if results().is_empty() {
                p {
                    style: "color: #8087a2; padding: 8px 0;",
                    "No results yet. Select and run a request."
                }
            }

            for (idx, result) in results().iter().enumerate() {
                match result {
                    ExecutionResult::Success {
                        method, url, status, duration_ms,
                        request_body, response_body, assertion_results
                    } => {
                        if compact_mode() {
                            rsx! {
                                div {
                                    key: "{idx}",
                                    class: "result-card result-success",
                                    div {
                                        class: "flex items-center gap-8 flex-wrap",
                                        span { class: "success", "✅" }
                                        span { class: "mono", "{method} {url}" }
                                        span { style: "color: #8087a2;", "| {status} | {duration_ms} ms" }
                                    }
                                    if !assertion_results.is_empty() {
                                        div {
                                            dangerous_inner_html: "{render_assertions(assertion_results)}"
                                        }
                                    }
                                }
                            }
                        } else {
                            rsx! {
                                div {
                                    key: "{idx}",
                                    class: "result-card result-success",
                                    p { class: "success", "✅ SUCCESS" }
                                    p { class: "mono", "{method} {url}" }
                                    p { "Status: {status}" }
                                    p { "Duration: {duration_ms} ms" }
                                    if !assertion_results.is_empty() {
                                        hr {}
                                        p { class: "section-title", "🔍 Assertions:" }
                                        div { dangerous_inner_html: "{render_assertions(assertion_results)}" }
                                    }
                                    if let Some(body) = &request_body {
                                        if !body.trim().is_empty() {
                                            hr {}
                                            p { class: "section-title", "Request Body:" }
                                            div { class: "code-block", pre { "{body}" } }
                                        }
                                    }
                                    if !response_body.trim().is_empty() {
                                        hr {}
                                        p { class: "section-title", "Response:" }
                                        div { class: "code-block", pre { "{response_body}" } }
                                    }
                                }
                            }
                        }
                    }
                    ExecutionResult::Failure { method, url, error } => {
                        if compact_mode() {
                            rsx! {
                                div {
                                    key: "{idx}",
                                    class: "result-card result-failure",
                                    div { class: "flex items-center gap-8 flex-wrap",
                                        span { class: "failure", "❌" }
                                        span { class: "mono", "{method} {url}" }
                                    }
                                    p { class: "failure", style: "margin-top: 4px;", "{error}" }
                                }
                            }
                        } else {
                            rsx! {
                                div {
                                    key: "{idx}",
                                    class: "result-card result-failure",
                                    p { class: "failure", "❌ FAILED" }
                                    p { class: "mono", "{method} {url}" }
                                    p { class: "failure", "{error}" }
                                }
                            }
                        }
                    }
                    ExecutionResult::Running { message } => {
                        rsx! {
                            div {
                                key: "{idx}",
                                class: "result-card result-running",
                                div { class: "flex items-center gap-8",
                                    span { class: "spinner" }
                                    span { class: "running", "⏳ RUNNING" }
                                }
                                p { "{message}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
