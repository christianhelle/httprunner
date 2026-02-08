#[cfg(not(target_arch = "wasm32"))]
use super::response_processor::{
    build_error_result, build_success_result, build_temp_result_for_assertions, extract_headers,
    should_capture_response,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::assertions;
#[cfg(not(target_arch = "wasm32"))]
use crate::telemetry::{ConnectionErrorCategory, track_connection_error};
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

#[cfg(not(target_arch = "wasm32"))]
type ResponseDetails = (Option<HashMap<String, String>>, Option<String>);

#[cfg(not(target_arch = "wasm32"))]
pub fn execute_http_request(
    request: &HttpRequest,
    verbose: bool,
    insecure: bool,
) -> Result<HttpResult> {
    let client = build_client(request, insecure)?;
    let req_builder = build_request(&client, request)?;

    let start_time = Instant::now();

    let response = match req_builder.send() {
        Ok(resp) => resp,
        Err(e) => {
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let (error_message, error_category) = classify_request_error(&e);

            // Track connection error telemetry
            track_connection_error(error_category, insecure);

            return Ok(build_error_result(request, error_message, duration_ms));
        }
    };

    let status_code = response.status().as_u16();
    let mut success = response.status().is_success();

    let (response_headers, response_body) = capture_response_details(request, verbose, response)?;

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

#[cfg(not(target_arch = "wasm32"))]
fn classify_request_error(e: &reqwest::Error) -> (&'static str, ConnectionErrorCategory) {
    // Check for SSL/TLS errors by inspecting the error source chain
    let error_string = e.to_string().to_lowercase();
    let has_ssl_error = error_string.contains("ssl")
        || error_string.contains("tls")
        || error_string.contains("certificate")
        || error_string.contains("handshake");

    if e.is_timeout() {
        ("Request timeout", ConnectionErrorCategory::Timeout)
    } else if e.is_connect() {
        if has_ssl_error {
            ("SSL/TLS error", ConnectionErrorCategory::Ssl)
        } else if error_string.contains("dns")
            || error_string.contains("resolve")
            || error_string.contains("name or service not known")
        {
            ("DNS resolution failed", ConnectionErrorCategory::Dns)
        } else {
            (
                "Connection error",
                ConnectionErrorCategory::ConnectionRefused,
            )
        }
    } else if has_ssl_error {
        ("SSL/TLS error", ConnectionErrorCategory::Ssl)
    } else {
        ("Request failed", ConnectionErrorCategory::Other)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn capture_response_details(
    request: &HttpRequest,
    verbose: bool,
    response: reqwest::blocking::Response,
) -> Result<ResponseDetails> {
    if should_capture_response(request, verbose) {
        let headers = Some(extract_headers(response.headers()));
        let body = response.text().ok();
        Ok((headers, body))
    } else {
        Ok((None, None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Header;

    fn create_test_request() -> HttpRequest {
        HttpRequest {
            name: Some("test_request".to_string()),
            method: "GET".to_string(),
            url: "https://httpbin.org/get".to_string(),
            headers: Vec::new(),
            body: None,
            assertions: Vec::new(),
            variables: Vec::new(),
            timeout: None,
            connection_timeout: None,
            depends_on: None,
            conditions: Vec::new(),
            pre_delay_ms: None,
            post_delay_ms: None,
        }
    }

    #[test]
    fn test_build_client_with_default_timeouts() {
        let request = create_test_request();
        let client = build_client(&request, false);
        assert!(client.is_ok());
    }

    #[test]
    fn test_build_client_with_custom_timeouts() {
        let mut request = create_test_request();
        request.timeout = Some(5000);
        request.connection_timeout = Some(2000);
        let client = build_client(&request, false);
        assert!(client.is_ok());
    }

    #[test]
    fn test_build_client_with_insecure_flag() {
        let request = create_test_request();
        let client = build_client(&request, true);
        assert!(client.is_ok());
    }

    #[test]
    fn test_build_client_with_zero_timeouts() {
        let mut request = create_test_request();
        request.timeout = Some(0);
        request.connection_timeout = Some(0);
        let client = build_client(&request, false);
        assert!(client.is_ok());
    }

    #[test]
    fn test_build_request_get() {
        let request = create_test_request();
        let client = Client::new();
        let result = build_request(&client, &request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_request_post_with_body() {
        let mut request = create_test_request();
        request.method = "POST".to_string();
        request.body = Some("{\"key\":\"value\"}".to_string());
        let client = Client::new();
        let result = build_request(&client, &request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_request_with_headers() {
        let mut request = create_test_request();
        request.headers.push(Header {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        });
        request.headers.push(Header {
            name: "Authorization".to_string(),
            value: "Bearer token123".to_string(),
        });
        let client = Client::new();
        let result = build_request(&client, &request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_request_put_method() {
        let mut request = create_test_request();
        request.method = "PUT".to_string();
        let client = Client::new();
        let result = build_request(&client, &request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_request_delete_method() {
        let mut request = create_test_request();
        request.method = "DELETE".to_string();
        let client = Client::new();
        let result = build_request(&client, &request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_request_patch_method() {
        let mut request = create_test_request();
        request.method = "PATCH".to_string();
        let client = Client::new();
        let result = build_request(&client, &request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_request_head_method() {
        let mut request = create_test_request();
        request.method = "HEAD".to_string();
        let client = Client::new();
        let result = build_request(&client, &request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_request_options_method() {
        let mut request = create_test_request();
        request.method = "OPTIONS".to_string();
        let client = Client::new();
        let result = build_request(&client, &request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_request_lowercase_method() {
        let mut request = create_test_request();
        request.method = "get".to_string();
        let client = Client::new();
        let result = build_request(&client, &request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_request_mixed_case_method() {
        let mut request = create_test_request();
        request.method = "PoSt".to_string();
        let client = Client::new();
        let result = build_request(&client, &request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_request_invalid_method() {
        let mut request = create_test_request();
        request.method = "INVALID_METHOD_@#$".to_string();
        let client = Client::new();
        let result = build_request(&client, &request);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_request_empty_method() {
        let mut request = create_test_request();
        request.method = "".to_string();
        let client = Client::new();
        let result = build_request(&client, &request);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_request_with_multiple_headers() {
        let mut request = create_test_request();
        for i in 0..10 {
            request.headers.push(Header {
                name: format!("X-Custom-Header-{}", i),
                value: format!("value_{}", i),
            });
        }
        let client = Client::new();
        let result = build_request(&client, &request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_request_post_without_body() {
        let mut request = create_test_request();
        request.method = "POST".to_string();
        request.body = None;
        let client = Client::new();
        let result = build_request(&client, &request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_request_with_empty_body() {
        let mut request = create_test_request();
        request.method = "POST".to_string();
        request.body = Some(String::new());
        let client = Client::new();
        let result = build_request(&client, &request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_request_with_large_body() {
        let mut request = create_test_request();
        request.method = "POST".to_string();
        request.body = Some("x".repeat(10000));
        let client = Client::new();
        let result = build_request(&client, &request);
        assert!(result.is_ok());
    }
}
