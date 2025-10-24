use crate::types::{RequestContext, RequestVariable, RequestVariableSource, RequestVariableTarget};
use anyhow::{anyhow, Result};

pub fn parse_request_variable(reference: &str) -> Result<RequestVariable> {
    // Parse syntax: {{<request_name>.(request|response).(body|headers).(*|JSONPath|XPath|<header_name>)}}

    let mut cleaned = reference;
    if reference.starts_with("{{") && reference.ends_with("}}") {
        cleaned = &reference[2..reference.len() - 2];
    }

    let parts: Vec<&str> = cleaned.split('.').collect();
    if parts.len() < 4 {
        return Err(anyhow!("Invalid request variable format"));
    }

    let request_name = parts[0];
    let source_str = parts[1];
    let target_str = parts[2];
    let path = parts[3..].join(".");

    let source = match source_str {
        "request" => RequestVariableSource::Request,
        "response" => RequestVariableSource::Response,
        _ => return Err(anyhow!("Invalid source: {}", source_str)),
    };

    let target = match target_str {
        "body" => RequestVariableTarget::Body,
        "headers" => RequestVariableTarget::Headers,
        _ => return Err(anyhow!("Invalid target: {}", target_str)),
    };

    Ok(RequestVariable {
        reference: reference.to_string(),
        request_name: request_name.to_string(),
        source,
        target,
        path,
    })
}

pub fn extract_request_variable_value(
    request_var: &RequestVariable,
    context: &[RequestContext],
) -> Result<Option<String>> {
    // Find the request context by name
    let target_context = context
        .iter()
        .find(|ctx| ctx.name == request_var.request_name);

    if target_context.is_none() {
        return Ok(None);
    }

    let ctx = target_context.unwrap();

    match request_var.source {
        RequestVariableSource::Request => extract_from_request(request_var, &ctx.request),
        RequestVariableSource::Response => {
            if let Some(ref result) = ctx.result {
                extract_from_response(request_var, result)
            } else {
                Ok(None)
            }
        }
    }
}

fn extract_from_request(
    request_var: &RequestVariable,
    request: &crate::types::HttpRequest,
) -> Result<Option<String>> {
    match request_var.target {
        RequestVariableTarget::Body => {
            if let Some(ref body) = request.body {
                Ok(Some(body.clone()))
            } else {
                Ok(None)
            }
        }
        RequestVariableTarget::Headers => {
            for header in &request.headers {
                if header.name.eq_ignore_ascii_case(&request_var.path) {
                    return Ok(Some(header.value.clone()));
                }
            }
            Ok(None)
        }
    }
}

fn extract_from_response(
    request_var: &RequestVariable,
    result: &crate::types::HttpResult,
) -> Result<Option<String>> {
    match request_var.target {
        RequestVariableTarget::Body => {
            if let Some(ref body) = result.response_body {
                if request_var.path == "*" {
                    return Ok(Some(body.clone()));
                }

                // Basic JSONPath support for $.property
                if request_var.path.starts_with("$.") {
                    let property = &request_var.path[2..];
                    return extract_json_property(body, property);
                }

                Ok(Some(body.clone()))
            } else {
                Ok(None)
            }
        }
        RequestVariableTarget::Headers => {
            if let Some(ref headers) = result.response_headers {
                for (name, value) in headers {
                    if name.eq_ignore_ascii_case(&request_var.path) {
                        return Ok(Some(value.clone()));
                    }
                }
            }
            Ok(None)
        }
    }
}

fn extract_json_property(json_body: &str, property: &str) -> Result<Option<String>> {
    // Handle nested properties like "json.token"
    let parts: Vec<&str> = property.split('.').collect();
    let mut current_json = json_body.to_string();

    for part in parts {
        match extract_simple_json_property(&current_json, part) {
            Ok(Some(value)) => current_json = value,
            Ok(None) => return Ok(None),
            Err(e) => return Err(e),
        }
    }

    Ok(Some(current_json))
}

fn extract_simple_json_property(json_body: &str, property: &str) -> Result<Option<String>> {
    let search_pattern = format!("\"{}\":", property);

    if let Some(start_pos) = json_body.find(&search_pattern) {
        let mut pos = start_pos + search_pattern.len();
        let chars: Vec<char> = json_body.chars().collect();

        // Skip whitespace
        while pos < chars.len() && matches!(chars[pos], ' ' | '\t' | '\n' | '\r') {
            pos += 1;
        }

        if pos >= chars.len() {
            return Ok(None);
        }

        // Check if value is a string (starts with ")
        if chars[pos] == '"' {
            pos += 1;
            let value_start = pos;

            // Find closing quote
            while pos < chars.len() && chars[pos] != '"' {
                pos += 1;
            }

            if pos < chars.len() {
                let value: String = chars[value_start..pos].iter().collect();
                return Ok(Some(value));
            }
        } else if chars[pos] == '{' {
            // Handle object value
            let mut brace_count = 1;
            pos += 1;
            let value_start = pos - 1;

            while pos < chars.len() && brace_count > 0 {
                if chars[pos] == '{' {
                    brace_count += 1;
                } else if chars[pos] == '}' {
                    brace_count -= 1;
                }
                pos += 1;
            }

            if brace_count == 0 {
                let value: String = chars[value_start..pos].iter().collect();
                return Ok(Some(value));
            }
        } else {
            // Non-string value (number, boolean, null)
            let value_start = pos;

            while pos < chars.len()
                && !matches!(chars[pos], ',' | '}' | ']' | ' ' | '\t' | '\n' | '\r')
            {
                pos += 1;
            }

            let value: String = chars[value_start..pos].iter().collect();
            return Ok(Some(value));
        }
    }

    Ok(None)
}

pub fn substitute_request_variables(input: &str, context: &[RequestContext]) -> Result<String> {
    let mut result = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '{' {
            let mut j = i + 2;
            while j + 1 < chars.len() && !(chars[j] == '}' && chars[j + 1] == '}') {
                j += 1;
            }

            if j + 1 < chars.len() {
                let var_ref: String = chars[i..=j + 1].iter().collect();

                // Check if this looks like a request variable (contains dots)
                if var_ref.matches('.').count() >= 3 {
                    match parse_request_variable(&var_ref) {
                        Ok(request_var) => {
                            match extract_request_variable_value(&request_var, context) {
                                Ok(Some(value)) => result.push_str(&value),
                                _ => result.push_str(&var_ref),
                            }
                        }
                        Err(_) => result.push_str(&var_ref),
                    }
                } else {
                    result.push_str(&var_ref);
                }

                i = j + 2;
            } else {
                result.push(chars[i]);
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    Ok(result)
}
