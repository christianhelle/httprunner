use crate::types::{Assertion, AssertionType, Condition, HttpRequest};
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Serialize a single HttpRequest to .http format string
pub fn serialize_http_request(request: &HttpRequest) -> String {
    let mut output = String::new();

    // Add separator
    output.push_str("###\n");

    // Add name if present
    if let Some(name) = &request.name {
        output.push_str(&format!("# @name {}\n", name));
    }

    // Add timeout if present
    if let Some(timeout) = request.timeout {
        output.push_str(&format!("# @timeout {}ms\n", timeout));
    }

    // Add connection timeout if present
    if let Some(connection_timeout) = request.connection_timeout {
        output.push_str(&format!("# @connection-timeout {}ms\n", connection_timeout));
    }

    // Add dependsOn if present
    if let Some(depends_on) = &request.depends_on {
        output.push_str(&format!("# @dependsOn {}\n", depends_on));
    }

    // Add conditions if present
    for condition in &request.conditions {
        output.push_str(&format!("# @if {}\n", format_condition(condition)));
    }

    // Add assertions if present
    for assertion in &request.assertions {
        output.push_str(&format!("# @assert {}\n", format_assertion(assertion)));
    }

    // Add request line (method and URL)
    output.push_str(&format!("{} {}\n", request.method, request.url));

    // Add headers
    for header in &request.headers {
        output.push_str(&format!("{}: {}\n", header.name, header.value));
    }

    // Add body if present
    if let Some(body) = &request.body {
        output.push('\n');
        output.push_str(body);
        if !body.ends_with('\n') {
            output.push('\n');
        }
    }

    output
}

/// Serialize multiple HttpRequests to .http format
pub fn serialize_http_requests(requests: &[HttpRequest]) -> String {
    requests
        .iter()
        .map(serialize_http_request)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Write HttpRequests to a .http file
pub fn write_http_file(path: &Path, requests: &[HttpRequest]) -> Result<()> {
    let content = serialize_http_requests(requests);
    fs::write(path, content)?;
    Ok(())
}

fn format_condition(condition: &Condition) -> String {
    use crate::types::ConditionType;
    
    match &condition.condition_type {
        ConditionType::BodyJsonPath(jsonpath) => {
            format!(
                "{}.response.body.${} {}",
                condition.request_name, jsonpath, condition.expected_value
            )
        }
        ConditionType::Status => {
            format!(
                "{}.response.status {}",
                condition.request_name, condition.expected_value
            )
        }
    }
}

fn format_assertion(assertion: &Assertion) -> String {
    match assertion.assertion_type {
        AssertionType::Status => format!("status {}", assertion.expected_value),
        AssertionType::Body => format!("body {}", assertion.expected_value),
        AssertionType::Headers => format!("headers {}", assertion.expected_value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Header;

    #[test]
    fn test_serialize_simple_request() {
        let request = HttpRequest {
            name: None,
            method: "GET".to_string(),
            url: "https://httpbin.org/get".to_string(),
            headers: vec![],
            body: None,
            assertions: vec![],
            variables: vec![],
            timeout: None,
            connection_timeout: None,
            depends_on: None,
            conditions: vec![],
        };

        let serialized = serialize_http_request(&request);
        assert!(serialized.contains("###"));
        assert!(serialized.contains("GET https://httpbin.org/get"));
    }

    #[test]
    fn test_serialize_request_with_name() {
        let request = HttpRequest {
            name: Some("test-request".to_string()),
            method: "POST".to_string(),
            url: "https://httpbin.org/post".to_string(),
            headers: vec![],
            body: None,
            assertions: vec![],
            variables: vec![],
            timeout: None,
            connection_timeout: None,
            depends_on: None,
            conditions: vec![],
        };

        let serialized = serialize_http_request(&request);
        assert!(serialized.contains("# @name test-request"));
    }

    #[test]
    fn test_serialize_request_with_headers_and_body() {
        let request = HttpRequest {
            name: None,
            method: "POST".to_string(),
            url: "https://httpbin.org/post".to_string(),
            headers: vec![
                Header {
                    name: "Content-Type".to_string(),
                    value: "application/json".to_string(),
                },
                Header {
                    name: "Authorization".to_string(),
                    value: "Bearer token123".to_string(),
                },
            ],
            body: Some(r#"{"key": "value"}"#.to_string()),
            assertions: vec![],
            variables: vec![],
            timeout: None,
            connection_timeout: None,
            depends_on: None,
            conditions: vec![],
        };

        let serialized = serialize_http_request(&request);
        assert!(serialized.contains("Content-Type: application/json"));
        assert!(serialized.contains("Authorization: Bearer token123"));
        assert!(serialized.contains(r#"{"key": "value"}"#));
    }
}
