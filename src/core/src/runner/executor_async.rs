use super::response_processor::{
    build_error_result, build_success_result, extract_headers,
    should_capture_response,
};
use super::{encode_form_body, needs_form_encoding};
use crate::types::{HttpRequest, HttpResult};
use anyhow::Result;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::OnceLock;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

pub async fn execute_http_request_async(
    request: &HttpRequest,
    verbose: bool,
    insecure: bool,
) -> Result<HttpResult> {
    let client = build_client_async(request, insecure)?;
    let req_builder = build_request_async(&client, request)?;

    let start_time = Instant::now();

    let response = match req_builder.send().await {
        Ok(resp) => resp,
        Err(e) => {
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let error_message = format!("Request failed: {}", e);
            return Ok(build_error_result(request, &error_message, duration_ms));
        }
    };

    let status_code = response.status().as_u16();
    let success = response.status().is_success();

    let (response_headers, response_body) =
        capture_response_details_async(request, verbose, response).await?;

    let duration_ms = start_time.elapsed().as_millis() as u64;

    Ok(build_success_result(
        request,
        status_code,
        success,
        duration_ms,
        response_headers,
        response_body,
        Vec::new(),
    ))
}

fn build_client_async(request: &HttpRequest, insecure: bool) -> Result<Client> {
    static CLIENT_CACHE: OnceLock<Client> = OnceLock::new();

    if let Some(client) = CLIENT_CACHE.get() {
        return Ok(client.clone());
    }

    #[allow(unused_mut)]
    let mut client_builder = Client::builder();

    #[cfg(not(target_arch = "wasm32"))]
    {
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

    #[cfg(target_arch = "wasm32")]
    {
        let _ = request;
        let _ = insecure;
    }

    let client = client_builder.build()?;
    let client = CLIENT_CACHE.get_or_init(|| client);

    Ok(client.clone())
}

fn build_request_async(client: &Client, request: &HttpRequest) -> Result<reqwest::RequestBuilder> {
    let method = request.method.to_uppercase();
    let method = reqwest::Method::from_bytes(method.as_bytes())?;

    let mut req_builder = client.request(method, &request.url);

    for header in &request.headers {
        req_builder = req_builder.header(&header.name, &header.value);
    }

    if let Some(ref body) = request.body {
        let body = if needs_form_encoding(&request.headers) {
            encode_form_body(body)
        } else {
            body.clone()
        };
        req_builder = req_builder.body(body);
    }

    Ok(req_builder)
}

async fn capture_response_details_async(
    request: &HttpRequest,
    verbose: bool,
    response: reqwest::Response,
) -> Result<(Option<HashMap<String, String>>, Option<String>)> {
    if should_capture_response(request, verbose) {
        let headers = Some(extract_headers(response.headers()));
        let body = response.text().await.ok();
        Ok((headers, body))
    } else {
        Ok((None, None))
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;
    use crate::types::Header;
    use serial_test::serial;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    fn create_test_request() -> HttpRequest {
        HttpRequest {
            name: Some("test_request".to_string()),
            method: "GET".to_string(),
            url: String::new(),
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

    fn spawn_ok_server() -> (u16, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let body = "OK";
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });
        (port, handle)
    }

    fn spawn_reset_server() -> (u16, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                drop(stream);
            }
        });
        (port, handle)
    }

    #[tokio::test]
    #[serial]
    async fn aaa_connection_reset_returns_error() {
        let (port, server) = spawn_reset_server();

        let mut request = create_test_request();
        request.url = format!("http://127.0.0.1:{}/", port);

        let result = execute_http_request_async(&request, false, false)
            .await
            .unwrap();

        server.join().ok();
        assert!(!result.success);
        assert!(result.error_message.is_some());
        assert_eq!(result.status_code, 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_successful_get_request() {
        let (port, server) = spawn_ok_server();
        let mut request = create_test_request();
        request.url = format!("http://127.0.0.1:{}/test", port);

        let result = execute_http_request_async(&request, false, false)
            .await
            .unwrap();

        server.join().ok();
        assert_eq!(result.status_code, 200);
        assert!(result.success);
        assert!(result.error_message.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn test_failed_connection_returns_error() {
        let mut request = create_test_request();
        request.url = "http://127.0.0.1:1/".to_string();
        request.connection_timeout = Some(500);

        let result = execute_http_request_async(&request, false, false)
            .await
            .unwrap();

        assert!(!result.success);
        assert!(result.error_message.is_some());
        assert_eq!(result.status_code, 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_verbose_mode_captures_headers_and_body() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let server = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nX-Custom: test-value\r\nContent-Length: 15\r\n\r\n{\"key\":\"value\"}";
                let _ = stream.write_all(response.as_bytes());
            }
        });

        let mut request = create_test_request();
        request.url = format!("http://127.0.0.1:{}/", port);

        let result = execute_http_request_async(&request, true, false)
            .await
            .unwrap();

        server.join().ok();
        assert_eq!(result.status_code, 200);
        assert!(result.success);

        let headers = result.response_headers.expect("should have headers in verbose mode");
        assert!(headers.contains_key("content-type"));
        assert!(headers.contains_key("x-custom"));

        let body = result.response_body.expect("should have body in verbose mode");
        assert_eq!(body, "{\"key\":\"value\"}");
    }

    #[tokio::test]
    #[serial]
    async fn test_non_verbose_mode_returns_empty_headers_and_body() {
        let (port, server) = spawn_ok_server();

        let mut request = create_test_request();
        request.name = None;
        request.url = format!("http://127.0.0.1:{}/", port);

        let result = execute_http_request_async(&request, false, false)
            .await
            .unwrap();

        server.join().ok();
        assert_eq!(result.status_code, 200);
        assert!(result.success);
        assert!(result.response_headers.is_none());
        assert!(result.response_body.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn test_client_caching_works() {
        {
            let mut request = create_test_request();
            request.url = "http://127.0.0.1:1/".to_string();
            request.timeout = Some(500);
            let client = build_client_async(&request, false);
            assert!(client.is_ok());
        }
        {
            let mut request = create_test_request();
            request.url = "http://127.0.0.1:2/".to_string();
            request.timeout = Some(9999);
            let client = build_client_async(&request, false);
            assert!(client.is_ok());
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_insecure_flag_passed_to_client_builder() {
        let mut request = create_test_request();
        request.url = "https://example.com".to_string();
        let client = build_client_async(&request, true);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_build_request_async_get() {
        let client = Client::new();
        let mut request = create_test_request();
        request.url = "http://127.0.0.1:1/".to_string();
        let result = build_request_async(&client, &request);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_request_async_post_with_body() {
        let client = Client::new();
        let mut request = create_test_request();
        request.url = "http://127.0.0.1:1/".to_string();
        request.method = "POST".to_string();
        request.body = Some("{\"key\":\"value\"}".to_string());
        let result = build_request_async(&client, &request);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_request_async_with_headers() {
        let client = Client::new();
        let mut request = create_test_request();
        request.url = "http://127.0.0.1:1/".to_string();
        request.headers.push(Header {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        });
        let result = build_request_async(&client, &request);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_request_async_put_method() {
        let client = Client::new();
        let mut request = create_test_request();
        request.url = "http://127.0.0.1:1/".to_string();
        request.method = "PUT".to_string();
        let result = build_request_async(&client, &request);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_request_async_delete_method() {
        let client = Client::new();
        let mut request = create_test_request();
        request.url = "http://127.0.0.1:1/".to_string();
        request.method = "DELETE".to_string();
        let result = build_request_async(&client, &request);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_request_async_patch_method() {
        let client = Client::new();
        let mut request = create_test_request();
        request.url = "http://127.0.0.1:1/".to_string();
        request.method = "PATCH".to_string();
        let result = build_request_async(&client, &request);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_request_async_head_method() {
        let client = Client::new();
        let mut request = create_test_request();
        request.url = "http://127.0.0.1:1/".to_string();
        request.method = "HEAD".to_string();
        let result = build_request_async(&client, &request);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_request_async_options_method() {
        let client = Client::new();
        let mut request = create_test_request();
        request.url = "http://127.0.0.1:1/".to_string();
        request.method = "OPTIONS".to_string();
        let result = build_request_async(&client, &request);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_request_async_lowercase_method() {
        let client = Client::new();
        let mut request = create_test_request();
        request.url = "http://127.0.0.1:1/".to_string();
        request.method = "get".to_string();
        let result = build_request_async(&client, &request);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_request_async_mixed_case_method() {
        let client = Client::new();
        let mut request = create_test_request();
        request.url = "http://127.0.0.1:1/".to_string();
        request.method = "PoSt".to_string();
        let result = build_request_async(&client, &request);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_request_async_invalid_method() {
        let client = Client::new();
        let mut request = create_test_request();
        request.url = "http://127.0.0.1:1/".to_string();
        request.method = "INVALID_METHOD_@#$".to_string();
        let result = build_request_async(&client, &request);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_build_request_async_empty_method() {
        let client = Client::new();
        let mut request = create_test_request();
        request.url = "http://127.0.0.1:1/".to_string();
        request.method = "".to_string();
        let result = build_request_async(&client, &request);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_build_request_async_with_multiple_headers() {
        let client = Client::new();
        let mut request = create_test_request();
        request.url = "http://127.0.0.1:1/".to_string();
        for i in 0..10 {
            request.headers.push(Header {
                name: format!("X-Custom-Header-{}", i),
                value: format!("value_{}", i),
            });
        }
        let result = build_request_async(&client, &request);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_request_async_post_without_body() {
        let client = Client::new();
        let mut request = create_test_request();
        request.url = "http://127.0.0.1:1/".to_string();
        request.method = "POST".to_string();
        request.body = None;
        let result = build_request_async(&client, &request);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_request_async_with_empty_body() {
        let client = Client::new();
        let mut request = create_test_request();
        request.url = "http://127.0.0.1:1/".to_string();
        request.method = "POST".to_string();
        request.body = Some(String::new());
        let result = build_request_async(&client, &request);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_request_async_with_large_body() {
        let client = Client::new();
        let mut request = create_test_request();
        request.url = "http://127.0.0.1:1/".to_string();
        request.method = "POST".to_string();
        request.body = Some("x".repeat(10000));
        let result = build_request_async(&client, &request);
        assert!(result.is_ok());
    }
}
