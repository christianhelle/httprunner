use crate::environment;
use crate::types::{Assertion, AssertionType, Condition, ConditionType, Header, HttpRequest, Variable};
use anyhow::{Context, Result};
use std::fs;

pub fn parse_http_file(
    file_path: &str,
    environment_name: Option<&str>,
) -> Result<Vec<HttpRequest>> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path))?;

    let mut requests = Vec::new();
    let env_variables = environment::load_environment_file(file_path, environment_name)?;
    let mut variables = env_variables.clone();

    let lines: Vec<&str> = content.lines().collect();
    let mut current_request: Option<HttpRequest> = None;
    let mut in_body = false;
    let mut body_content = String::new();
    let mut pending_request_name: Option<String> = None;
    let mut pending_timeout: Option<u64> = None;
    let mut pending_connection_timeout: Option<u64> = None;
    let mut pending_depends_on: Option<String> = None;
    let mut pending_conditions: Vec<Condition> = Vec::new();
    let mut in_intellij_script = false;

    for line in lines {
        let trimmed = line.trim();

        // Check for IntelliJ HTTP Client script blocks
        if line.trim_start().starts_with("> {%") {
            in_intellij_script = true;
            continue;
        }

        if in_intellij_script {
            if trimmed == "%}" || trimmed.ends_with("%}") {
                in_intellij_script = false;
            }
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        // Check for request name
        if trimmed.starts_with("# @name ") || trimmed.starts_with("// @name ") {
            let name_start = if trimmed.starts_with("# @name ") {
                8
            } else {
                9
            };
            pending_request_name = Some(trimmed[name_start..].trim().to_string());
            continue;
        }

        // Check for timeout directive
        let timeout_value = trimmed
            .strip_prefix("# @timeout ")
            .or_else(|| trimmed.strip_prefix("// @timeout "))
            .map(|s| s.trim());
        if let Some(value) = timeout_value {
            pending_timeout = parse_timeout_value(value);
            continue;
        }

        // Check for connection-timeout directive
        let connection_timeout_value = trimmed
            .strip_prefix("# @connection-timeout ")
            .or_else(|| trimmed.strip_prefix("// @connection-timeout "))
            .map(|s| s.trim());
        if let Some(value) = connection_timeout_value {
            pending_connection_timeout = parse_timeout_value(value);
            continue;
        }

        // Check for @dependsOn directive
        let depends_on_value = trimmed
            .strip_prefix("# @dependsOn ")
            .or_else(|| trimmed.strip_prefix("// @dependsOn "))
            .map(|s| s.trim());
        if let Some(value) = depends_on_value {
            pending_depends_on = Some(value.to_string());
            continue;
        }

        // Check for @if directive
        let if_value = trimmed
            .strip_prefix("# @if ")
            .or_else(|| trimmed.strip_prefix("// @if "))
            .map(|s| s.trim());
        if let Some(value) = if_value {
            if let Some(condition) = parse_condition(value) {
                pending_conditions.push(condition);
            }
            continue;
        }

        // Skip comments
        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        // Parse variables
        if trimmed.starts_with('@') {
            if let Some(eq_pos) = trimmed.find('=') {
                let var_name = trimmed[1..eq_pos].trim();
                let var_value = trimmed[eq_pos + 1..].trim();
                let substituted_value = substitute_variables(var_value, &variables);

                // Update or add variable
                if let Some(var) = variables.iter_mut().find(|v| v.name == var_name) {
                    var.value = substituted_value;
                } else {
                    variables.push(Variable {
                        name: var_name.to_string(),
                        value: substituted_value,
                    });
                }
            }
            continue;
        }

        // Parse assertions (with optional "> " prefix)
        let assertion_line = trimmed.strip_prefix("> ").unwrap_or(trimmed);

        if let Some(stripped) = assertion_line.strip_prefix("EXPECTED_RESPONSE_STATUS ") {
            if let Some(ref mut req) = current_request {
                let status_str = stripped.trim();
                req.assertions.push(Assertion {
                    assertion_type: AssertionType::Status,
                    expected_value: substitute_variables(status_str, &variables),
                });
            }
            continue;
        } else if let Some(stripped) = assertion_line.strip_prefix("EXPECTED_RESPONSE_BODY ") {
            if let Some(ref mut req) = current_request {
                let mut body_value = stripped.trim();
                if body_value.starts_with('"') && body_value.ends_with('"') && body_value.len() >= 2
                {
                    body_value = &body_value[1..body_value.len() - 1];
                }
                req.assertions.push(Assertion {
                    assertion_type: AssertionType::Body,
                    expected_value: substitute_variables(body_value, &variables),
                });
            }
            continue;
        } else if let Some(stripped) = assertion_line.strip_prefix("EXPECTED_RESPONSE_HEADERS ") {
            if let Some(ref mut req) = current_request {
                let mut headers_value = stripped.trim();
                if headers_value.starts_with('"')
                    && headers_value.ends_with('"')
                    && headers_value.len() >= 2
                {
                    headers_value = &headers_value[1..headers_value.len() - 1];
                }
                req.assertions.push(Assertion {
                    assertion_type: AssertionType::Headers,
                    expected_value: substitute_variables(headers_value, &variables),
                });
            }
            continue;
        }

        // Check if this is a new request line
        if is_http_request_line(trimmed) {
            // Save previous request if exists
            if let Some(mut req) = current_request.take() {
                if !body_content.is_empty() {
                    req.body = Some(substitute_variables(&body_content, &variables));
                }
                requests.push(req);
                body_content.clear();
            }

            // Parse new request
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let method = substitute_variables(parts[0], &variables);
                let url = substitute_variables(parts[1], &variables);

                current_request = Some(HttpRequest {
                    name: pending_request_name.take(),
                    method,
                    url,
                    headers: Vec::new(),
                    body: None,
                    assertions: Vec::new(),
                    variables: Vec::new(),
                    timeout: pending_timeout.take(),
                    connection_timeout: pending_connection_timeout.take(),
                    depends_on: pending_depends_on.take(),
                    conditions: std::mem::take(&mut pending_conditions),
                });
                in_body = false;
            }
        } else if trimmed.contains(':') && !in_body {
            // Parse header
            if let Some(ref mut req) = current_request {
                if let Some(colon_pos) = trimmed.find(':') {
                    let name = trimmed[..colon_pos].trim();
                    let value = trimmed[colon_pos + 1..].trim();

                    req.headers.push(Header {
                        name: substitute_variables(name, &variables),
                        value: substitute_variables(value, &variables),
                    });
                }
            }
        } else {
            // Body content
            in_body = true;
            if !body_content.is_empty() {
                body_content.push('\n');
            }
            body_content.push_str(trimmed);
        }
    }

    // Save last request if exists
    if let Some(mut req) = current_request {
        if !body_content.is_empty() {
            req.body = Some(substitute_variables(&body_content, &variables));
        }
        requests.push(req);
    }

    Ok(requests)
}

fn is_http_request_line(line: &str) -> bool {
    line.contains("HTTP/")
        || line.starts_with("GET ")
        || line.starts_with("POST ")
        || line.starts_with("PUT ")
        || line.starts_with("DELETE ")
        || line.starts_with("PATCH ")
}

fn substitute_variables(input: &str, variables: &[Variable]) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                chars.next(); // consume second '{'

                // Find closing }}
                let mut var_name = String::new();
                let mut found_closing = false;

                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        if chars.peek() == Some(&'}') {
                            chars.next(); // consume second '}'
                            found_closing = true;
                            break;
                        } else {
                            var_name.push(ch);
                        }
                    } else {
                        var_name.push(ch);
                    }
                }

                if found_closing {
                    // Look up variable
                    if let Some(var) = variables.iter().find(|v| v.name == var_name) {
                        result.push_str(&var.value);
                    } else {
                        // Variable not found, keep original
                        result.push_str("{{");
                        result.push_str(&var_name);
                        result.push_str("}}");
                    }
                } else {
                    // No closing found, keep original
                    result.push_str("{{");
                    result.push_str(&var_name);
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

fn parse_condition(value: &str) -> Option<Condition> {
    // Parse @if directive
    // Format 1: @if request-name.response.status 200
    // Format 2: @if request-name.response.body.$.property expected_value
    
    let parts: Vec<&str> = value.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    
    let reference = parts[0];
    let expected_value = parts[1..].join(" ");
    
    // Parse the reference to determine condition type
    // Format: request-name.response.status or request-name.response.body.$.property
    let ref_parts: Vec<&str> = reference.split('.').collect();
    
    if ref_parts.len() < 3 {
        return None;
    }
    
    let request_name = ref_parts[0].to_string();
    
    // Check if this is a status check
    if ref_parts.len() == 3 && ref_parts[1] == "response" && ref_parts[2] == "status" {
        return Some(Condition {
            request_name,
            condition_type: ConditionType::Status,
            expected_value,
        });
    }
    
    // Check if this is a body JSONPath check
    // Format: request-name.response.body.$.property
    if ref_parts.len() >= 4 && ref_parts[1] == "response" && ref_parts[2] == "body" {
        let json_path = ref_parts[3..].join(".");
        return Some(Condition {
            request_name,
            condition_type: ConditionType::BodyJsonPath(json_path),
            expected_value,
        });
    }
    
    None
}

fn parse_timeout_value(value: &str) -> Option<u64> {
    let value = value.trim();

    // Check for time unit suffix
    // Returns timeout in milliseconds
    if value.ends_with("ms") {
        // Milliseconds
        let num_str = value[..value.len() - 2].trim();
        num_str.parse::<u64>().ok()
    } else if value.ends_with('m') {
        // Minutes - convert to milliseconds
        let num_str = value[..value.len() - 1].trim();
        num_str.parse::<u64>().ok().and_then(|m| m.checked_mul(60_000))
    } else if value.ends_with('s') {
        // Seconds (explicit) - convert to milliseconds
        let num_str = value[..value.len() - 1].trim();
        num_str.parse::<u64>().ok().and_then(|s| s.checked_mul(1_000))
    } else {
        // No unit, default to seconds - convert to milliseconds
        value.parse::<u64>().ok().and_then(|s| s.checked_mul(1_000))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timeout_value_seconds_default() {
        assert_eq!(parse_timeout_value("30"), Some(30_000));
        assert_eq!(parse_timeout_value("100"), Some(100_000));
    }

    #[test]
    fn test_parse_timeout_value_seconds_explicit() {
        assert_eq!(parse_timeout_value("30 s"), Some(30_000));
        assert_eq!(parse_timeout_value("100s"), Some(100_000));
    }

    #[test]
    fn test_parse_timeout_value_minutes() {
        assert_eq!(parse_timeout_value("2 m"), Some(120_000));
        assert_eq!(parse_timeout_value("5m"), Some(300_000));
    }

    #[test]
    fn test_parse_timeout_value_milliseconds() {
        assert_eq!(parse_timeout_value("5000 ms"), Some(5_000));
        assert_eq!(parse_timeout_value("10000ms"), Some(10_000));
        assert_eq!(parse_timeout_value("999 ms"), Some(999));
        assert_eq!(parse_timeout_value("1500 ms"), Some(1_500));
    }

    #[test]
    fn test_parse_timeout_value_invalid() {
        assert_eq!(parse_timeout_value("invalid"), None);
        assert_eq!(parse_timeout_value(""), None);
    }

    #[test]
    fn test_parse_http_file_with_timeout() {
        let content = r#"
# @timeout 600
GET https://example.com/api
"#;
        // Create a temporary file for testing
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_timeout.http");
        std::fs::write(&test_file, content).unwrap();

        let requests = parse_http_file(test_file.to_str().unwrap(), None).unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].timeout, Some(600_000)); // 600 seconds = 600,000 ms
        assert_eq!(requests[0].connection_timeout, None);

        std::fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_parse_http_file_with_connection_timeout() {
        let content = r#"
// @connection-timeout 2 m
GET https://example.com/api
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_connection_timeout.http");
        std::fs::write(&test_file, content).unwrap();

        let requests = parse_http_file(test_file.to_str().unwrap(), None).unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].timeout, None);
        assert_eq!(requests[0].connection_timeout, Some(120_000)); // 2 minutes = 120,000 ms

        std::fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_parse_http_file_with_both_timeouts() {
        let content = r#"
# @timeout 600
// @connection-timeout 30
GET https://example.com/api
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_both_timeouts.http");
        std::fs::write(&test_file, content).unwrap();

        let requests = parse_http_file(test_file.to_str().unwrap(), None).unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].timeout, Some(600_000)); // 600 seconds = 600,000 ms
        assert_eq!(requests[0].connection_timeout, Some(30_000)); // 30 seconds = 30,000 ms

        std::fs::remove_file(&test_file).ok();
    }
}
