use crate::types::{HttpRequest, HttpResult};
use std::collections::HashMap;

pub fn build_error_result(
    request: &HttpRequest,
    error_message: &str,
    duration_ms: u64,
) -> HttpResult {
    HttpResult {
        request_name: request.name.clone(),
        status_code: 0,
        success: false,
        error_message: Some(error_message.to_string()),
        duration_ms,
        response_headers: None,
        response_body: None,
        assertion_results: Vec::new(),
    }
}

pub fn extract_headers(headers: &reqwest::header::HeaderMap) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for (name, value) in headers {
        if let Ok(value_str) = value.to_str() {
            result.insert(name.to_string(), value_str.to_string());
        }
    }
    result
}

pub fn should_capture_response(request: &HttpRequest, verbose: bool) -> bool {
    verbose || !request.assertions.is_empty() || request.name.is_some()
}

pub fn build_success_result(
    request: &HttpRequest,
    status_code: u16,
    is_success: bool,
    duration_ms: u64,
    response_headers: Option<HashMap<String, String>>,
    response_body: Option<String>,
    assertion_results: Vec<crate::types::AssertionResult>,
) -> HttpResult {
    HttpResult {
        request_name: request.name.clone(),
        status_code,
        success: is_success,
        error_message: None,
        duration_ms,
        response_headers,
        response_body,
        assertion_results,
    }
}

pub fn build_temp_result_for_assertions(
    request: &HttpRequest,
    status_code: u16,
    success: bool,
    duration_ms: u64,
    response_headers: Option<HashMap<String, String>>,
    response_body: Option<String>,
) -> HttpResult {
    HttpResult {
        request_name: request.name.clone(),
        status_code,
        success,
        error_message: None,
        duration_ms,
        response_headers,
        response_body,
        assertion_results: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Assertion, AssertionType};

    fn create_test_request() -> HttpRequest {
        HttpRequest {
            name: Some("test_request".to_string()),
            method: "GET".to_string(),
            url: "https://example.com".to_string(),
            headers: Vec::new(),
            body: None,
            assertions: Vec::new(),
            variables: Vec::new(),
            timeout: None,
            connection_timeout: None,
            depends_on: None,
            conditions: Vec::new(),
        }
    }

    #[test]
    fn test_build_error_result() {
        let request = create_test_request();
        let result = build_error_result(&request, "Connection failed", 100);

        assert_eq!(result.request_name, Some("test_request".to_string()));
        assert_eq!(result.status_code, 0);
        assert!(!result.success);
        assert_eq!(result.error_message, Some("Connection failed".to_string()));
        assert_eq!(result.duration_ms, 100);
        assert!(result.response_headers.is_none());
        assert!(result.response_body.is_none());
        assert!(result.assertion_results.is_empty());
    }

    #[test]
    fn test_build_error_result_without_name() {
        let mut request = create_test_request();
        request.name = None;
        let result = build_error_result(&request, "Timeout", 500);

        assert!(result.request_name.is_none());
        assert_eq!(result.error_message, Some("Timeout".to_string()));
    }

    #[test]
    fn test_should_capture_response_verbose() {
        let request = create_test_request();
        assert!(should_capture_response(&request, true));
    }

    #[test]
    fn test_should_capture_response_with_assertions() {
        let mut request = create_test_request();
        request.name = None;
        request.assertions.push(Assertion {
            assertion_type: AssertionType::Status,
            expected_value: "200".to_string(),
        });
        assert!(should_capture_response(&request, false));
    }

    #[test]
    fn test_should_capture_response_with_name() {
        let request = create_test_request();
        assert!(should_capture_response(&request, false));
    }

    #[test]
    fn test_should_not_capture_response() {
        let mut request = create_test_request();
        request.name = None;
        request.assertions.clear();
        assert!(!should_capture_response(&request, false));
    }

    #[test]
    fn test_build_success_result() {
        let request = create_test_request();
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let result = build_success_result(
            &request,
            200,
            true,
            150,
            Some(headers.clone()),
            Some("{\"ok\": true}".to_string()),
            Vec::new(),
        );

        assert_eq!(result.request_name, Some("test_request".to_string()));
        assert_eq!(result.status_code, 200);
        assert!(result.success);
        assert!(result.error_message.is_none());
        assert_eq!(result.duration_ms, 150);
        assert_eq!(result.response_headers, Some(headers));
        assert_eq!(result.response_body, Some("{\"ok\": true}".to_string()));
    }

    #[test]
    fn test_build_temp_result_for_assertions() {
        let request = create_test_request();
        let result = build_temp_result_for_assertions(&request, 201, true, 75, None, None);

        assert_eq!(result.status_code, 201);
        assert!(result.success);
        assert!(result.assertion_results.is_empty());
    }
}
