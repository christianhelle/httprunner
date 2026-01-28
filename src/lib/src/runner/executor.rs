#[cfg(not(target_arch = "wasm32"))]
use super::response_processor::{
    build_error_result, build_success_result, build_temp_result_for_assertions, extract_headers,
    should_capture_response,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::assertions;
#[cfg(not(target_arch = "wasm32"))]
use crate::types::{HttpRequest, HttpResult};
#[cfg(not(target_arch = "wasm32"))]
use anyhow::Result;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::blocking::Client;
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

/// Execute an HTTP request synchronously and return the result.
#[cfg(not(target_arch = "wasm32"))]
pub fn execute_http_request(
    request: &HttpRequest,
    verbose: bool,
    insecure: bool,
) -> Result<HttpResult> {
    let start_time = Instant::now();

    let client = build_client(request, insecure)?;
    let req_builder = build_request(&client, request)?;

    let response = match req_builder.send() {
        Ok(resp) => resp,
        Err(e) => {
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let error_message = classify_request_error(&e);
            return Ok(build_error_result(request, error_message, duration_ms));
        }
    };

    let status_code = response.status().as_u16();
    let mut success = response.status().is_success();

    let (response_headers, response_body) =
        capture_response_details(request, verbose, response)?;

    let duration_ms = start_time.elapsed().as_millis() as u64;

    let assertion_results = if !request.assertions.is_empty() {
        let temp_result = build_temp_result_for_assertions(
            request,
            status_code,
            success,
            duration_ms,
            response_headers.clone(),
            response_body.clone(),
        );

        let results = assertions::evaluate_assertions(&request.assertions, &temp_result);
        let all_passed = results.iter().all(|r| r.passed);
        success = success && all_passed;
        results
    } else {
        Vec::new()
    };

    Ok(build_success_result(
        request,
        status_code,
        success,
        duration_ms,
        response_headers,
        response_body,
        assertion_results,
    ))
}

/// Build an HTTP client with the appropriate timeout and security settings.
#[cfg(not(target_arch = "wasm32"))]
fn build_client(request: &HttpRequest, insecure: bool) -> Result<Client> {
    let connection_timeout = request.connection_timeout.unwrap_or(30_000);
    let read_timeout = request.timeout.unwrap_or(60_000);

    let mut client_builder = Client::builder()
        .connect_timeout(std::time::Duration::from_millis(connection_timeout))
        .timeout(std::time::Duration::from_millis(read_timeout));

    if insecure {
        client_builder = client_builder
            .danger_accept_invalid_hostnames(true)
            .danger_accept_invalid_certs(true);
    }

    Ok(client_builder.build()?)
}

/// Build the HTTP request with method, URL, headers, and body.
#[cfg(not(target_arch = "wasm32"))]
fn build_request(
    client: &Client,
    request: &HttpRequest,
) -> Result<reqwest::blocking::RequestBuilder> {
    let method = request.method.to_uppercase();
    let method = reqwest::Method::from_bytes(method.as_bytes())?;

    let mut req_builder = client.request(method, &request.url);

    for header in &request.headers {
        req_builder = req_builder.header(&header.name, &header.value);
    }

    if let Some(ref body) = request.body {
        req_builder = req_builder.body(body.clone());
    }

    Ok(req_builder)
}

/// Classify request errors into user-friendly messages.
#[cfg(not(target_arch = "wasm32"))]
fn classify_request_error(e: &reqwest::Error) -> &'static str {
    if e.is_connect() {
        "Connection error"
    } else if e.is_timeout() {
        "Request timeout"
    } else {
        "Request failed"
    }
}

/// Capture response headers and body if needed.
#[cfg(not(target_arch = "wasm32"))]
fn capture_response_details(
    request: &HttpRequest,
    verbose: bool,
    response: reqwest::blocking::Response,
) -> Result<(Option<HashMap<String, String>>, Option<String>)> {
    if should_capture_response(request, verbose) {
        let headers = Some(extract_headers(response.headers()));
        let body = response.text().ok();
        Ok((headers, body))
    } else {
        Ok((None, None))
    }
}
