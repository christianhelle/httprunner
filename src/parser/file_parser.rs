use super::condition_parser::parse_condition;
use super::timeout_parser::parse_timeout_value;
use super::utils::is_http_request_line;
use super::variable_substitution::substitute_variables;
use crate::environment;
use crate::types::{Assertion, AssertionType, Condition, Header, HttpRequest, Variable};
use anyhow::{Context, Result};
use std::fs;

/// State tracking for parser while processing .http files
struct ParserState {
    current_request: Option<HttpRequest>,
    in_body: bool,
    body_content: String,
    pending_request_name: Option<String>,
    pending_timeout: Option<u64>,
    pending_connection_timeout: Option<u64>,
    pending_depends_on: Option<String>,
    pending_conditions: Vec<Condition>,
    in_intellij_script: bool,
}

impl ParserState {
    fn new() -> Self {
        Self {
            current_request: None,
            in_body: false,
            body_content: String::new(),
            pending_request_name: None,
            pending_timeout: None,
            pending_connection_timeout: None,
            pending_depends_on: None,
            pending_conditions: Vec::new(),
            in_intellij_script: false,
        }
    }

    /// Finalize the current request and add it to the requests list
    fn finalize_current_request(
        &mut self,
        requests: &mut Vec<HttpRequest>,
        variables: &[Variable],
    ) {
        if let Some(mut req) = self.current_request.take() {
            if !self.body_content.is_empty() {
                req.body = Some(substitute_variables(&self.body_content, variables));
            }
            requests.push(req);
            self.body_content.clear();
        }
    }

    /// Create a new HTTP request from the parsed line
    fn create_request(&mut self, method: String, url: String) {
        self.current_request = Some(HttpRequest {
            name: self.pending_request_name.take(),
            method,
            url,
            headers: Vec::new(),
            body: None,
            assertions: Vec::new(),
            variables: Vec::new(),
            timeout: self.pending_timeout.take(),
            connection_timeout: self.pending_connection_timeout.take(),
            depends_on: self.pending_depends_on.take(),
            conditions: std::mem::take(&mut self.pending_conditions),
        });
        self.in_body = false;
    }
}

/// Extract a directive value from a line with comment prefix (# or //)
fn extract_directive<'a>(line: &'a str, directive: &str) -> Option<&'a str> {
    line.strip_prefix(&format!("# {}", directive))
        .or_else(|| line.strip_prefix(&format!("// {}", directive)))
        .map(|s| s.trim())
}

/// Remove surrounding quotes from a string if present
fn strip_quotes(s: &str) -> &str {
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Parse and add an assertion to the current request
fn parse_assertion(
    current_request: &mut Option<HttpRequest>,
    assertion_line: &str,
    variables: &[Variable],
) -> bool {
    if let Some(stripped) = assertion_line.strip_prefix("EXPECTED_RESPONSE_STATUS ") {
        if let Some(req) = current_request {
            let status_str = stripped.trim();
            req.assertions.push(Assertion {
                assertion_type: AssertionType::Status,
                expected_value: substitute_variables(status_str, variables),
            });
        }
        return true;
    }

    if let Some(stripped) = assertion_line.strip_prefix("EXPECTED_RESPONSE_BODY ") {
        if let Some(req) = current_request {
            let body_value = strip_quotes(stripped.trim());
            req.assertions.push(Assertion {
                assertion_type: AssertionType::Body,
                expected_value: substitute_variables(body_value, variables),
            });
        }
        return true;
    }

    if let Some(stripped) = assertion_line.strip_prefix("EXPECTED_RESPONSE_HEADERS ") {
        if let Some(req) = current_request {
            let headers_value = strip_quotes(stripped.trim());
            req.assertions.push(Assertion {
                assertion_type: AssertionType::Headers,
                expected_value: substitute_variables(headers_value, variables),
            });
        }
        return true;
    }

    false
}

/// Process a variable assignment line (@variable = value)
fn process_variable_assignment(line: &str, variables: &mut Vec<Variable>) {
    if let Some(eq_pos) = line.find('=') {
        let var_name = line[1..eq_pos].trim();
        let var_value = line[eq_pos + 1..].trim();
        let substituted_value = substitute_variables(var_value, variables);

        if let Some(var) = variables.iter_mut().find(|v| v.name == var_name) {
            var.value = substituted_value;
        } else {
            variables.push(Variable {
                name: var_name.to_string(),
                value: substituted_value,
            });
        }
    }
}

pub fn parse_http_file(
    file_path: &str,
    environment_name: Option<&str>,
) -> Result<Vec<HttpRequest>> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path))?;

    let mut requests = Vec::new();
    let env_variables = environment::load_environment_file(file_path, environment_name)?;
    let mut variables = env_variables.clone();
    let mut state = ParserState::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Handle IntelliJ script blocks
        if line.trim_start().starts_with("> {%") {
            state.in_intellij_script = true;
            continue;
        }
        if state.in_intellij_script {
            if trimmed == "%}" || trimmed.ends_with("%}") {
                state.in_intellij_script = false;
            }
            continue;
        }

        // Handle empty lines
        if trimmed.is_empty() {
            if state.in_body {
                state.body_content.push('\n');
            }
            continue;
        }

        // Parse @name directive
        if trimmed.starts_with("# @name ") || trimmed.starts_with("// @name ") {
            let name_start = if trimmed.starts_with("# @name ") {
                8
            } else {
                9
            };
            state.pending_request_name = Some(trimmed[name_start..].trim().to_string());
            continue;
        }

        // Parse @timeout directive
        if let Some(value) = extract_directive(trimmed, "@timeout") {
            state.pending_timeout = parse_timeout_value(value);
            continue;
        }

        // Parse @connection-timeout directive
        if let Some(value) = extract_directive(trimmed, "@connection-timeout") {
            state.pending_connection_timeout = parse_timeout_value(value);
            continue;
        }

        // Parse @dependsOn directive
        if let Some(value) = extract_directive(trimmed, "@dependsOn") {
            state.pending_depends_on = Some(value.to_string());
            continue;
        }

        // Parse @if directive
        if let Some(value) = extract_directive(trimmed, "@if ") {
            match parse_condition(value, false) {
                Some(condition) => state.pending_conditions.push(condition),
                None => eprintln!("Warning: Invalid @if directive format: '{}'", value),
            }
            continue;
        }

        // Parse @if-not directive
        if let Some(value) = extract_directive(trimmed, "@if-not ") {
            match parse_condition(value, true) {
                Some(condition) => state.pending_conditions.push(condition),
                None => eprintln!("Warning: Invalid @if-not directive format: '{}'", value),
            }
            continue;
        }

        // Skip comments
        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        // Process variable assignments
        if trimmed.starts_with('@') {
            process_variable_assignment(trimmed, &mut variables);
            continue;
        }

        // Parse assertions
        let assertion_line = trimmed.strip_prefix("> ").unwrap_or(trimmed);
        if parse_assertion(&mut state.current_request, assertion_line, &variables) {
            continue;
        }

        // Parse HTTP request line
        if is_http_request_line(trimmed) {
            state.finalize_current_request(&mut requests, &variables);

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let method = substitute_variables(parts[0], &variables);
                let url = substitute_variables(parts[1], &variables);
                state.create_request(method, url);
            }
        } else if trimmed.contains(':') && !state.in_body {
            // Parse headers
            if let Some(ref mut req) = state.current_request
                && let Some(colon_pos) = trimmed.find(':')
            {
                let name = trimmed[..colon_pos].trim();
                let value = trimmed[colon_pos + 1..].trim();

                req.headers.push(Header {
                    name: substitute_variables(name, &variables),
                    value: substitute_variables(value, &variables),
                });
            }
        } else {
            // Parse body content
            state.in_body = true;
            if !state.body_content.is_empty() {
                state.body_content.push('\n');
            }
            state.body_content.push_str(trimmed);
        }
    }

    // Finalize the last request
    state.finalize_current_request(&mut requests, &variables);

    Ok(requests)
}
