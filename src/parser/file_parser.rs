use super::condition_parser::parse_condition;
use super::timeout_parser::parse_timeout_value;
use super::utils::is_http_request_line;
use super::variable_substitution::substitute_variables;
use crate::environment;
use crate::types::{Assertion, AssertionType, Condition, Header, HttpRequest, Variable};
use crate::error::{Result, Context};
use std::fs;

pub fn parse_http_file(
    file_path: &str,
    environment_name: Option<&str>,
) -> Result<Vec<HttpRequest>> {
    let content = fs::read_to_string(file_path)
        .context(format!("Failed to read file: {}", file_path))?;

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
            if in_body {
                body_content.push('\n');
            }
            continue;
        }

        if trimmed.starts_with("# @name ") || trimmed.starts_with("// @name ") {
            let name_start = if trimmed.starts_with("# @name ") {
                8
            } else {
                9
            };
            pending_request_name = Some(trimmed[name_start..].trim().to_string());
            continue;
        }

        let timeout_value = trimmed
            .strip_prefix("# @timeout ")
            .or_else(|| trimmed.strip_prefix("// @timeout "))
            .map(|s| s.trim());
        if let Some(value) = timeout_value {
            pending_timeout = parse_timeout_value(value);
            continue;
        }

        let connection_timeout_value = trimmed
            .strip_prefix("# @connection-timeout ")
            .or_else(|| trimmed.strip_prefix("// @connection-timeout "))
            .map(|s| s.trim());
        if let Some(value) = connection_timeout_value {
            pending_connection_timeout = parse_timeout_value(value);
            continue;
        }

        let depends_on_value = trimmed
            .strip_prefix("# @dependsOn ")
            .or_else(|| trimmed.strip_prefix("// @dependsOn "))
            .map(|s| s.trim());
        if let Some(value) = depends_on_value {
            pending_depends_on = Some(value.to_string());
            continue;
        }

        let if_value = trimmed
            .strip_prefix("# @if ")
            .or_else(|| trimmed.strip_prefix("// @if "))
            .map(|s| s.trim());
        if let Some(value) = if_value {
            match parse_condition(value, false) {
                Some(condition) => pending_conditions.push(condition),
                None => eprintln!("Warning: Invalid @if directive format: '{}'", value),
            }
            continue;
        }

        let if_not_value = trimmed
            .strip_prefix("# @if-not ")
            .or_else(|| trimmed.strip_prefix("// @if-not "))
            .map(|s| s.trim());
        if let Some(value) = if_not_value {
            match parse_condition(value, true) {
                Some(condition) => pending_conditions.push(condition),
                None => eprintln!("Warning: Invalid @if-not directive format: '{}'", value),
            }
            continue;
        }

        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with('@') {
            if let Some(eq_pos) = trimmed.find('=') {
                let var_name = trimmed[1..eq_pos].trim();
                let var_value = trimmed[eq_pos + 1..].trim();
                let substituted_value = substitute_variables(var_value, &variables);

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

        if is_http_request_line(trimmed) {
            if let Some(mut req) = current_request.take() {
                if !body_content.is_empty() {
                    req.body = Some(substitute_variables(&body_content, &variables));
                }
                requests.push(req);
                body_content.clear();
            }

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
            if let Some(ref mut req) = current_request
                && let Some(colon_pos) = trimmed.find(':') {
                    let name = trimmed[..colon_pos].trim();
                    let value = trimmed[colon_pos + 1..].trim();

                    req.headers.push(Header {
                        name: substitute_variables(name, &variables),
                        value: substitute_variables(value, &variables),
                    });
                }
        } else {
            in_body = true;
            if !body_content.is_empty() {
                body_content.push('\n');
            }
            body_content.push_str(trimmed);
        }
    }

    if let Some(mut req) = current_request {
        if !body_content.is_empty() {
            req.body = Some(substitute_variables(&body_content, &variables));
        }
        requests.push(req);
    }

    Ok(requests)
}
