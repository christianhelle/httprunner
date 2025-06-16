use reqwest::{Client, Method};
use std::time::Instant;
use anyhow::Result;
use crate::types::{HttpRequest, HttpResult, Header};

pub async fn execute_http_request(request: &HttpRequest, verbose: bool) -> Result<HttpResult> {
    let start_time = Instant::now();
    
    // Parse URL
    let url = match url::Url::parse(&request.url) {
        Ok(url) => url,
        Err(_) => {
            let duration_ms = start_time.elapsed().as_millis() as u64;
            return Ok(HttpResult::with_error("Invalid URL".to_string(), duration_ms));
        }
    };
    
    // Parse HTTP method
    let method = match request.method.parse::<Method>() {
        Ok(method) => method,
        Err(_) => {
            let duration_ms = start_time.elapsed().as_millis() as u64;
            return Ok(HttpResult::with_error(
                format!("Invalid HTTP method: {}", request.method),
                duration_ms
            ));
        }
    };
    
    // Create HTTP client
    let client = Client::new();
    let mut req_builder = client.request(method, url);
    
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
        Ok(response) => response,
        Err(e) => {
            let duration_ms = start_time.elapsed().as_millis() as u64;
            return Ok(HttpResult::with_error(e.to_string(), duration_ms));
        }
    };
    
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let status_code = response.status().as_u16();
    let mut result = HttpResult::new(status_code, duration_ms);
    
    // Collect response headers if verbose
    if verbose {
        let headers: Vec<Header> = response.headers()
            .iter()
            .map(|(name, value)| Header {
                name: name.to_string(),
                value: value.to_str().unwrap_or("").to_string(),
            })
            .collect();
        result.with_headers(headers);
        
        // Get response body
        match response.text().await {
            Ok(body) => result.with_body(body),
            Err(e) => {
                result.error_message = Some(format!("Failed to read response body: {}", e));
            }
        }
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::HttpRequest;

    #[tokio::test]
    #[ignore] // Skip network tests in CI/sandbox environments
    async fn test_execute_simple_get_request() {
        let request = HttpRequest::new("GET".to_string(), "https://httpbin.org/status/200".to_string());
        let result = execute_http_request(&request, false).await.unwrap();
        
        assert_eq!(result.status_code, 200);
        assert!(result.success);
        assert!(result.error_message.is_none());
        assert!(result.duration_ms > 0);
    }

    #[tokio::test]
    async fn test_execute_invalid_url() {
        let request = HttpRequest::new("GET".to_string(), "invalid-url".to_string());
        let result = execute_http_request(&request, false).await.unwrap();
        
        assert_eq!(result.status_code, 0);
        assert!(!result.success);
        assert!(result.error_message.is_some());
        assert_eq!(result.error_message.unwrap(), "Invalid URL");
    }

    #[tokio::test]
    #[ignore] // Skip network tests in CI/sandbox environments
    async fn test_execute_with_verbose() {
        let request = HttpRequest::new("GET".to_string(), "https://httpbin.org/status/200".to_string());
        let result = execute_http_request(&request, true).await.unwrap();
        
        assert_eq!(result.status_code, 200);
        assert!(result.success);
        assert!(result.response_headers.is_some());
        assert!(result.response_body.is_some());
    }
}