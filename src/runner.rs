use crate::assertions;
use crate::types::{HttpRequest, HttpResult};
use anyhow::Result;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::time::Instant;

pub fn execute_http_request(request: &HttpRequest, verbose: bool) -> Result<HttpResult> {
    let start_time = Instant::now();
    let has_assertions = !request.assertions.is_empty();

    let client = Client::builder()
        .danger_accept_invalid_hostnames(true)
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

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
    let response = match req_builder.send() {
        Ok(resp) => resp,
        Err(e) => {
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let error_message = if e.is_connect() {
                "Connection error"
            } else if e.is_timeout() {
                "Request timeout"
            } else {
                "Request failed"
            };

            return Ok(HttpResult {
                request_name: request.name.clone(),
                status_code: 0,
                success: false,
                error_message: Some(error_message.to_string()),
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

    if verbose || has_assertions {
        // Collect headers
        let mut headers = HashMap::new();
        for (name, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(name.to_string(), value_str.to_string());
            }
        }
        response_headers = Some(headers);

        // Collect body
        if let Ok(body) = response.text() {
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
