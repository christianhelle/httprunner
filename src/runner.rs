use crate::assertions;
use crate::types::{HttpRequest, HttpResult};
use anyhow::{anyhow, Result};
use native_tls::TlsConnector;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

pub fn execute_http_request(request: &HttpRequest, verbose: bool) -> Result<HttpResult> {
    let start_time = Instant::now();
    let has_assertions = !request.assertions.is_empty();

    // Parse URL
    let url = &request.url;
    let is_https = url.starts_with("https://");
    let is_http = url.starts_with("http://");

    if !is_http && !is_https {
        return Ok(HttpResult {
            request_name: request.name.clone(),
            status_code: 0,
            success: false,
            error_message: Some("Invalid URL scheme".to_string()),
            duration_ms: start_time.elapsed().as_millis() as u64,
            response_headers: None,
            response_body: None,
            assertion_results: Vec::new(),
        });
    }

    let url_without_scheme = if is_https { &url[8..] } else { &url[7..] };

    let (host, path) = if let Some(slash_pos) = url_without_scheme.find('/') {
        let host = &url_without_scheme[..slash_pos];
        let path = &url_without_scheme[slash_pos..];
        (host, path)
    } else {
        (url_without_scheme, "/")
    };

    let (hostname, port) = if let Some(colon_pos) = host.rfind(':') {
        let hostname = &host[..colon_pos];
        let port_str = &host[colon_pos + 1..];
        let port = port_str
            .parse::<u16>()
            .unwrap_or(if is_https { 443 } else { 80 });
        (hostname, port)
    } else {
        (host, if is_https { 443 } else { 80 })
    };

    // Execute request
    let response = match send_http_request(
        &request.method,
        hostname,
        port,
        path,
        &request.headers,
        &request.body,
        is_https,
    ) {
        Ok(resp) => resp,
        Err(e) => {
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let error_message = format!("Request failed: {}", e);

            return Ok(HttpResult {
                request_name: request.name.clone(),
                status_code: 0,
                success: false,
                error_message: Some(error_message),
                duration_ms,
                response_headers: None,
                response_body: None,
                assertion_results: Vec::new(),
            });
        }
    };

    let status_code = response.status_code;
    let mut success = (200..300).contains(&status_code);

    let mut response_headers: Option<HashMap<String, String>> = None;
    let mut response_body: Option<String> = None;

    if verbose || has_assertions {
        response_headers = Some(response.headers);
        response_body = Some(response.body);
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

struct HttpResponse {
    status_code: u16,
    headers: HashMap<String, String>,
    body: String,
}

fn send_http_request(
    method: &str,
    hostname: &str,
    port: u16,
    path: &str,
    headers: &[crate::types::Header],
    body: &Option<String>,
    is_https: bool,
) -> Result<HttpResponse> {
    let addr = format!("{}:{}", hostname, port);
    let stream = TcpStream::connect_timeout(
        &std::net::ToSocketAddrs::to_socket_addrs(&addr.as_str())?
            .next()
            .ok_or_else(|| anyhow!("Failed to resolve hostname"))?,
        Duration::from_secs(30),
    )?;
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;

    let mut stream: Box<dyn ReadWrite> = if is_https {
        let connector = TlsConnector::new()?;
        let tls_stream = connector.connect(hostname, stream)?;
        Box::new(tls_stream)
    } else {
        Box::new(stream)
    };

    // Build HTTP request
    let mut request = format!("{} {} HTTP/1.1\r\n", method.to_uppercase(), path);
    request.push_str(&format!("Host: {}\r\n", hostname));
    request.push_str("Connection: close\r\n");

    let mut has_content_length = false;
    let mut has_user_agent = false;

    for header in headers {
        request.push_str(&format!("{}: {}\r\n", header.name, header.value));
        if header.name.eq_ignore_ascii_case("content-length") {
            has_content_length = true;
        }
        if header.name.eq_ignore_ascii_case("user-agent") {
            has_user_agent = true;
        }
    }

    if !has_user_agent {
        request.push_str("User-Agent: httprunner\r\n");
    }

    if let Some(body_content) = body {
        if !has_content_length {
            request.push_str(&format!("Content-Length: {}\r\n", body_content.len()));
        }
        request.push_str("\r\n");
        request.push_str(body_content);
    } else {
        request.push_str("\r\n");
    }

    // Send request
    stream.write_all(request.as_bytes())?;
    stream.flush()?;

    // Read response
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer)?;

    let response_str = String::from_utf8_lossy(&buffer);

    // Parse response
    parse_http_response(&response_str)
}

fn parse_http_response(response_str: &str) -> Result<HttpResponse> {
    // Prefer CRLF per RFC; fall back to LF for robustness
    let (head, body) = if let Some(i) = response_str.find("\r\n\r\n") {
        (&response_str[..i], &response_str[i + 4..])
    } else if let Some(i) = response_str.find("\n\n") {
        (&response_str[..i], &response_str[i + 2..])
    } else {
        return Err(anyhow!("Invalid HTTP response: missing header terminator"));
    };

    let mut head_lines = head.lines();
    let status_line = head_lines.next().ok_or_else(|| anyhow!("Empty response"))?;
    let parts: Vec<&str> = status_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(anyhow!("Invalid status line"));
    }
    let status_code = parts[1]
        .parse::<u16>()
        .map_err(|_| anyhow!("Invalid status code"))?;

    let mut headers = HashMap::new();
    for line in head_lines {
        if let Some(colon_pos) = line.find(':') {
            let name = line[..colon_pos].trim().to_string();
            let value = line[colon_pos + 1..].trim().to_string();
            headers.insert(name, value);
        }
    }

    Ok(HttpResponse {
        status_code,
        headers,
        body: body.to_string(),
    })
}

trait ReadWrite: Read + Write {}
impl<T: Read + Write> ReadWrite for T {}
