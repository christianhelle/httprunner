use crate::assertions;
use crate::types::{HttpRequest, HttpResult};
use anyhow::Result;
use reqwest::Client;
use std::collections::HashMap;
use std::time::Instant;

pub async fn execute_http_request_async(
    request: &HttpRequest,
    verbose: bool,
    insecure: bool,
) -> Result<HttpResult> {
    let start_time = Instant::now();
    let has_assertions = !request.assertions.is_empty();

    let mut client_builder = Client::builder();

    // Timeouts are not supported in WASM/browser environment
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Default timeouts: 30 seconds for connection, 60 seconds for read
        // Timeouts are stored in milliseconds
        let connection_timeout = request.connection_timeout.unwrap_or(30_000);
        let read_timeout = request.timeout.unwrap_or(60_000);

        client_builder = client_builder
            .connect_timeout(std::time::Duration::from_millis(connection_timeout))
            .timeout(std::time::Duration::from_millis(read_timeout));

        if insecure {
            client_builder = client_builder
                .danger_accept_invalid_hostnames(true)
                .danger_accept_invalid_certs(true);
        }
    }

    let client = client_builder.build()?;

    let method = request.method.to_uppercase();
    let method = reqwest::Method::from_bytes(method.as_bytes())?;

    let mut req_builder = client.request(method, &request.url);

    // Add headers
    for header in &request.headers {
        req_builder = req_builder.header(&header.name, &header.value);
    }

    // Add body if present
    if let Some(ref body) = request.body {
        req_builder = req_builder.body(body.clone());
    }

    // Execute request
    let response = match req_builder.send().await {
        Ok(resp) => resp,
        Err(e) => {
            let duration_ms = start_time.elapsed().as_millis() as u64;
            
            return Ok(HttpResult {
                request_name: request.name.clone(),
                status_code: 0,
                success: false,
                error_message: Some(format!("Request failed: {}", e)),
                duration_ms,
                response_headers: None,
                response_body: None,
                assertion_results: Vec::new(),
            });
        }
    };

    let status_code = response.status().as_u16();
    let mut success = response.status().is_success();

    let mut response_headers: Option<HashMap<String, String>> = None;
    let mut response_body: Option<String> = None;

    // Collect response data if:
    // - Verbose mode is enabled (for display)
    // - Request has assertions (for assertion evaluation)
    // - Request is named (might be referenced by conditions in subsequent requests)
    let is_named = request.name.is_some();
    if verbose || has_assertions || is_named {
        // Collect headers
        let mut headers = HashMap::new();
        for (name, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(name.to_string(), value_str.to_string());
            }
        }
        response_headers = Some(headers);

        // Collect body
        if let Ok(body) = response.text().await {
            response_body = Some(body);
        }
    }

    let duration_ms = start_time.elapsed().as_millis() as u64;

    let mut assertion_results = Vec::new();
    if has_assertions {
        let temp_result = HttpResult {
            request_name: request.name.clone(),
            status_code,
            success,
            error_message: None,
            duration_ms,
            response_headers: response_headers.clone(),
            response_body: response_body.clone(),
            assertion_results: Vec::new(),
        };

        assertion_results = assertions::evaluate_assertions(&request.assertions, &temp_result);

        // Check if all assertions passed
        let all_assertions_passed = assertion_results.iter().all(|r| r.passed);
        success = success && all_assertions_passed;
    }

    Ok(HttpResult {
        request_name: request.name.clone(),
        status_code,
        success,
        error_message: None,
        duration_ms,
        response_headers,
        response_body,
        assertion_results,
    })
}
