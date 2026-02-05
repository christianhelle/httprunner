use super::condition_parser::parse_condition;
use super::substitution::*;
use super::timeout_parser::parse_timeout_value;
use super::utils::is_http_request_line;
use crate::environment;
use crate::types::{Assertion, AssertionType, Condition, Header, HttpRequest, Variable};
use anyhow::{Context, Result};
use std::fs;

pub fn parse_http_file(
    file_path: &str,
    environment_name: Option<&str>,
) -> Result<Vec<HttpRequest>> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path))?;

    let env_variables = environment::load_environment_file(file_path, environment_name)?;
    parse_http_content_with_vars(&content, env_variables)
}

pub fn parse_http_content(
    content: &str,
    _environment_name: Option<&str>,
) -> Result<Vec<HttpRequest>> {
    let env_variables = Vec::new();
    parse_http_content_with_vars(content, env_variables)
}

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
    pending_pre_delay: Option<u64>,
    pending_post_delay: Option<u64>,
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
            pending_pre_delay: None,
            pending_post_delay: None,
            in_intellij_script: false,
        }
    }

    fn finalize_current_request(&mut self) {
        if let Some(mut req) = self.current_request.take() {
            if !self.body_content.is_empty() {
                req.body = Some(substitute_variables(&self.body_content, &self.variables));
            }
            self.requests.push(req);
            self.body_content.clear();
        }
    }

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
            pre_delay_ms: self.pending_pre_delay.take(),
            post_delay_ms: self.pending_post_delay.take(),
        });
        self.in_body = false;
    }

    fn add_header(&mut self, name: &str, value: &str) {
        if let Some(ref mut req) = self.current_request {
            req.headers.push(Header {
                name: substitute_variables(name, &self.variables),
                value: substitute_variables(value, &self.variables),
            });
        }
    }

    fn add_assertion(&mut self, assertion_type: AssertionType, expected_value: &str) {
        if let Some(ref mut req) = self.current_request {
            req.assertions.push(Assertion {
                assertion_type,
                expected_value: substitute_variables(expected_value, &self.variables),
            });
        }
    }

    fn append_body_content(&mut self, content: &str) {
        self.in_body = true;
        if !self.body_content.is_empty() {
            self.body_content.push('\n');
        }
        self.body_content.push_str(content);
    }

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

enum LineParseResult {
    Continue,
    NotHandled,
    Error(String),
}

fn try_parse_name_directive(trimmed: &str, state: &mut ParserState) -> LineParseResult {
    if let Some(name) = trimmed.strip_prefix("# @name ") {
        state.pending_request_name = Some(name.trim().to_string());
        LineParseResult::Continue
    } else if let Some(name) = trimmed.strip_prefix("// @name ") {
        state.pending_request_name = Some(name.trim().to_string());
        LineParseResult::Continue
    } else {
        LineParseResult::NotHandled
    }
}

fn try_parse_timeout_directive(trimmed: &str, state: &mut ParserState) -> LineParseResult {
    let timeout_value = trimmed
        .strip_prefix("# @timeout ")
        .or_else(|| trimmed.strip_prefix("// @timeout "))
        .map(|s| s.trim());

    if let Some(value) = timeout_value {
        match parse_timeout_value(value) {
            Some(timeout) => {
                state.pending_timeout = Some(timeout);
                LineParseResult::Continue
            }
            None => LineParseResult::Error(format!("Invalid timeout value: '{}'", value)),
        }
    } else {
        LineParseResult::NotHandled
    }
}

fn try_parse_connection_timeout_directive(
    trimmed: &str,
    state: &mut ParserState,
) -> LineParseResult {
    let timeout_value = trimmed
        .strip_prefix("# @connection-timeout ")
        .or_else(|| trimmed.strip_prefix("// @connection-timeout "))
        .map(|s| s.trim());

    if let Some(value) = timeout_value {
        match parse_timeout_value(value) {
            Some(timeout) => {
                state.pending_connection_timeout = Some(timeout);
                LineParseResult::Continue
            }
            None => {
                LineParseResult::Error(format!("Invalid connection-timeout value: '{}'", value))
            }
        }
    } else {
        LineParseResult::NotHandled
    }
}

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

fn try_parse_pre_delay_directive(trimmed: &str, state: &mut ParserState) -> LineParseResult {
    let delay_value = trimmed
        .strip_prefix("# @pre-delay ")
        .or_else(|| trimmed.strip_prefix("// @pre-delay "))
        .map(|s| s.trim());

    if let Some(value) = delay_value {
        match value.parse::<u64>() {
            Ok(delay_ms) => {
                state.pending_pre_delay = Some(delay_ms);
                LineParseResult::Continue
            }
            Err(_) => LineParseResult::Error(format!(
                "Invalid @pre-delay value '{}', expected number in milliseconds",
                value
            )),
        }
    } else {
        LineParseResult::NotHandled
    }
}

fn try_parse_post_delay_directive(trimmed: &str, state: &mut ParserState) -> LineParseResult {
    let delay_value = trimmed
        .strip_prefix("# @post-delay ")
        .or_else(|| trimmed.strip_prefix("// @post-delay "))
        .map(|s| s.trim());

    if let Some(value) = delay_value {
        match value.parse::<u64>() {
            Ok(delay_ms) => {
                state.pending_post_delay = Some(delay_ms);
                LineParseResult::Continue
            }
            Err(_) => LineParseResult::Error(format!(
                "Invalid @post-delay value '{}', expected number in milliseconds",
                value
            )),
        }
    } else {
        LineParseResult::NotHandled
    }
}

fn try_parse_variable_line(trimmed: &str, state: &mut ParserState) -> LineParseResult {
    if trimmed.starts_with('@') {
        // If we're in the body, don't treat '@' as a variable declaration
        if state.in_body {
            return LineParseResult::NotHandled;
        }

        // Outside the body, '@' lines should be variable declarations with '='
        if let Some(eq_pos) = trimmed.find('=') {
            let var_name = trimmed[1..eq_pos].trim();
            let var_value = trimmed[eq_pos + 1..].trim();
            state.set_variable(var_name, var_value);
            LineParseResult::Continue
        } else {
            // '@' line without '=' is likely a typo
            LineParseResult::Error(format!(
                "Invalid variable declaration: '{}' (missing '=')",
                trimmed
            ))
        }
    } else {
        LineParseResult::NotHandled
    }
}

fn strip_quotes(value: &str) -> &str {
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

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
        } else if state.current_request.is_some() {
            state.in_body = true;
        }
        return;
    }

    // Try parsing directives in order
    match try_parse_name_directive(trimmed, state) {
        LineParseResult::Continue => return,
        LineParseResult::Error(msg) => {
            eprintln!("Warning: {}", msg);
            return;
        }
        LineParseResult::NotHandled => {}
    }

    match try_parse_timeout_directive(trimmed, state) {
        LineParseResult::Continue => return,
        LineParseResult::Error(msg) => {
            eprintln!("Warning: {}", msg);
            return;
        }
        LineParseResult::NotHandled => {}
    }

    match try_parse_connection_timeout_directive(trimmed, state) {
        LineParseResult::Continue => return,
        LineParseResult::Error(msg) => {
            eprintln!("Warning: {}", msg);
            return;
        }
        LineParseResult::NotHandled => {}
    }

    match try_parse_depends_on_directive(trimmed, state) {
        LineParseResult::Continue => return,
        LineParseResult::Error(msg) => {
            eprintln!("Warning: {}", msg);
            return;
        }
        LineParseResult::NotHandled => {}
    }

    match try_parse_condition_directive(trimmed, state) {
        LineParseResult::Continue => return,
        LineParseResult::Error(msg) => {
            eprintln!("Warning: {}", msg);
            return;
        }
        LineParseResult::NotHandled => {}
    }

    match try_parse_pre_delay_directive(trimmed, state) {
        LineParseResult::Continue => return,
        LineParseResult::Error(msg) => {
            eprintln!("Warning: {}", msg);
            return;
        }
        LineParseResult::NotHandled => {}
    }

    match try_parse_post_delay_directive(trimmed, state) {
        LineParseResult::Continue => return,
        LineParseResult::Error(msg) => {
            eprintln!("Warning: {}", msg);
            return;
        }
        LineParseResult::NotHandled => {}
    }

    // Skip comment lines (after directive processing)
    if trimmed.starts_with('#') || trimmed.starts_with("//") {
        return;
    }

    match try_parse_variable_line(trimmed, state) {
        LineParseResult::Continue => return,
        LineParseResult::Error(msg) => {
            eprintln!("Warning: {}", msg);
            return;
        }
        LineParseResult::NotHandled => {}
    }

    match try_parse_assertion_line(trimmed, state) {
        LineParseResult::Continue => return,
        LineParseResult::Error(msg) => {
            eprintln!("Warning: {}", msg);
            return;
        }
        LineParseResult::NotHandled => {}
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
        if state.current_request.is_some()
            && let Some(colon_pos) = trimmed.find(':')
        {
            let name = trimmed[..colon_pos].trim();
            let value = trimmed[colon_pos + 1..].trim();
            state.add_header(name, value);
        }
        return;
    }

    // Must be body content - preserve original line with whitespace
    state.append_body_content(line);
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
