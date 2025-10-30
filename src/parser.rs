use crate::environment;
use crate::types::{Assertion, AssertionType, Header, HttpRequest, Variable};
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
