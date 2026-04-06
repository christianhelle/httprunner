use super::condition_parser::parse_condition;
use super::pest_parse_tree::{PestHttpFile, PestLine, PestLineKind, PestScriptBlock};
use super::pest_parser::parse_http_content_to_pest_tree;
use super::substitution::substitute_variables;
use super::timeout_parser::parse_timeout_value;
use super::utils::is_http_request_line;
use crate::environment;
use crate::types::{Assertion, AssertionType, Condition, Header, HttpRequest, Variable};
use anyhow::{Context, Result, anyhow};
use std::fs;

pub fn parse_http_file(
    file_path: &str,
    environment_name: Option<&str>,
) -> Result<Vec<HttpRequest>> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path))?;

    let env_variables = environment::load_environment_file(file_path, environment_name)?;
    parse_http_content_with_pest_semantics(&content, env_variables)
}

pub fn parse_http_content(
    content: &str,
    _environment_name: Option<&str>,
) -> Result<Vec<HttpRequest>> {
    parse_http_content_with_pest_semantics(content, Vec::new())
}

pub(crate) fn parse_http_content_with_pest_semantics(
    content: &str,
    env_variables: Vec<Variable>,
) -> Result<Vec<HttpRequest>> {
    let tree = parse_http_content_to_pest_tree(content)?;
    assemble_http_requests_from_pest_tree(&tree, env_variables)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn assemble_http_requests_from_pest_tree(
    tree: &PestHttpFile,
    env_variables: Vec<Variable>,
) -> Result<Vec<HttpRequest>> {
    let mut state = SemanticAssemblerState::new(env_variables);

    for line in &tree.lines {
        assemble_line(line, &mut state).with_context(|| {
            format!(
                "Failed to parse line {}: {}",
                line.line_number,
                line.raw.trim()
            )
        })?;
    }

    state.finalize_current_request();
    Ok(state.requests)
}

struct SemanticAssemblerState {
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

impl SemanticAssemblerState {
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

fn try_parse_name_directive(trimmed: &str, state: &mut SemanticAssemblerState) -> LineParseResult {
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

fn try_parse_timeout_directive(
    trimmed: &str,
    state: &mut SemanticAssemblerState,
) -> LineParseResult {
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
    state: &mut SemanticAssemblerState,
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

fn try_parse_depends_on_directive(
    trimmed: &str,
    state: &mut SemanticAssemblerState,
) -> LineParseResult {
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

fn try_parse_condition_directive(
    trimmed: &str,
    state: &mut SemanticAssemblerState,
) -> LineParseResult {
    let if_value = trimmed
        .strip_prefix("# @if ")
        .or_else(|| trimmed.strip_prefix("// @if "))
        .map(|s| s.trim());

    if let Some(value) = if_value {
        match parse_condition(value, false) {
            Some(condition) => state.pending_conditions.push(condition),
            None => {
                return LineParseResult::Error(format!("Invalid @if directive format: '{value}'"));
            }
        }
        return LineParseResult::Continue;
    }

    let if_not_value = trimmed
        .strip_prefix("# @if-not ")
        .or_else(|| trimmed.strip_prefix("// @if-not "))
        .map(|s| s.trim());

    if let Some(value) = if_not_value {
        match parse_condition(value, true) {
            Some(condition) => state.pending_conditions.push(condition),
            None => {
                return LineParseResult::Error(format!(
                    "Invalid @if-not directive format: '{value}'"
                ));
            }
        }
        return LineParseResult::Continue;
    }

    LineParseResult::NotHandled
}

fn try_parse_pre_delay_directive(
    trimmed: &str,
    state: &mut SemanticAssemblerState,
) -> LineParseResult {
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

fn try_parse_post_delay_directive(
    trimmed: &str,
    state: &mut SemanticAssemblerState,
) -> LineParseResult {
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

fn try_parse_variable_line(
    trimmed: &str,
    raw: &str,
    state: &mut SemanticAssemblerState,
) -> LineParseResult {
    if trimmed.starts_with('@') {
        if state.in_body {
            state.append_body_content(raw);
            return LineParseResult::Continue;
        }

        if let Some(eq_pos) = trimmed.find('=') {
            let var_name = trimmed[1..eq_pos].trim();
            let var_value = trimmed[eq_pos + 1..].trim();
            state.set_variable(var_name, var_value);
            LineParseResult::Continue
        } else {
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

fn try_parse_assertion_line(trimmed: &str, state: &mut SemanticAssemblerState) -> LineParseResult {
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

fn handle_line_parse_result(result: LineParseResult) -> Result<bool> {
    match result {
        LineParseResult::Continue => Ok(true),
        LineParseResult::NotHandled => Ok(false),
        LineParseResult::Error(message) => Err(anyhow!(message)),
    }
}

fn handle_grouped_script_block(
    block: &PestScriptBlock,
    state: &mut SemanticAssemblerState,
) -> Result<()> {
    if block.start.trim_start().starts_with("> {%") {
        state.in_intellij_script = true;
    }

    if let Some(end) = &block.end {
        let trimmed = end.trim();
        if trimmed == "%}" || trimmed.ends_with("%}") {
            state.in_intellij_script = false;
        }
    }

    Ok(())
}

fn assemble_line(line: &PestLine, state: &mut SemanticAssemblerState) -> Result<()> {
    if let PestLineKind::IgnoredScriptBlock(block) = &line.kind {
        return handle_grouped_script_block(block, state);
    }

    let raw = line.raw.as_str();
    let trimmed = raw.trim();

    if raw.trim_start().starts_with("> {%") {
        state.in_intellij_script = true;
        return Ok(());
    }

    if state.in_intellij_script {
        if trimmed == "%}" || trimmed.ends_with("%}") {
            state.in_intellij_script = false;
        }
        return Ok(());
    }

    if trimmed.is_empty() {
        if state.in_body {
            state.body_content.push('\n');
        } else if state.current_request.is_some() {
            state.in_body = true;
        }
        return Ok(());
    }

    if handle_line_parse_result(try_parse_name_directive(trimmed, state))? {
        return Ok(());
    }

    if handle_line_parse_result(try_parse_timeout_directive(trimmed, state))? {
        return Ok(());
    }

    if handle_line_parse_result(try_parse_connection_timeout_directive(trimmed, state))? {
        return Ok(());
    }

    if handle_line_parse_result(try_parse_depends_on_directive(trimmed, state))? {
        return Ok(());
    }

    if handle_line_parse_result(try_parse_condition_directive(trimmed, state))? {
        return Ok(());
    }

    if handle_line_parse_result(try_parse_pre_delay_directive(trimmed, state))? {
        return Ok(());
    }

    if handle_line_parse_result(try_parse_post_delay_directive(trimmed, state))? {
        return Ok(());
    }

    if trimmed.starts_with('#') || trimmed.starts_with("//") {
        return Ok(());
    }

    if handle_line_parse_result(try_parse_variable_line(trimmed, raw, state))? {
        return Ok(());
    }

    if handle_line_parse_result(try_parse_assertion_line(trimmed, state))? {
        return Ok(());
    }

    if is_http_request_line(trimmed) {
        state.finalize_current_request();

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            let method = substitute_variables(parts[0], &state.variables);
            let url = substitute_variables(parts[1], &state.variables);
            state.start_new_request(method, url);
        }
        return Ok(());
    }

    if trimmed.contains(':') && !state.in_body {
        if state.current_request.is_some()
            && let Some(colon_pos) = trimmed.find(':')
        {
            let name = trimmed[..colon_pos].trim();
            let value = trimmed[colon_pos + 1..].trim();
            state.add_header(name, value);
        }
        return Ok(());
    }

    state.append_body_content(raw);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{
        parse_http_content, parse_http_content_with_legacy_backend, parse_http_file,
        parse_http_file_with_legacy_backend,
    };
    use crate::types::ConditionType;
    use std::fs;
    use tempfile::TempDir;

    fn assert_requests_match(actual: &[HttpRequest], expected: &[HttpRequest]) {
        assert_eq!(actual.len(), expected.len());

        for (actual, expected) in actual.iter().zip(expected.iter()) {
            assert_eq!(actual.name, expected.name);
            assert_eq!(actual.method, expected.method);
            assert_eq!(actual.url, expected.url);
            assert_eq!(actual.body, expected.body);
            assert_eq!(actual.timeout, expected.timeout);
            assert_eq!(actual.connection_timeout, expected.connection_timeout);
            assert_eq!(actual.depends_on, expected.depends_on);
            assert_eq!(actual.pre_delay_ms, expected.pre_delay_ms);
            assert_eq!(actual.post_delay_ms, expected.post_delay_ms);
            assert_eq!(actual.headers.len(), expected.headers.len());
            assert_eq!(actual.assertions.len(), expected.assertions.len());
            assert_eq!(actual.conditions.len(), expected.conditions.len());

            for (actual, expected) in actual.headers.iter().zip(expected.headers.iter()) {
                assert_eq!(actual.name, expected.name);
                assert_eq!(actual.value, expected.value);
            }

            for (actual, expected) in actual.assertions.iter().zip(expected.assertions.iter()) {
                assert_eq!(actual.assertion_type, expected.assertion_type);
                assert_eq!(actual.expected_value, expected.expected_value);
            }

            for (actual, expected) in actual.conditions.iter().zip(expected.conditions.iter()) {
                assert_eq!(actual.request_name, expected.request_name);
                assert_eq!(actual.condition_type, expected.condition_type);
                assert_eq!(actual.expected_value, expected.expected_value);
                assert_eq!(actual.negate, expected.negate);
            }
        }
    }

    fn assert_public_parser_matches_legacy_backend(content: &str) {
        let expected = parse_http_content_with_legacy_backend(content, None)
            .expect("legacy parser should succeed");
        let actual = parse_http_content(content, None).expect("public parser should succeed");
        assert_requests_match(&actual, &expected);
    }

    fn assert_public_file_parser_matches_legacy_backend(content: &str) {
        let temp_dir = TempDir::new().expect("temp dir should create");
        let file_path = temp_dir.path().join("parser-backend-swap.http");
        fs::write(&file_path, content).expect("test file should write");

        let file_path = file_path.to_str().expect("temp file path should be utf-8");
        let expected = parse_http_file_with_legacy_backend(file_path, None)
            .expect("legacy file parser should succeed");
        let actual = parse_http_file(file_path, None).expect("public file parser should succeed");

        assert_requests_match(&actual, &expected);
    }

    #[test]
    fn pest_semantic_assembler_matches_body_mode_and_directive_buffering() {
        let content = r#"# @name first
POST https://api.example.com/first
Content-Type: text/plain

first-body
# @name second
# @dependsOn first
# @timeout 5s
GET https://api.example.com/second"#;

        assert_public_parser_matches_legacy_backend(content);
    }

    #[test]
    fn pest_semantic_assembler_matches_script_blocks_assertions_and_request_boundaries() {
        let content = r#"POST https://api.example.com/first

body line
# ignored comment
> {%
client.test("ignored", function() {});
%}
> EXPECTED_RESPONSE_STATUS 204
GET https://api.example.com/second"#;

        assert_public_parser_matches_legacy_backend(content);
    }

    #[test]
    fn pest_semantic_assembler_matches_variable_substitution_timing() {
        let content = r#"@host = first.example.com
POST https://{{host}}/users
@host = second.example.com
X-Host: {{host}}

{{host}}
GET https://api.example.com/final"#;

        let actual = parse_http_content(content, None).expect("public parser should succeed");

        assert_eq!(actual.len(), 2);
        assert_eq!(actual[0].url, "https://first.example.com/users");
        assert_eq!(actual[0].headers[0].value, "second.example.com");
        assert_eq!(actual[0].body.as_deref(), Some("second.example.com"));

        let expected = parse_http_content_with_legacy_backend(content, None)
            .expect("legacy parser should succeed");
        assert_requests_match(&actual, &expected);
    }

    #[test]
    fn pest_semantic_assembler_matches_indented_lines_and_body_whitespace() {
        let content = "   # @name spaced\n   POST https://api.example.com/users\n   Content-Type: application/json\n   \n     {\n       \"name\": \"Jane\"\n     }\n";

        assert_public_parser_matches_legacy_backend(content);
    }

    #[test]
    fn pest_semantic_assembler_matches_reference_directive_example() {
        let content = r#"# @name login
# @timeout 30
POST https://api.example.com/auth/login
Content-Type: application/json

{"username": "user", "password": "pass"}

###

# @pre-delay 2000
// @connection-timeout 5
GET https://api.example.com/status

###

# @name getUser
# @dependsOn login
# @if login.response.status 200
GET https://api.example.com/user/profile
Authorization: Bearer {{login.response.body.$.token}}

###

# @if-not getUser.response.status 200
POST https://api.example.com/user/create
Content-Type: application/json

{"username": "newuser"}"#;

        let actual = parse_http_content(content, None).expect("public parser should succeed");

        assert_eq!(actual.len(), 4);
        assert_eq!(actual[0].timeout, Some(30_000));
        assert_eq!(actual[1].pre_delay_ms, Some(2000));
        assert_eq!(actual[1].connection_timeout, Some(5000));
        assert_eq!(actual[2].depends_on.as_deref(), Some("login"));
        assert!(matches!(
            actual[2].conditions[0].condition_type,
            ConditionType::Status
        ));
        assert!(actual[3].conditions[0].negate);

        let expected = parse_http_content_with_legacy_backend(content, None)
            .expect("legacy parser should succeed");
        assert_requests_match(&actual, &expected);
    }

    #[test]
    fn pest_semantic_assembler_preserves_invalid_directive_errors_with_line_context() {
        let content = "GET https://api.example.com/users\n  # @timeout nope";

        let error = parse_http_content_with_pest_semantics(content, Vec::new()).unwrap_err();
        let message = format!("{error:#}");

        assert!(message.contains("Failed to parse line 2: # @timeout nope"));
        assert!(message.contains("Invalid timeout value: 'nope'"));
    }

    #[test]
    fn public_parse_http_file_matches_legacy_backend() {
        let content = r#"# @name login
POST https://api.example.com/login
Content-Type: application/json

{"username":"demo","password":"secret"}

###

# @dependsOn login
# @if login.response.status == 200
GET https://api.example.com/profile
Authorization: Bearer {{login.response.body.$.token}}"#;

        assert_public_file_parser_matches_legacy_backend(content);
    }
}
