use crate::environment;
use crate::types::{
    Assertion, AssertionType, Condition, ConditionType, Header, HttpRequest, Variable,
};
use anyhow::{Context, Result};
use std::fs;

/// Parses an HTTP description file into a list of HttpRequest objects, applying environment variables and in-file directives.
///
/// The parser reads the file at `file_path`, loads environment variables (optionally selecting `environment_name`), expands `{{variable}}` templates, and recognizes in-file directives such as request naming (`# @name` / `// @name`), timeouts (`@timeout`, `@connection-timeout`), dependency (`@dependsOn`), conditional execution (`@if` and `@if-not`), variable definitions (`@NAME=VALUE`), assertions (`EXPECTED_RESPONSE_*`), headers, and request bodies. IntelliJ HTTP Client script blocks are ignored. Each detected HTTP request becomes an HttpRequest with populated fields including name, method, url, headers, body, assertions, variables, timeouts, depends_on, and conditions.
///
/// # Returns
///
/// A `Vec<HttpRequest>` containing one HttpRequest per request found in the file, with variables substituted and pending directives applied.
///
/// # Examples
///
/// ```
/// // Parse requests from "examples/requests.http" using the default environment
/// let requests = parse_http_file("examples/requests.http", None).unwrap();
/// assert!(!requests.is_empty());
/// ```
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
            match parse_condition(value, false) {
                Some(condition) => pending_conditions.push(condition),
                None => eprintln!("Warning: Invalid @if directive format: '{}'", value),
            }
            continue;
        }

        // Check for @if-not directive
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
            if let Some(ref mut req) = current_request
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

/// Substitutes `{{name}}` placeholders in a string with provided variable values.
///
/// Placeholders are delimited by double braces (`{{` and `}}`). If a placeholder's
/// name matches a `Variable.name` in `variables`, the placeholder is replaced with
/// that `Variable.value`. Unmatched or unterminated placeholders are left unchanged
/// (unterminated placeholders keep their leading `{{` and collected text).
///
/// # Examples
///
/// ```
/// use crate::Variable;
///
/// let vars = vec![Variable { name: "USER".into(), value: "alice".into() }];
/// let s = "Hello, {{USER}}! Unknown: {{MISSING}}".to_string();
/// let out = substitute_variables(&s, &vars);
/// assert_eq!(out, "Hello, alice! Unknown: {{MISSING}}");
/// ```
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

/// Parses an `@if` or `@if-not` directive string into a `Condition`.
///
/// The `value` should contain a reference and an expected value separated by whitespace.
/// Supported reference formats:
/// - `request-name.response.status` (expects a status check)
/// - `request-name.response.body.$.path.to.field` (expects a JSONPath-like body check)
///
/// `negate` marks the produced condition as negated (true for `@if-not`, false for `@if`).
///
/// # Returns
///
/// `Some(Condition)` if the directive is a valid status or body JSONPath condition; `None` if the format is invalid.
///
/// # Examples
///
/// ```
/// use crate::ConditionType;
///
/// // Status condition
/// let c = parse_condition("login.response.status 200", false).unwrap();
/// assert_eq!(c.request_name, "login");
/// assert!(matches!(c.condition_type, ConditionType::Status));
/// assert_eq!(c.expected_value, "200");
/// assert!(!c.negate);
///
/// // Body JSONPath condition (negated)
/// let c2 = parse_condition("fetch.response.body.$.user.id 42", true).unwrap();
/// assert_eq!(c2.request_name, "fetch");
/// assert!(matches!(c2.condition_type, ConditionType::BodyJsonPath(ref p) if p == " $.user.id".trim_start_matches('.').trim_start_matches('$').trim()));
/// assert_eq!(c2.expected_value, "42");
/// assert!(c2.negate);
/// ```
fn parse_condition(value: &str, negate: bool) -> Option<Condition> {
    // Parse @if or @if-not directive
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
            negate,
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
            negate,
        });
    }

    None
}

fn parse_timeout_value(value: &str) -> Option<u64> {
    let value = value.trim();

    // Check for time unit suffix
    // Returns timeout in milliseconds
    if let Some(stripped) = value.strip_suffix("ms") {
        // Milliseconds
        let num_str = stripped.trim();
        num_str.parse::<u64>().ok()
    } else if let Some(stripped) = value.strip_suffix('m') {
        // Minutes - convert to milliseconds
        let num_str = stripped.trim();
        num_str
            .parse::<u64>()
            .ok()
            .and_then(|m| m.checked_mul(60_000))
    } else if let Some(stripped) = value.strip_suffix('s') {
        // Seconds (explicit) - convert to milliseconds
        let num_str = stripped.trim();
        num_str
            .parse::<u64>()
            .ok()
            .and_then(|s| s.checked_mul(1_000))
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

    #[test]
    fn test_parse_http_file_with_depends_on() {
        let content = r#"
# @name request-one
GET https://example.com/api

###
# @name request-two
# @dependsOn request-one
POST https://example.com/api
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_depends_on.http");
        std::fs::write(&test_file, content).unwrap();

        let requests = parse_http_file(test_file.to_str().unwrap(), None).unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].depends_on, None);
        assert_eq!(requests[1].depends_on, Some("request-one".to_string()));

        std::fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_parse_http_file_with_if_status() {
        let content = r#"
# @name request-one
GET https://example.com/api

###
# @name request-two
# @if request-one.response.status 200
POST https://example.com/api
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_if_status.http");
        std::fs::write(&test_file, content).unwrap();

        let requests = parse_http_file(test_file.to_str().unwrap(), None).unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].conditions.len(), 0);
        assert_eq!(requests[1].conditions.len(), 1);
        assert_eq!(requests[1].conditions[0].request_name, "request-one");
        assert_eq!(requests[1].conditions[0].expected_value, "200");

        std::fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_parse_http_file_with_if_jsonpath() {
        let content = r#"
# @name request-one
GET https://example.com/api

###
# @name request-two
# @if request-one.response.body.$.username testuser
POST https://example.com/api
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_if_jsonpath.http");
        std::fs::write(&test_file, content).unwrap();

        let requests = parse_http_file(test_file.to_str().unwrap(), None).unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].conditions.len(), 0);
        assert_eq!(requests[1].conditions.len(), 1);
        assert_eq!(requests[1].conditions[0].request_name, "request-one");
        assert_eq!(requests[1].conditions[0].expected_value, "testuser");

        std::fs::remove_file(&test_file).ok();
    }

    /// Verifies that parsing an HTTP file attaches multiple `@if` conditions to the following request.
    ///
    /// # Examples
    ///
    /// ```
    /// // Creates a temporary HTTP file with two requests where the second request
    /// // has two `@if` condition directives. The parser should return two
    /// // requests and the second should have two conditions.
    /// let content = r#"
    /// # @name request-one
    /// GET https://example.com/api
    ///
    /// ###
    /// # @name request-two
    /// # @if request-one.response.status 200
    /// # @if request-one.response.body.$.username testuser
    /// POST https://example.com/api
    /// "#;
    /// let temp_dir = std::env::temp_dir();
    /// let test_file = temp_dir.join("test_multiple_conditions.http");
    /// std::fs::write(&test_file, content).unwrap();
    ///
    /// let requests = parse_http_file(test_file.to_str().unwrap(), None).unwrap();
    /// assert_eq!(requests.len(), 2);
    /// assert_eq!(requests[0].conditions.len(), 0);
    /// assert_eq!(requests[1].conditions.len(), 2);
    ///
    /// std::fs::remove_file(&test_file).ok();
    /// ```
    #[test]
    fn test_parse_http_file_with_multiple_conditions() {
        let content = r#"
# @name request-one
GET https://example.com/api

###
# @name request-two
# @if request-one.response.status 200
# @if request-one.response.body.$.username testuser
POST https://example.com/api
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_multiple_conditions.http");
        std::fs::write(&test_file, content).unwrap();

        let requests = parse_http_file(test_file.to_str().unwrap(), None).unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].conditions.len(), 0);
        assert_eq!(requests[1].conditions.len(), 2);

        std::fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_parse_http_file_with_if_not_status() {
        let content = r#"
# @name request-one
GET https://example.com/api

###
# @name request-two
# @if-not request-one.response.status 404
POST https://example.com/api
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_if_not_status.http");
        std::fs::write(&test_file, content).unwrap();

        let requests = parse_http_file(test_file.to_str().unwrap(), None).unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].conditions.len(), 0);
        assert_eq!(requests[1].conditions.len(), 1);
        assert_eq!(requests[1].conditions[0].request_name, "request-one");
        assert_eq!(requests[1].conditions[0].expected_value, "404");
        assert_eq!(requests[1].conditions[0].negate, true);

        std::fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_parse_http_file_with_if_not_jsonpath() {
        let content = r#"
# @name request-one
GET https://example.com/api

###
# @name request-two
# @if-not request-one.response.body.$.username testuser
POST https://example.com/api
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_if_not_jsonpath.http");
        std::fs::write(&test_file, content).unwrap();

        let requests = parse_http_file(test_file.to_str().unwrap(), None).unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].conditions.len(), 0);
        assert_eq!(requests[1].conditions.len(), 1);
        assert_eq!(requests[1].conditions[0].request_name, "request-one");
        assert_eq!(requests[1].conditions[0].expected_value, "testuser");
        assert_eq!(requests[1].conditions[0].negate, true);

        std::fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_parse_http_file_with_mixed_if_and_if_not() {
        let content = r#"
# @name request-one
GET https://example.com/api

###
# @name request-two
# @if request-one.response.status 200
# @if-not request-one.response.body.$.error true
POST https://example.com/api
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_mixed_conditions.http");
        std::fs::write(&test_file, content).unwrap();

        let requests = parse_http_file(test_file.to_str().unwrap(), None).unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].conditions.len(), 0);
        assert_eq!(requests[1].conditions.len(), 2);
        assert_eq!(requests[1].conditions[0].negate, false); // @if
        assert_eq!(requests[1].conditions[1].negate, true); // @if-not

        std::fs::remove_file(&test_file).ok();
    }
}