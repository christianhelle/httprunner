use crate::types::{
    Header, HttpFileResults, HttpRequest, HttpResult, ProcessorResults, RequestContext,
};
use serde_json::Value;
use std::collections::HashMap;

const REDACTED_VALUE: &str = "***REDACTED***";

pub fn sanitize_processor_results(
    results: &ProcessorResults,
    include_secrets: bool,
) -> ProcessorResults {
    if include_secrets {
        return results.clone();
    }

    ProcessorResults {
        success: results.success,
        files: results
            .files
            .iter()
            .map(|file| sanitize_http_file_results(file, include_secrets))
            .collect(),
    }
}

pub fn sanitize_request_for_output(request: &HttpRequest, include_secrets: bool) -> HttpRequest {
    if include_secrets {
        return request.clone();
    }

    let mut sanitized = request.clone();
    sanitized.url = sanitize_url(&request.url, include_secrets);
    sanitized.headers = request
        .headers
        .iter()
        .map(|header| Header {
            name: header.name.clone(),
            value: redact_header_value(&header.name, &header.value, include_secrets),
        })
        .collect();
    sanitized.body = request
        .body
        .as_deref()
        .map(|body| sanitize_text(body, include_secrets));

    for assertion in &mut sanitized.assertions {
        assertion.expected_value = sanitize_text(&assertion.expected_value, include_secrets);
    }

    for condition in &mut sanitized.conditions {
        condition.expected_value = sanitize_text(&condition.expected_value, include_secrets);
    }

    sanitized
}

pub fn sanitize_result_for_output(result: &HttpResult, include_secrets: bool) -> HttpResult {
    if include_secrets {
        return result.clone();
    }

    let mut sanitized = result.clone();
    sanitized.response_headers =
        sanitize_response_headers(result.response_headers.as_ref(), include_secrets);
    sanitized.response_body = result
        .response_body
        .as_deref()
        .map(|body| sanitize_text(body, include_secrets));

    for assertion_result in &mut sanitized.assertion_results {
        assertion_result.assertion.expected_value =
            sanitize_text(&assertion_result.assertion.expected_value, include_secrets);
        assertion_result.actual_value = assertion_result
            .actual_value
            .as_deref()
            .map(|actual| sanitize_text(actual, include_secrets));
    }

    sanitized
}

pub fn redact_header_value(header_name: &str, value: &str, include_secrets: bool) -> String {
    if include_secrets || !is_sensitive_header(header_name) {
        value.to_string()
    } else {
        REDACTED_VALUE.to_string()
    }
}

pub fn sanitize_text(value: &str, include_secrets: bool) -> String {
    if include_secrets || value.is_empty() {
        return value.to_string();
    }

    if let Ok(mut json) = serde_json::from_str::<Value>(value) {
        return if redact_json_value(&mut json) {
            json.to_string()
        } else {
            value.to_string()
        };
    }

    let query_sanitized = sanitize_delimited_assignments(value, '&', '=');
    sanitize_line_assignments(&query_sanitized)
}

fn sanitize_http_file_results(
    file_results: &HttpFileResults,
    include_secrets: bool,
) -> HttpFileResults {
    HttpFileResults {
        filename: file_results.filename.clone(),
        success_count: file_results.success_count,
        failed_count: file_results.failed_count,
        skipped_count: file_results.skipped_count,
        result_contexts: file_results
            .result_contexts
            .iter()
            .map(|context| sanitize_request_context(context, include_secrets))
            .collect(),
    }
}

fn sanitize_request_context(context: &RequestContext, include_secrets: bool) -> RequestContext {
    RequestContext {
        name: context.name.clone(),
        request: sanitize_request_for_output(&context.request, include_secrets),
        result: context
            .result
            .as_ref()
            .map(|result| sanitize_result_for_output(result, include_secrets)),
    }
}

fn sanitize_response_headers(
    headers: Option<&HashMap<String, String>>,
    include_secrets: bool,
) -> Option<HashMap<String, String>> {
    headers.map(|headers| {
        headers
            .iter()
            .map(|(name, value)| {
                (
                    name.clone(),
                    redact_header_value(name, value, include_secrets),
                )
            })
            .collect()
    })
}

fn sanitize_url(url: &str, include_secrets: bool) -> String {
    if include_secrets {
        return url.to_string();
    }

    let (url_without_fragment, fragment) = match url.split_once('#') {
        Some((base, fragment)) => (base, Some(fragment)),
        None => (url, None),
    };

    let Some((base, query)) = url_without_fragment.split_once('?') else {
        return url.to_string();
    };

    let mut sanitized = format!("{base}?{}", sanitize_delimited_assignments(query, '&', '='));
    if let Some(fragment) = fragment {
        sanitized.push('#');
        sanitized.push_str(fragment);
    }
    sanitized
}

fn sanitize_delimited_assignments(
    input: &str,
    pair_delimiter: char,
    key_value_delimiter: char,
) -> String {
    if !input.contains(key_value_delimiter) {
        return input.to_string();
    }

    input
        .split(pair_delimiter)
        .map(|segment| {
            let Some((key, _value)) = segment.split_once(key_value_delimiter) else {
                return segment.to_string();
            };

            if is_sensitive_field_name(key) {
                format!("{key}{key_value_delimiter}{REDACTED_VALUE}")
            } else {
                segment.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(&pair_delimiter.to_string())
}

fn sanitize_line_assignments(input: &str) -> String {
    input
        .lines()
        .map(sanitize_line_assignment)
        .collect::<Vec<_>>()
        .join("\n")
}

fn sanitize_line_assignment(line: &str) -> String {
    if let Some((key, _value)) = line.split_once(':')
        && is_sensitive_field_name(key)
    {
        return format!("{}: {}", key.trim(), REDACTED_VALUE);
    }

    if let Some((key, _value)) = line.split_once('=')
        && is_sensitive_field_name(key)
    {
        return format!("{}={}", key.trim(), REDACTED_VALUE);
    }

    line.to_string()
}

fn redact_json_value(value: &mut Value) -> bool {
    match value {
        Value::Object(map) => {
            let mut redacted = false;
            for (key, child) in map.iter_mut() {
                if is_sensitive_field_name(key) {
                    *child = Value::String(REDACTED_VALUE.to_string());
                    redacted = true;
                } else {
                    redacted |= redact_json_value(child);
                }
            }
            redacted
        }
        Value::Array(items) => {
            let mut redacted = false;
            for item in items {
                redacted |= redact_json_value(item);
            }
            redacted
        }
        _ => false,
    }
}

fn is_sensitive_header(name: &str) -> bool {
    matches!(
        normalize_name(name).as_str(),
        "authorization"
            | "proxyauthorization"
            | "cookie"
            | "setcookie"
            | "apikey"
            | "xapikey"
            | "authtoken"
            | "xauthtoken"
            | "xaccesstoken"
            | "xrefreshtoken"
            | "xcsrftoken"
            | "xxsrftoken"
    )
}

fn is_sensitive_field_name(name: &str) -> bool {
    let normalized = normalize_name(name);
    normalized.contains("token")
        || normalized.contains("secret")
        || normalized.contains("password")
        || normalized.contains("passwd")
        || normalized.contains("apikey")
        || normalized.contains("authorization")
        || normalized.contains("cookie")
}

fn normalize_name(name: &str) -> String {
    name.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{HttpFileResults, ProcessorResults, RequestContext};

    fn sample_request() -> HttpRequest {
        HttpRequest {
            name: Some("login".to_string()),
            method: "POST".to_string(),
            url: "https://example.com?token=secret-token".to_string(),
            headers: vec![
                Header {
                    name: "Authorization".to_string(),
                    value: "Bearer secret-token".to_string(),
                },
                Header {
                    name: "Content-Type".to_string(),
                    value: "application/json".to_string(),
                },
            ],
            body: Some(r#"{"token":"secret-token","name":"john"}"#.to_string()),
            assertions: vec![],
            variables: vec![],
            timeout: None,
            connection_timeout: None,
            depends_on: None,
            conditions: vec![],
            pre_delay_ms: None,
            post_delay_ms: None,
        }
    }

    fn sample_result() -> HttpResult {
        let mut headers = HashMap::new();
        headers.insert("Set-Cookie".to_string(), "session=abc123".to_string());
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        HttpResult {
            request_name: Some("login".to_string()),
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 1,
            response_headers: Some(headers),
            response_body: Some(r#"{"password":"secret","status":"ok"}"#.to_string()),
            assertion_results: vec![],
        }
    }

    #[test]
    fn sanitize_request_for_output_redacts_sensitive_values() {
        let sanitized = sanitize_request_for_output(&sample_request(), false);

        assert!(sanitized.url.contains("token=***REDACTED***"));
        assert_eq!(sanitized.headers[0].value, "***REDACTED***");
        assert_eq!(sanitized.headers[1].value, "application/json");
        assert_eq!(
            sanitized.body.as_deref(),
            Some(r#"{"name":"john","token":"***REDACTED***"}"#)
        );
    }

    #[test]
    fn sanitize_result_for_output_redacts_sensitive_values() {
        let sanitized = sanitize_result_for_output(&sample_result(), false);
        let headers = sanitized.response_headers.unwrap();

        assert_eq!(
            headers.get("Set-Cookie").map(String::as_str),
            Some("***REDACTED***")
        );
        assert_eq!(
            sanitized.response_body.as_deref(),
            Some(r#"{"password":"***REDACTED***","status":"ok"}"#)
        );
    }

    #[test]
    fn sanitize_text_redacts_form_encoded_values() {
        assert_eq!(
            sanitize_text("username=test&password=secret", false),
            "username=test&password=***REDACTED***"
        );
    }

    #[test]
    fn sanitize_processor_results_preserves_values_when_opted_in() {
        let results = ProcessorResults {
            success: true,
            files: vec![HttpFileResults {
                filename: "test.http".to_string(),
                success_count: 1,
                failed_count: 0,
                skipped_count: 0,
                result_contexts: vec![RequestContext {
                    name: "login".to_string(),
                    request: sample_request(),
                    result: Some(sample_result()),
                }],
            }],
        };

        let sanitized = sanitize_processor_results(&results, true);
        assert_eq!(
            sanitized.files[0].result_contexts[0].request.headers[0].value,
            "Bearer secret-token"
        );
    }
}
