use super::condition_parser::parse_condition;
use super::substitution::*;
use super::timeout_parser::parse_timeout_value;
use super::utils::is_http_request_line;
use crate::environment;
use crate::types::{Assertion, AssertionType, Condition, Header, HttpRequest, Variable};
use anyhow::{Context, Result};
use std::fs;

/// Parses an HTTP file from disk and returns a list of HTTP requests.
pub fn parse_http_file(
    file_path: &str,
    environment_name: Option<&str>,
) -> Result<Vec<HttpRequest>> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path))?;

    let env_variables = environment::load_environment_file(file_path, environment_name)?;
    parse_http_content_with_vars(&content, env_variables)
}

/// Parses HTTP content from a string (without file-based environment loading).
pub fn parse_http_content(
    content: &str,
    _environment_name: Option<&str>,
) -> Result<Vec<HttpRequest>> {
    let env_variables = Vec::new();
    parse_http_content_with_vars(content, env_variables)
}

/// State tracked during parsing of an HTTP file.
struct ParserState {
    requests: Vec<HttpRequest>,
    variables: Vec<Variable>,
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
    fn new(env_variables: Vec<Variable>) -> Self {
        Self {
            requests: Vec::new(),
            variables: env_variables,
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

    /// Finalize the current request and add it to the requests list.
    fn finalize_current_request(&mut self) {
        if let Some(mut req) = self.current_request.take() {
            if !self.body_content.is_empty() {
                req.body = Some(substitute_variables(&self.body_content, &self.variables));
            }
            self.requests.push(req);
            self.body_content.clear();
        }
    }

    /// Start a new HTTP request with the given method and URL.
    fn start_new_request(&mut self, method: String, url: String) {
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

    /// Add a header to the current request.
    fn add_header(&mut self, name: &str, value: &str) {
        if let Some(ref mut req) = self.current_request {
            req.headers.push(Header {
                name: substitute_variables(name, &self.variables),
                value: substitute_variables(value, &self.variables),
            });
        }
    }

    /// Add an assertion to the current request.
    fn add_assertion(&mut self, assertion_type: AssertionType, expected_value: &str) {
        if let Some(ref mut req) = self.current_request {
            req.assertions.push(Assertion {
                assertion_type,
                expected_value: substitute_variables(expected_value, &self.variables),
            });
        }
    }

    /// Append content to the request body.
    fn append_body_content(&mut self, content: &str) {
        self.in_body = true;
        if !self.body_content.is_empty() {
            self.body_content.push('\n');
        }
        self.body_content.push_str(content);
    }

    /// Update or add a variable.
    fn set_variable(&mut self, name: &str, value: &str) {
        let substituted_value = substitute_variables(value, &self.variables);
        if let Some(var) = self.variables.iter_mut().find(|v| v.name == name) {
            var.value = substituted_value;
        } else {
            self.variables.push(Variable {
                name: name.to_string(),
                value: substituted_value,
            });
        }
    }
}

/// Result of parsing a single line.
enum LineParseResult {
    /// Line was handled, continue to next line
    Continue,
    /// Line was not a directive, needs further processing
    NotHandled,
}

/// Try to parse a @name directive.
fn try_parse_name_directive(trimmed: &str, state: &mut ParserState) -> LineParseResult {
    if trimmed.starts_with("# @name ") {
        state.pending_request_name = Some(trimmed[8..].trim().to_string());
        LineParseResult::Continue
    } else if trimmed.starts_with("// @name ") {
        state.pending_request_name = Some(trimmed[9..].trim().to_string());
        LineParseResult::Continue
    } else {
        LineParseResult::NotHandled
    }
}

/// Try to parse a @timeout directive.
fn try_parse_timeout_directive(trimmed: &str, state: &mut ParserState) -> LineParseResult {
    let timeout_value = trimmed
        .strip_prefix("# @timeout ")
        .or_else(|| trimmed.strip_prefix("// @timeout "))
        .map(|s| s.trim());

    if let Some(value) = timeout_value {
        state.pending_timeout = parse_timeout_value(value);
        LineParseResult::Continue
    } else {
        LineParseResult::NotHandled
    }
}

/// Try to parse a @connection-timeout directive.
fn try_parse_connection_timeout_directive(
    trimmed: &str,
    state: &mut ParserState,
) -> LineParseResult {
    let timeout_value = trimmed
        .strip_prefix("# @connection-timeout ")
        .or_else(|| trimmed.strip_prefix("// @connection-timeout "))
        .map(|s| s.trim());

    if let Some(value) = timeout_value {
        state.pending_connection_timeout = parse_timeout_value(value);
        LineParseResult::Continue
    } else {
        LineParseResult::NotHandled
    }
}

/// Try to parse a @dependsOn directive.
fn try_parse_depends_on_directive(trimmed: &str, state: &mut ParserState) -> LineParseResult {
    let depends_on_value = trimmed
        .strip_prefix("# @dependsOn ")
        .or_else(|| trimmed.strip_prefix("// @dependsOn "))
        .map(|s| s.trim());

    if let Some(value) = depends_on_value {
        state.pending_depends_on = Some(value.to_string());
        LineParseResult::Continue
    } else {
        LineParseResult::NotHandled
    }
}

/// Try to parse @if or @if-not condition directives.
fn try_parse_condition_directive(trimmed: &str, state: &mut ParserState) -> LineParseResult {
    // Try @if directive
    let if_value = trimmed
        .strip_prefix("# @if ")
        .or_else(|| trimmed.strip_prefix("// @if "))
        .map(|s| s.trim());

    if let Some(value) = if_value {
        match parse_condition(value, false) {
            Some(condition) => state.pending_conditions.push(condition),
            None => eprintln!("Warning: Invalid @if directive format: '{}'", value),
        }
        return LineParseResult::Continue;
    }

    // Try @if-not directive
    let if_not_value = trimmed
        .strip_prefix("# @if-not ")
        .or_else(|| trimmed.strip_prefix("// @if-not "))
        .map(|s| s.trim());

    if let Some(value) = if_not_value {
        match parse_condition(value, true) {
            Some(condition) => state.pending_conditions.push(condition),
            None => eprintln!("Warning: Invalid @if-not directive format: '{}'", value),
        }
        return LineParseResult::Continue;
    }

    LineParseResult::NotHandled
}

/// Try to parse a variable definition line (@varname = value).
fn try_parse_variable_line(trimmed: &str, state: &mut ParserState) -> LineParseResult {
    if trimmed.starts_with('@') {
        if let Some(eq_pos) = trimmed.find('=') {
            let var_name = trimmed[1..eq_pos].trim();
            let var_value = trimmed[eq_pos + 1..].trim();
            state.set_variable(var_name, var_value);
        }
        LineParseResult::Continue
    } else {
        LineParseResult::NotHandled
    }
}

/// Strip quotes from a string value if present.
fn strip_quotes(value: &str) -> &str {
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

/// Try to parse an assertion line.
fn try_parse_assertion_line(trimmed: &str, state: &mut ParserState) -> LineParseResult {
    let assertion_line = trimmed.strip_prefix("> ").unwrap_or(trimmed);

    if let Some(stripped) = assertion_line.strip_prefix("EXPECTED_RESPONSE_STATUS ") {
        let status_str = stripped.trim();
        state.add_assertion(AssertionType::Status, status_str);
        return LineParseResult::Continue;
    }

    if let Some(stripped) = assertion_line.strip_prefix("EXPECTED_RESPONSE_BODY ") {
        let body_value = strip_quotes(stripped.trim());
        state.add_assertion(AssertionType::Body, body_value);
        return LineParseResult::Continue;
    }

    if let Some(stripped) = assertion_line.strip_prefix("EXPECTED_RESPONSE_HEADERS ") {
        let headers_value = strip_quotes(stripped.trim());
        state.add_assertion(AssertionType::Headers, headers_value);
        return LineParseResult::Continue;
    }

    LineParseResult::NotHandled
}

/// Parse a single line in the HTTP file.
fn parse_line(line: &str, state: &mut ParserState) {
    let trimmed = line.trim();

    // Handle IntelliJ script blocks
    if line.trim_start().starts_with("> {%") {
        state.in_intellij_script = true;
        return;
    }

    if state.in_intellij_script {
        if trimmed == "%}" || trimmed.ends_with("%}") {
            state.in_intellij_script = false;
        }
        return;
    }

    // Handle empty lines
    if trimmed.is_empty() {
        if state.in_body {
            state.body_content.push('\n');
        }
        return;
    }

    // Try parsing directives in order
    if matches!(try_parse_name_directive(trimmed, state), LineParseResult::Continue) {
        return;
    }

    if matches!(try_parse_timeout_directive(trimmed, state), LineParseResult::Continue) {
        return;
    }

    if matches!(
        try_parse_connection_timeout_directive(trimmed, state),
        LineParseResult::Continue
    ) {
        return;
    }

    if matches!(try_parse_depends_on_directive(trimmed, state), LineParseResult::Continue) {
        return;
    }

    if matches!(try_parse_condition_directive(trimmed, state), LineParseResult::Continue) {
        return;
    }

    // Skip comment lines (after directive processing)
    if trimmed.starts_with('#') || trimmed.starts_with("//") {
        return;
    }

    if matches!(try_parse_variable_line(trimmed, state), LineParseResult::Continue) {
        return;
    }

    if matches!(try_parse_assertion_line(trimmed, state), LineParseResult::Continue) {
        return;
    }

    // Parse HTTP request line
    if is_http_request_line(trimmed) {
        state.finalize_current_request();

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            let method = substitute_variables(parts[0], &state.variables);
            let url = substitute_variables(parts[1], &state.variables);
            state.start_new_request(method, url);
        }
        return;
    }

    // Parse header line
    if trimmed.contains(':') && !state.in_body {
        if state.current_request.is_some() {
            if let Some(colon_pos) = trimmed.find(':') {
                let name = trimmed[..colon_pos].trim();
                let value = trimmed[colon_pos + 1..].trim();
                state.add_header(name, value);
            }
        }
        return;
    }

    // Must be body content
    state.append_body_content(trimmed);
}

fn parse_http_content_with_vars(
    content: &str,
    env_variables: Vec<Variable>,
) -> Result<Vec<HttpRequest>> {
    let mut state = ParserState::new(env_variables);

    for line in content.lines() {
        parse_line(line, &mut state);
    }

    // Finalize any remaining request
    state.finalize_current_request();

    Ok(state.requests)
}
