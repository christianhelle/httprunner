use std::fs;
use std::path::Path;
use anyhow::{Result, anyhow};
use crate::types::HttpRequest;

pub fn parse_http_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<HttpRequest>> {
    let content = fs::read_to_string(&file_path)
        .map_err(|e| anyhow!("Failed to read file '{}': {}", file_path.as_ref().display(), e))?;
    
    parse_http_content(&content)
}

pub fn parse_http_content(content: &str) -> Result<Vec<HttpRequest>> {
    let mut requests = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    
    let mut current_request: Option<HttpRequest> = None;
    let mut in_body = false;
    let mut body_content = Vec::new();
    
    for line in lines {
        let trimmed = line.trim();
        
        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            if !trimmed.is_empty() || in_body {
                // If we're already in body mode, empty lines are part of body
                if in_body {
                    body_content.push(trimmed.to_string());
                }
            } else {
                // Empty line after headers transitions to body mode
                in_body = true;
            }
            continue;
        }
        
        // Check for HTTP request line (method + URL)
        if is_request_line(trimmed) {
            // Save previous request if exists
            if let Some(mut req) = current_request.take() {
                if !body_content.is_empty() {
                    req.set_body(body_content.join("\n"));
                }
                requests.push(req);
                body_content.clear();
            }
            
            // Parse new request
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() < 2 {
                return Err(anyhow!("Invalid request line: {}", trimmed));
            }
            
            let method = parts[0].to_string();
            let url = parts[1].to_string();
            
            current_request = Some(HttpRequest::new(method, url));
            in_body = false;
        }
        // Check for header line (contains ':')
        else if trimmed.contains(':') && !in_body {
            if let Some(ref mut req) = current_request {
                let colon_pos = trimmed.find(':').unwrap();
                let name = trimmed[..colon_pos].trim().to_string();
                let value = trimmed[colon_pos + 1..].trim().to_string();
                req.add_header(name, value);
            }
        }
        // Body content
        else {
            in_body = true;
            body_content.push(trimmed.to_string());
        }
    }
    
    // Save last request if exists
    if let Some(mut req) = current_request {
        if !body_content.is_empty() {
            req.set_body(body_content.join("\n"));
        }
        requests.push(req);
    }
    
    Ok(requests)
}

fn is_request_line(line: &str) -> bool {
    let methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
    
    for method in &methods {
        if line.starts_with(method) && line.len() > method.len() {
            let next_char = line.chars().nth(method.len()).unwrap();
            if next_char.is_whitespace() {
                return true;
            }
        }
    }
    
    // Also check for HTTP response lines (though we mainly process requests)
    line.contains("HTTP/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_get_request() {
        let content = "GET https://httpbin.org/status/200";
        let requests = parse_http_content(content).unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, "GET");
        assert_eq!(requests[0].url, "https://httpbin.org/status/200");
        assert!(requests[0].headers.is_empty());
        assert!(requests[0].body.is_none());
    }

    #[test]
    fn test_parse_request_with_headers() {
        let content = r#"POST https://httpbin.org/post
Content-Type: application/json
Authorization: Bearer token123

{"test": "data"}"#;
        
        let requests = parse_http_content(content).unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, "POST");
        assert_eq!(requests[0].url, "https://httpbin.org/post");
        assert_eq!(requests[0].headers.len(), 2);
        assert_eq!(requests[0].headers[0].name, "Content-Type");
        assert_eq!(requests[0].headers[0].value, "application/json");
        assert_eq!(requests[0].headers[1].name, "Authorization");
        assert_eq!(requests[0].headers[1].value, "Bearer token123");
        assert_eq!(requests[0].body.as_ref().unwrap(), r#"{"test": "data"}"#);
    }

    #[test]
    fn test_parse_multiple_requests() {
        let content = r#"# First request
GET https://httpbin.org/status/200

# Second request  
POST https://httpbin.org/post
Content-Type: application/json

{"data": "test"}"#;
        
        let requests = parse_http_content(content).unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].method, "GET");
        assert_eq!(requests[1].method, "POST");
    }
}