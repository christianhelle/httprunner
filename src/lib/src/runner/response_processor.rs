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

    #[test]
    fn test_extract_headers_empty() {
        let headers = reqwest::header::HeaderMap::new();
        let result = extract_headers(&headers);
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_headers_single() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        let result = extract_headers(&headers);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.get("content-type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_extract_headers_multiple() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        headers.insert(reqwest::header::CONTENT_LENGTH, "1234".parse().unwrap());
        headers.insert(
            reqwest::header::HeaderName::from_static("x-custom"),
            "custom-value".parse().unwrap(),
        );
        let result = extract_headers(&headers);
        assert_eq!(result.len(), 3);
        assert!(result.contains_key("content-type"));
        assert!(result.contains_key("content-length"));
        assert!(result.contains_key("x-custom"));
    }

    #[test]
    fn test_build_success_result_with_headers_and_body() {
        let request = create_test_request();
        let mut headers = HashMap::new();
        headers.insert("server".to_string(), "nginx".to_string());
        headers.insert("content-type".to_string(), "text/plain".to_string());

        let result = build_success_result(
            &request,
            201,
            true,
            250,
            Some(headers.clone()),
            Some("Created".to_string()),
            Vec::new(),
        );

        assert_eq!(result.status_code, 201);
        assert!(result.success);
        assert_eq!(result.duration_ms, 250);
        assert_eq!(result.response_headers, Some(headers));
        assert_eq!(result.response_body, Some("Created".to_string()));
    }

    #[test]
    fn test_build_success_result_without_headers_and_body() {
        let request = create_test_request();

        let result = build_success_result(&request, 204, true, 100, None, None, Vec::new());

        assert_eq!(result.status_code, 204);
        assert!(result.success);
        assert!(result.response_headers.is_none());
        assert!(result.response_body.is_none());
    }

    #[test]
    fn test_build_success_result_with_assertions() {
        let request = create_test_request();
        let assertion_results = vec![crate::types::AssertionResult {
            assertion: crate::types::Assertion {
                assertion_type: AssertionType::Status,
                expected_value: "200".to_string(),
            },
            passed: true,
            actual_value: Some("200".to_string()),
            error_message: None,
        }];

        let result = build_success_result(
            &request,
            200,
            true,
            150,
            None,
            None,
            assertion_results.clone(),
        );

        assert_eq!(result.assertion_results.len(), 1);
        assert!(result.assertion_results[0].passed);
    }

    #[test]
    fn test_build_error_result_with_long_message() {
        let request = create_test_request();
        let long_message = "Error: ".to_string() + &"x".repeat(1000);
        let result = build_error_result(&request, &long_message, 5000);

        assert_eq!(result.status_code, 0);
        assert!(!result.success);
        assert!(result.error_message.is_some());
        assert_eq!(result.duration_ms, 5000);
    }

    #[test]
    fn test_build_temp_result_with_all_fields() {
        let request = create_test_request();
        let mut headers = HashMap::new();
        headers.insert("test".to_string(), "value".to_string());

        let result = build_temp_result_for_assertions(
            &request,
            200,
            true,
            100,
            Some(headers.clone()),
            Some("body".to_string()),
        );

        assert_eq!(result.status_code, 200);
        assert!(result.success);
        assert_eq!(result.duration_ms, 100);
        assert_eq!(result.response_headers, Some(headers));
        assert_eq!(result.response_body, Some("body".to_string()));
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_should_capture_response_all_conditions_false() {
        let mut request = create_test_request();
        request.name = None;
        request.assertions.clear();

        assert!(!should_capture_response(&request, false));
    }

    #[test]
    fn test_should_capture_response_verbose_overrides_all() {
        let mut request = create_test_request();
        request.name = None;
        request.assertions.clear();

        assert!(should_capture_response(&request, true));
    }

    #[test]
    fn test_build_success_result_failure_status() {
        let request = create_test_request();

        let result = build_success_result(
            &request,
            404,
            false,
            200,
            None,
            Some("Not Found".to_string()),
            Vec::new(),
        );

        assert_eq!(result.status_code, 404);
        assert!(!result.success);
        assert_eq!(result.response_body, Some("Not Found".to_string()));
    }

    #[test]
    fn test_build_success_result_server_error() {
        let request = create_test_request();

        let result = build_success_result(
            &request,
            500,
            false,
            300,
            None,
            Some("Internal Server Error".to_string()),
            Vec::new(),
        );

        assert_eq!(result.status_code, 500);
        assert!(!result.success);
    }

    #[test]
    fn test_build_error_result_zero_duration() {
        let request = create_test_request();
        let result = build_error_result(&request, "Immediate failure", 0);

        assert_eq!(result.duration_ms, 0);
        assert!(!result.success);
    }

    #[test]
    fn test_build_success_result_redirect_status() {
        let request = create_test_request();

        let result = build_success_result(&request, 302, true, 150, None, None, Vec::new());

        assert_eq!(result.status_code, 302);
    }

    #[test]
    fn test_extract_headers_with_multiple_same_name() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::SET_COOKIE,
            "cookie1=value1".parse().unwrap(),
        );
        // In HeaderMap, inserting the same key replaces the value
        headers.append(
            reqwest::header::SET_COOKIE,
            "cookie2=value2".parse().unwrap(),
        );

        let result = extract_headers(&headers);
        // Should have the set-cookie header
        assert!(result.contains_key("set-cookie"));
    }

    #[test]
    fn test_build_temp_result_preserves_request_name() {
        let mut request = create_test_request();
        request.name = Some("my_custom_request".to_string());

        let result = build_temp_result_for_assertions(&request, 200, true, 100, None, None);

        assert_eq!(result.request_name, Some("my_custom_request".to_string()));
    }

    #[test]
    fn test_build_temp_result_without_request_name() {
        let mut request = create_test_request();
        request.name = None;

        let result = build_temp_result_for_assertions(&request, 200, true, 100, None, None);

        assert!(result.request_name.is_none());
    }
}
