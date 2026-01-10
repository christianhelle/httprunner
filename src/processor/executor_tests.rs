use crate::runner::HttpExecutor;
use crate::types::{
    Assertion, AssertionResult, AssertionType, Header, HttpRequest,
    HttpResult,
};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock HTTP executor for testing
/// This is the Rust equivalent of a C# test double (mock/stub)
struct MockHttpExecutor {
    /// Predefined responses to return for requests
    responses: Arc<Mutex<Vec<HttpResult>>>,
    /// Track calls made to the executor
    call_count: Arc<Mutex<usize>>,
    /// Track requests that were executed
    executed_requests: Arc<Mutex<Vec<HttpRequest>>>,
}

impl MockHttpExecutor {
    fn new(responses: Vec<HttpResult>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
            call_count: Arc::new(Mutex::new(0)),
            executed_requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }

    fn get_executed_requests(&self) -> Vec<HttpRequest> {
        self.executed_requests.lock().unwrap().clone()
    }
}

impl HttpExecutor for MockHttpExecutor {
    fn execute(&self, request: &HttpRequest, _verbose: bool, _insecure: bool) -> Result<HttpResult> {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;

        self.executed_requests
            .lock()
            .unwrap()
            .push(request.clone());

        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            // Default response if none configured
            Ok(HttpResult {
                request_name: request.name.clone(),
                status_code: 200,
                success: true,
                error_message: None,
                duration_ms: 100,
                response_headers: None,
                response_body: Some(r#"{"status":"ok"}"#.to_string()),
                assertion_results: Vec::new(),
            })
        } else {
            Ok(responses.remove(0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_simple_request(name: &str, method: &str, url: &str) -> HttpRequest {
        HttpRequest {
            name: Some(name.to_string()),
            method: method.to_string(),
            url: url.to_string(),
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

    fn create_http_result(
        request_name: Option<String>,
        status_code: u16,
        success: bool,
        body: Option<String>,
    ) -> HttpResult {
        HttpResult {
            request_name,
            status_code,
            success,
            error_message: None,
            duration_ms: 100,
            response_headers: None,
            response_body: body,
            assertion_results: Vec::new(),
        }
    }

    #[test]
    fn test_mock_executor_basic_request() {
        let mock_response = create_http_result(
            Some("test".to_string()),
            200,
            true,
            Some(r#"{"result":"success"}"#.to_string()),
        );
        let mock = MockHttpExecutor::new(vec![mock_response.clone()]);

        let request = create_simple_request("test", "GET", "https://api.example.com/test");
        let result = mock.execute(&request, false, false).unwrap();

        assert_eq!(result.status_code, 200);
        assert!(result.success);
        assert_eq!(result.response_body.unwrap(), r#"{"result":"success"}"#);
        assert_eq!(mock.get_call_count(), 1);
    }

    #[test]
    fn test_mock_executor_multiple_requests() {
        let responses = vec![
            create_http_result(Some("req1".to_string()), 200, true, Some("body1".to_string())),
            create_http_result(Some("req2".to_string()), 201, true, Some("body2".to_string())),
            create_http_result(Some("req3".to_string()), 204, true, None),
        ];
        let mock = MockHttpExecutor::new(responses);

        let req1 = create_simple_request("req1", "GET", "https://api.example.com/1");
        let req2 = create_simple_request("req2", "POST", "https://api.example.com/2");
        let req3 = create_simple_request("req3", "DELETE", "https://api.example.com/3");

        let result1 = mock.execute(&req1, false, false).unwrap();
        let result2 = mock.execute(&req2, false, false).unwrap();
        let result3 = mock.execute(&req3, false, false).unwrap();

        assert_eq!(result1.status_code, 200);
        assert_eq!(result1.response_body.unwrap(), "body1");

        assert_eq!(result2.status_code, 201);
        assert_eq!(result2.response_body.unwrap(), "body2");

        assert_eq!(result3.status_code, 204);
        assert!(result3.response_body.is_none());

        assert_eq!(mock.get_call_count(), 3);
    }

    #[test]
    fn test_mock_executor_tracks_executed_requests() {
        let mock = MockHttpExecutor::new(vec![
            create_http_result(None, 200, true, None),
            create_http_result(None, 200, true, None),
        ]);

        let req1 = create_simple_request("first", "GET", "https://api.example.com/first");
        let req2 = create_simple_request("second", "POST", "https://api.example.com/second");

        mock.execute(&req1, false, false).unwrap();
        mock.execute(&req2, false, false).unwrap();

        let executed = mock.get_executed_requests();
        assert_eq!(executed.len(), 2);
        assert_eq!(executed[0].name.as_ref().unwrap(), "first");
        assert_eq!(executed[0].method, "GET");
        assert_eq!(executed[1].name.as_ref().unwrap(), "second");
        assert_eq!(executed[1].method, "POST");
    }

    #[test]
    fn test_mock_executor_error_response() {
        let error_response = HttpResult {
            request_name: Some("error_test".to_string()),
            status_code: 500,
            success: false,
            error_message: Some("Internal Server Error".to_string()),
            duration_ms: 50,
            response_headers: None,
            response_body: Some(r#"{"error":"Something went wrong"}"#.to_string()),
            assertion_results: Vec::new(),
        };
        let mock = MockHttpExecutor::new(vec![error_response]);

        let request = create_simple_request("error_test", "GET", "https://api.example.com/error");
        let result = mock.execute(&request, false, false).unwrap();

        assert_eq!(result.status_code, 500);
        assert!(!result.success);
        assert_eq!(
            result.error_message.unwrap(),
            "Internal Server Error"
        );
    }

    #[test]
    fn test_mock_executor_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());

        let response = HttpResult {
            request_name: Some("headers_test".to_string()),
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 100,
            response_headers: Some(headers.clone()),
            response_body: Some(r#"{"data":"test"}"#.to_string()),
            assertion_results: Vec::new(),
        };
        let mock = MockHttpExecutor::new(vec![response]);

        let request = create_simple_request("headers_test", "GET", "https://api.example.com/test");
        let result = mock.execute(&request, true, false).unwrap();

        assert!(result.response_headers.is_some());
        let result_headers = result.response_headers.unwrap();
        assert_eq!(result_headers.get("Content-Type").unwrap(), "application/json");
        assert_eq!(result_headers.get("X-Custom-Header").unwrap(), "custom-value");
    }

    #[test]
    fn test_mock_executor_with_assertions() {
        let assertion_results = vec![
            AssertionResult {
                assertion: Assertion {
                    assertion_type: AssertionType::Status,
                    expected_value: "200".to_string(),
                },
                passed: true,
                actual_value: Some("200".to_string()),
                error_message: None,
            },
            AssertionResult {
                assertion: Assertion {
                    assertion_type: AssertionType::Body,
                    expected_value: r#"{"status":"ok"}"#.to_string(),
                },
                passed: true,
                actual_value: Some(r#"{"status":"ok"}"#.to_string()),
                error_message: None,
            },
        ];

        let response = HttpResult {
            request_name: Some("assertion_test".to_string()),
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 100,
            response_headers: None,
            response_body: Some(r#"{"status":"ok"}"#.to_string()),
            assertion_results: assertion_results.clone(),
        };
        let mock = MockHttpExecutor::new(vec![response]);

        let request = create_simple_request("assertion_test", "GET", "https://api.example.com/test");
        let result = mock.execute(&request, false, false).unwrap();

        assert_eq!(result.assertion_results.len(), 2);
        assert!(result.assertion_results.iter().all(|r| r.passed));
    }

    #[test]
    fn test_mock_executor_failed_assertion() {
        let assertion_results = vec![AssertionResult {
            assertion: Assertion {
                assertion_type: AssertionType::Status,
                expected_value: "200".to_string(),
            },
            passed: false,
            actual_value: Some("500".to_string()),
            error_message: Some("Expected 200 but got 500".to_string()),
        }];

        let response = HttpResult {
            request_name: Some("failed_assertion".to_string()),
            status_code: 500,
            success: false,
            error_message: None,
            duration_ms: 100,
            response_headers: None,
            response_body: None,
            assertion_results: assertion_results.clone(),
        };
        let mock = MockHttpExecutor::new(vec![response]);

        let request = create_simple_request("failed_assertion", "GET", "https://api.example.com/test");
        let result = mock.execute(&request, false, false).unwrap();

        assert!(!result.success);
        assert_eq!(result.assertion_results.len(), 1);
        assert!(!result.assertion_results[0].passed);
        assert_eq!(
            result.assertion_results[0].error_message.as_ref().unwrap(),
            "Expected 200 but got 500"
        );
    }

    #[test]
    fn test_mock_executor_default_response() {
        // When no responses are configured, should return a default success response
        let mock = MockHttpExecutor::new(vec![]);

        let request = create_simple_request("default", "GET", "https://api.example.com/test");
        let result = mock.execute(&request, false, false).unwrap();

        assert_eq!(result.status_code, 200);
        assert!(result.success);
        assert_eq!(result.response_body.unwrap(), r#"{"status":"ok"}"#);
    }

    #[test]
    fn test_mock_executor_preserves_request_name() {
        let response = create_http_result(
            Some("named_request".to_string()),
            200,
            true,
            Some("test".to_string()),
        );
        let mock = MockHttpExecutor::new(vec![response]);

        let request = create_simple_request("named_request", "GET", "https://api.example.com/test");
        let result = mock.execute(&request, false, false).unwrap();

        assert_eq!(result.request_name.unwrap(), "named_request");
    }

    #[test]
    fn test_mock_executor_with_request_body() {
        let mock = MockHttpExecutor::new(vec![create_http_result(None, 201, true, None)]);

        let mut request = create_simple_request("with_body", "POST", "https://api.example.com/create");
        request.body = Some(r#"{"name":"John","age":30}"#.to_string());

        mock.execute(&request, false, false).unwrap();

        let executed = mock.get_executed_requests();
        assert_eq!(executed.len(), 1);
        assert_eq!(
            executed[0].body.as_ref().unwrap(),
            r#"{"name":"John","age":30}"#
        );
    }

    #[test]
    fn test_mock_executor_with_request_headers() {
        let mock = MockHttpExecutor::new(vec![create_http_result(None, 200, true, None)]);

        let mut request = create_simple_request("with_headers", "GET", "https://api.example.com/test");
        request.headers = vec![
            Header {
                name: "Authorization".to_string(),
                value: "Bearer token123".to_string(),
            },
            Header {
                name: "Accept".to_string(),
                value: "application/json".to_string(),
            },
        ];

        mock.execute(&request, false, false).unwrap();

        let executed = mock.get_executed_requests();
        assert_eq!(executed.len(), 1);
        assert_eq!(executed[0].headers.len(), 2);
        assert_eq!(executed[0].headers[0].name, "Authorization");
        assert_eq!(executed[0].headers[0].value, "Bearer token123");
    }

    #[test]
    fn test_mock_executor_timeout_configuration() {
        let mock = MockHttpExecutor::new(vec![create_http_result(None, 200, true, None)]);

        let mut request = create_simple_request("timeout_test", "GET", "https://api.example.com/slow");
        request.timeout = Some(5000); // 5 seconds
        request.connection_timeout = Some(2000); // 2 seconds

        mock.execute(&request, false, false).unwrap();

        let executed = mock.get_executed_requests();
        assert_eq!(executed.len(), 1);
        assert_eq!(executed[0].timeout, Some(5000));
        assert_eq!(executed[0].connection_timeout, Some(2000));
    }

    #[test]
    fn test_mock_executor_connection_error_simulation() {
        let error_response = HttpResult {
            request_name: Some("connection_error".to_string()),
            status_code: 0,
            success: false,
            error_message: Some("Connection error".to_string()),
            duration_ms: 10,
            response_headers: None,
            response_body: None,
            assertion_results: Vec::new(),
        };
        let mock = MockHttpExecutor::new(vec![error_response]);

        let request = create_simple_request("connection_error", "GET", "https://api.example.com/test");
        let result = mock.execute(&request, false, false).unwrap();

        assert_eq!(result.status_code, 0);
        assert!(!result.success);
        assert_eq!(result.error_message.unwrap(), "Connection error");
    }

    #[test]
    fn test_mock_executor_timeout_error_simulation() {
        let timeout_response = HttpResult {
            request_name: Some("timeout".to_string()),
            status_code: 0,
            success: false,
            error_message: Some("Request timeout".to_string()),
            duration_ms: 30000,
            response_headers: None,
            response_body: None,
            assertion_results: Vec::new(),
        };
        let mock = MockHttpExecutor::new(vec![timeout_response]);

        let request = create_simple_request("timeout", "GET", "https://api.example.com/slow");
        let result = mock.execute(&request, false, false).unwrap();

        assert_eq!(result.status_code, 0);
        assert!(!result.success);
        assert_eq!(result.error_message.unwrap(), "Request timeout");
    }

    #[test]
    fn test_mock_executor_various_http_methods() {
        let responses = vec![
            create_http_result(None, 200, true, None), // GET
            create_http_result(None, 201, true, None), // POST
            create_http_result(None, 200, true, None), // PUT
            create_http_result(None, 200, true, None), // PATCH
            create_http_result(None, 204, true, None), // DELETE
            create_http_result(None, 200, true, None), // HEAD
            create_http_result(None, 200, true, None), // OPTIONS
        ];
        let mock = MockHttpExecutor::new(responses);

        let methods = vec!["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];
        for method in &methods {
            let request = create_simple_request("test", method, "https://api.example.com/test");
            let result = mock.execute(&request, false, false).unwrap();
            assert!(result.success, "Failed for method: {}", method);
        }

        assert_eq!(mock.get_call_count(), 7);
    }

    #[test]
    fn test_mock_executor_various_status_codes() {
        let status_codes = vec![
            (200, true),  // OK
            (201, true),  // Created
            (204, true),  // No Content
            (301, false), // Moved Permanently
            (400, false), // Bad Request
            (401, false), // Unauthorized
            (403, false), // Forbidden
            (404, false), // Not Found
            (500, false), // Internal Server Error
            (503, false), // Service Unavailable
        ];

        let responses: Vec<HttpResult> = status_codes
            .iter()
            .map(|(code, success)| create_http_result(None, *code, *success, None))
            .collect();

        let mock = MockHttpExecutor::new(responses);

        for (expected_code, expected_success) in &status_codes {
            let request = create_simple_request("test", "GET", "https://api.example.com/test");
            let result = mock.execute(&request, false, false).unwrap();
            assert_eq!(result.status_code, *expected_code);
            assert_eq!(result.success, *expected_success);
        }
    }

    #[test]
    fn test_mock_executor_insecure_flag_ignored() {
        // The mock should accept the insecure flag but doesn't need to act on it
        let mock = MockHttpExecutor::new(vec![create_http_result(None, 200, true, None)]);

        let request = create_simple_request("insecure", "GET", "https://self-signed.example.com");
        let result = mock.execute(&request, false, true).unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_mock_executor_verbose_flag_ignored() {
        // The mock should accept the verbose flag but doesn't need to act on it differently
        let mock = MockHttpExecutor::new(vec![create_http_result(None, 200, true, None)]);

        let request = create_simple_request("verbose", "GET", "https://api.example.com/test");
        let result_verbose = mock.execute(&request, true, false).unwrap();

        assert!(result_verbose.success);
        assert_eq!(mock.get_call_count(), 1);
    }
}
