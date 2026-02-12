use crate::types::{HttpRequest, HttpResult};
use anyhow::Result;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;

struct MockHttpExecutor {
    responses: Arc<Mutex<Vec<HttpResult>>>,
    call_count: Arc<Mutex<usize>>,
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

    #[allow(dead_code)]
    fn get_executed_requests(&self) -> Vec<HttpRequest> {
        self.executed_requests.lock().unwrap().clone()
    }

    fn execute(
        &self,
        request: &HttpRequest,
        _verbose: bool,
        _insecure: bool,
    ) -> Result<HttpResult> {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;

        self.executed_requests.lock().unwrap().push(request.clone());

        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            Ok(HttpResult {
                request_name: request.name.clone(),
                status_code: 200,
                success: true,
                error_message: None,
                duration_ms: 1,
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
    use super::super::executor::process_http_files_with_executor;
    use super::*;

    fn create_success_response(name: Option<String>) -> HttpResult {
        HttpResult {
            request_name: name,
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 1,
            response_headers: None,
            response_body: Some(r#"{"result":"ok"}"#.to_string()),
            assertion_results: Vec::new(),
        }
    }

    fn create_temp_http_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_single_request_with_mock() {
        let file_content = "GET https://api.example.com/test\n";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![create_success_response(None)]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(res.success);
        assert_eq!(res.files[0].success_count, 1);
        assert_eq!(mock.get_call_count(), 1);
    }

    #[test]
    fn test_multiple_requests_with_mock() {
        let file_content = r#"
GET https://api.example.com/1
###
POST https://api.example.com/2
###
DELETE https://api.example.com/3
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![
            create_success_response(None),
            create_success_response(None),
            create_success_response(None),
        ]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(res.success);
        assert_eq!(res.files[0].success_count, 3);
        assert_eq!(mock.get_call_count(), 3);
    }

    #[test]
    fn test_failed_request_with_mock() {
        let file_content = "GET https://api.example.com/error\n";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![HttpResult {
            request_name: None,
            status_code: 500,
            success: false,
            error_message: Some("Internal Server Error".to_string()),
            duration_ms: 1,
            response_headers: None,
            response_body: None,
            assertion_results: Vec::new(),
        }]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(!res.success);
        assert_eq!(res.files[0].failed_count, 1);
    }

    #[test]
    fn test_named_request_with_mock() {
        let file_content = r#"
# @name testReq
GET https://api.example.com/named
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock =
            MockHttpExecutor::new(vec![create_success_response(Some("testReq".to_string()))]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(res.files[0].result_contexts[0].name, "testReq");
    }

    #[test]
    fn test_empty_file() {
        let temp_file = create_temp_http_file("");
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        assert_eq!(mock.get_call_count(), 0);
    }

    #[test]
    fn test_verbose_mode() {
        let file_content = "GET https://api.example.com/test\n";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![create_success_response(None)]);

        let result = process_http_files_with_executor(
            &[file_path],
            true,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_insecure_flag_passed_to_executor() {
        let file_content = "GET https://selfsigned.example.com/test\n";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![create_success_response(None)]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            true,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_files() {
        let file1 = create_temp_http_file("GET https://api.example.com/1\n");
        let file2 = create_temp_http_file("GET https://api.example.com/2\n");

        let path1 = file1.path().to_str().unwrap().to_string();
        let path2 = file2.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![
            create_success_response(None),
            create_success_response(None),
        ]);

        let result = process_http_files_with_executor(
            &[path1, path2],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(res.files.len(), 2);
        assert_eq!(mock.get_call_count(), 2);
    }

    #[test]
    fn test_request_with_headers() {
        let file_content = r#"
GET https://api.example.com/test
Authorization: Bearer token123
Content-Type: application/json
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![create_success_response(None)]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let executed = mock.get_executed_requests();
        assert_eq!(executed[0].headers.len(), 2);
    }

    #[test]
    fn test_request_with_body() {
        let file_content = r#"
POST https://api.example.com/data
Content-Type: application/json

{"key":"value"}
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![create_success_response(None)]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        assert_eq!(mock.get_call_count(), 1);
        let res = result.unwrap();
        assert_eq!(res.files[0].success_count, 1);
    }

    #[test]
    fn test_all_http_methods() {
        let file_content = r#"
GET https://api.example.com/test
###
POST https://api.example.com/test
###
PUT https://api.example.com/test
###
PATCH https://api.example.com/test
###
DELETE https://api.example.com/test
###
HEAD https://api.example.com/test
###
OPTIONS https://api.example.com/test
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let responses: Vec<_> = (0..7).map(|_| create_success_response(None)).collect();
        let mock = MockHttpExecutor::new(responses);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        assert_eq!(mock.get_call_count(), 7);
    }

    #[test]
    fn test_dependency_execution() {
        let file_content = r#"
# @name setup
GET https://api.example.com/setup
###
# @dependsOn setup
GET https://api.example.com/data
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![
            create_success_response(Some("setup".to_string())),
            create_success_response(None),
        ]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        assert_eq!(mock.get_call_count(), 2);
    }

    #[test]
    fn test_skipped_due_to_failed_dependency() {
        let file_content = r#"
# @name setup
GET https://api.example.com/setup
###
# @dependsOn setup
GET https://api.example.com/data
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![HttpResult {
            request_name: Some("setup".to_string()),
            status_code: 500,
            success: false,
            error_message: None,
            duration_ms: 1,
            response_headers: None,
            response_body: None,
            assertion_results: Vec::new(),
        }]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(mock.get_call_count(), 1);
        assert_eq!(res.files[0].skipped_count, 1);
    }

    #[test]
    fn test_request_with_timeout() {
        let file_content = r#"
# @timeout 5000ms
GET https://api.example.com/slow
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![create_success_response(None)]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let executed = mock.get_executed_requests();
        assert_eq!(executed[0].timeout, Some(5000));
    }

    #[test]
    fn test_request_with_connection_timeout() {
        let file_content = r#"
# @connection-timeout 3000ms
GET https://api.example.com/test
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![create_success_response(None)]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let executed = mock.get_executed_requests();
        assert_eq!(executed[0].connection_timeout, Some(3000));
    }

    #[test]
    fn test_request_with_assertions() {
        use crate::types::{Assertion, AssertionResult, AssertionType};

        let file_content = r#"
GET https://api.example.com/test
> EXPECTED_RESPONSE_STATUS 200
> EXPECTED_RESPONSE_BODY "ok"
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mut result_with_assertions = create_success_response(None);
        result_with_assertions.assertion_results = vec![
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
                    expected_value: "ok".to_string(),
                },
                passed: true,
                actual_value: Some("ok".to_string()),
                error_message: None,
            },
        ];

        let mock = MockHttpExecutor::new(vec![result_with_assertions]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let executed = mock.get_executed_requests();
        // Parser creates assertions based on EXPECTED_RESPONSE_* directives
        assert!(!executed[0].assertions.is_empty());
    }

    #[test]
    fn test_request_parsing_complex() {
        let file_content = r#"
# @name getUser
GET https://api.example.com/user
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock =
            MockHttpExecutor::new(vec![create_success_response(Some("getUser".to_string()))]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let executed = mock.get_executed_requests();
        assert_eq!(executed[0].name, Some("getUser".to_string()));
    }

    #[test]
    fn test_mixed_success_and_failure() {
        let file_content = r#"
GET https://api.example.com/success
###
GET https://api.example.com/fail
###
GET https://api.example.com/success2
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![
            create_success_response(None),
            HttpResult {
                request_name: None,
                status_code: 404,
                success: false,
                error_message: Some("Not Found".to_string()),
                duration_ms: 1,
                response_headers: None,
                response_body: None,
                assertion_results: Vec::new(),
            },
            create_success_response(None),
        ]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(!res.success);
        assert_eq!(res.files[0].success_count, 2);
        assert_eq!(res.files[0].failed_count, 1);
        assert_eq!(mock.get_call_count(), 3);
    }

    #[test]
    fn test_pretty_json_mode() {
        let file_content = r#"
POST https://api.example.com/data
Content-Type: application/json

{"key":"value"}
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![create_success_response(None)]);

        let result = process_http_files_with_executor(
            &[file_path],
            true,
            None,
            None,
            false,
            true,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_log_file_creation() {
        use std::fs;

        let file_content = "GET https://api.example.com/test\n";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![create_success_response(None)]);

        let log_base = "test_log_executor";

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            Some(log_base),
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());

        // Find the generated log file (it has a timestamp suffix)
        let entries = fs::read_dir(".").unwrap();
        let log_files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name().to_string_lossy().starts_with(log_base)
                    && e.file_name().to_string_lossy().ends_with(".log")
            })
            .collect();

        assert!(!log_files.is_empty(), "Log file should be created");

        // Read and verify the log content
        let log_path = log_files[0].path();
        let log_content = fs::read_to_string(&log_path).unwrap();
        assert!(!log_content.is_empty());
        assert!(log_content.contains("https://api.example.com/test"));

        // Clean up
        let _ = fs::remove_file(&log_path);
    }

    #[test]
    fn test_environment_parameter() {
        let file_content = r#"
GET https://api.example.com/test
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![create_success_response(None)]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            Some("production"),
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_chain_of_dependencies() {
        let file_content = r#"
# @name first
GET https://api.example.com/first
###
# @name second
# @dependsOn first
GET https://api.example.com/second
###
# @dependsOn second
GET https://api.example.com/third
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![
            create_success_response(Some("first".to_string())),
            create_success_response(Some("second".to_string())),
            create_success_response(None),
        ]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(mock.get_call_count(), 3);
        assert_eq!(res.files[0].success_count, 3);
    }

    #[test]
    fn test_failed_dependency_in_chain() {
        let file_content = r#"
# @name first
GET https://api.example.com/first
###
# @name second
# @dependsOn first
GET https://api.example.com/second
###
# @dependsOn second
GET https://api.example.com/third
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![
            create_success_response(Some("first".to_string())),
            HttpResult {
                request_name: Some("second".to_string()),
                status_code: 500,
                success: false,
                error_message: None,
                duration_ms: 1,
                response_headers: None,
                response_body: None,
                assertion_results: Vec::new(),
            },
        ]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(mock.get_call_count(), 2);
        assert_eq!(res.files[0].skipped_count, 1);
    }

    #[test]
    fn test_invalid_http_file() {
        let file_content = "This is not a valid HTTP request format";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        // Should handle parse errors gracefully
        assert!(result.is_ok());
        assert_eq!(mock.get_call_count(), 0);
    }

    #[test]
    fn test_request_with_response_headers() {
        use std::collections::HashMap;

        let file_content = "GET https://api.example.com/test\n";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());

        let response = HttpResult {
            request_name: None,
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 1,
            response_headers: Some(headers),
            response_body: Some(r#"{"status":"ok"}"#.to_string()),
            assertion_results: Vec::new(),
        };

        let mock = MockHttpExecutor::new(vec![response]);

        let result = process_http_files_with_executor(
            &[file_path],
            true,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(
            res.files[0].result_contexts[0]
                .result
                .as_ref()
                .unwrap()
                .response_headers
                .is_some()
        );
    }

    #[test]
    fn test_verbose_mode_with_request_details() {
        let file_content = r#"
# @name verboseTest
POST https://api.example.com/data
Content-Type: application/json

{"test":"data"}
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![create_success_response(Some(
            "verboseTest".to_string(),
        ))]);

        let result = process_http_files_with_executor(
            &[file_path],
            true,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        assert_eq!(mock.get_call_count(), 1);
    }

    #[test]
    fn test_verbose_mode_with_response_details() {
        use std::collections::HashMap;

        let file_content = "GET https://api.example.com/test\n";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let response = HttpResult {
            request_name: None,
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 150,
            response_headers: Some(headers),
            response_body: Some(r#"{"result":"success"}"#.to_string()),
            assertion_results: Vec::new(),
        };

        let mock = MockHttpExecutor::new(vec![response]);

        let result = process_http_files_with_executor(
            &[file_path],
            true,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_verbose_mode_with_pretty_json() {
        let file_content = r#"
POST https://api.example.com/data
Content-Type: application/json

{"nested":{"key":"value"},"array":[1,2,3]}
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let response = HttpResult {
            request_name: None,
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 100,
            response_headers: None,
            response_body: Some(r#"{"status":"ok","data":{"id":123}}"#.to_string()),
            assertion_results: Vec::new(),
        };

        let mock = MockHttpExecutor::new(vec![response]);

        let result = process_http_files_with_executor(
            &[file_path],
            true,
            None,
            None,
            false,
            true,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_verbose_mode_with_assertions_passed() {
        use crate::types::{Assertion, AssertionResult, AssertionType};

        let file_content = r#"
GET https://api.example.com/test
> EXPECTED_RESPONSE_STATUS 200
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mut response = create_success_response(None);
        response.assertion_results = vec![AssertionResult {
            assertion: Assertion {
                assertion_type: AssertionType::Status,
                expected_value: "200".to_string(),
            },
            passed: true,
            actual_value: Some("200".to_string()),
            error_message: None,
        }];

        let mock = MockHttpExecutor::new(vec![response]);

        let result = process_http_files_with_executor(
            &[file_path],
            true,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_verbose_mode_with_assertions_failed() {
        use crate::types::{Assertion, AssertionResult, AssertionType};

        let file_content = r#"
GET https://api.example.com/test
> EXPECTED_RESPONSE_STATUS 200
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let response = HttpResult {
            request_name: None,
            status_code: 404,
            success: false,
            error_message: None,
            duration_ms: 50,
            response_headers: None,
            response_body: None,
            assertion_results: vec![AssertionResult {
                assertion: Assertion {
                    assertion_type: AssertionType::Status,
                    expected_value: "200".to_string(),
                },
                passed: false,
                actual_value: Some("404".to_string()),
                error_message: Some("Expected 200, got 404".to_string()),
            }],
        };

        let mock = MockHttpExecutor::new(vec![response]);

        let result = process_http_files_with_executor(
            &[file_path],
            true,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(!res.success);
    }

    #[test]
    fn test_condition_evaluation_verbose_mode_met() {
        let file_content = r#"
# @name setup
GET https://api.example.com/setup
###
# @if setup.response.status 200
GET https://api.example.com/data
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![
            create_success_response(Some("setup".to_string())),
            create_success_response(None),
        ]);

        let result = process_http_files_with_executor(
            &[file_path],
            true,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        assert_eq!(mock.get_call_count(), 2);
    }

    #[test]
    fn test_condition_evaluation_verbose_mode_not_met() {
        let file_content = r#"
# @name setup
GET https://api.example.com/setup
###
# @if setup.response.status 404
GET https://api.example.com/data
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![create_success_response(Some("setup".to_string()))]);

        let result = process_http_files_with_executor(
            &[file_path],
            true,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(mock.get_call_count(), 1);
        assert_eq!(res.files[0].skipped_count, 1);
    }

    #[test]
    fn test_condition_evaluation_non_verbose_mode() {
        let file_content = r#"
# @name setup
GET https://api.example.com/setup
###
# @if setup.response.status 200
GET https://api.example.com/data
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![
            create_success_response(Some("setup".to_string())),
            create_success_response(None),
        ]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        assert_eq!(mock.get_call_count(), 2);
    }

    #[test]
    fn test_executor_error_handling() {
        let file_content = "GET https://api.example.com/test\n";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        // Executor that always fails
        let failing_executor =
            |_req: &HttpRequest, _v: bool, _i: bool| Err(anyhow::anyhow!("Network error"));

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &failing_executor,
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(res.files[0].failed_count, 1);
        assert_eq!(res.files[0].success_count, 0);
    }

    #[test]
    fn test_failed_request_with_error_message() {
        let file_content = "GET https://api.example.com/error\n";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let response = HttpResult {
            request_name: None,
            status_code: 500,
            success: false,
            error_message: Some("Internal Server Error: Database connection failed".to_string()),
            duration_ms: 250,
            response_headers: None,
            response_body: None,
            assertion_results: Vec::new(),
        };

        let mock = MockHttpExecutor::new(vec![response]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(!res.success);
        assert_eq!(res.files[0].failed_count, 1);
    }

    #[test]
    fn test_failed_request_without_error_message() {
        let file_content = "GET https://api.example.com/error\n";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let response = HttpResult {
            request_name: None,
            status_code: 404,
            success: false,
            error_message: None,
            duration_ms: 50,
            response_headers: None,
            response_body: None,
            assertion_results: Vec::new(),
        };

        let mock = MockHttpExecutor::new(vec![response]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(!res.success);
    }

    #[test]
    fn test_multiple_files_with_overall_summary() {
        let file1 = create_temp_http_file("GET https://api.example.com/1\n");
        let file2 = create_temp_http_file("GET https://api.example.com/2\n");
        let file3 = create_temp_http_file("GET https://api.example.com/3\n");

        let path1 = file1.path().to_str().unwrap().to_string();
        let path2 = file2.path().to_str().unwrap().to_string();
        let path3 = file3.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![
            create_success_response(None),
            create_success_response(None),
            create_success_response(None),
        ]);

        let result = process_http_files_with_executor(
            &[path1, path2, path3],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(res.files.len(), 3);
        assert!(res.success);
    }

    #[test]
    fn test_request_without_name_generates_context_name() {
        let file_content = "GET https://api.example.com/test\n";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![create_success_response(None)]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(res.files[0].result_contexts[0].name, "request_1");
    }

    #[test]
    fn test_parse_error_continues_processing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let invalid_file = temp_dir.path().join("invalid.http");
        let valid_file = temp_dir.path().join("valid.http");

        std::fs::write(&invalid_file, "INVALID REQUEST FORMAT").unwrap();
        std::fs::write(&valid_file, "GET https://api.example.com/test").unwrap();

        let mock = MockHttpExecutor::new(vec![create_success_response(None)]);

        let result = process_http_files_with_executor(
            &[
                invalid_file.to_str().unwrap().to_string(),
                valid_file.to_str().unwrap().to_string(),
            ],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        // Should process the valid file even though the first one failed
        assert_eq!(mock.get_call_count(), 1);
    }

    #[test]
    fn test_verbose_mode_without_body() {
        let file_content = r#"
GET https://api.example.com/test
Authorization: Bearer token123
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![create_success_response(None)]);

        let result = process_http_files_with_executor(
            &[file_path],
            true,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_verbose_mode_response_without_body() {
        let file_content = "HEAD https://api.example.com/test\n";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let response = HttpResult {
            request_name: None,
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 50,
            response_headers: None,
            response_body: None,
            assertion_results: Vec::new(),
        };

        let mock = MockHttpExecutor::new(vec![response]);

        let result = process_http_files_with_executor(
            &[file_path],
            true,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_named_request_with_verbose_mode() {
        let file_content = r#"
# @name myRequest
POST https://api.example.com/create
Content-Type: application/json

{"name":"test"}
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock =
            MockHttpExecutor::new(vec![create_success_response(Some("myRequest".to_string()))]);

        let result = process_http_files_with_executor(
            &[file_path],
            true,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(res.files[0].result_contexts[0].name, "myRequest");
    }

    // Tests for assertion-based success determination.
    // Requests with assertions should be treated as successful when all assertions pass,
    // even if the HTTP status code is non-2xx (e.g., testing for expected 400/404 responses).

    #[test]
    fn test_non_2xx_with_passing_assertions_counts_as_success() {
        let file_content = "GET https://api.example.com/bad-request\n";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![HttpResult {
            request_name: None,
            status_code: 400,
            success: true, // assertions all passed, so success = true
            error_message: None,
            duration_ms: 5,
            response_headers: None,
            response_body: Some(r#"{"error":"bad request"}"#.to_string()),
            assertion_results: vec![crate::types::AssertionResult {
                assertion: crate::types::Assertion {
                    assertion_type: crate::types::AssertionType::Status,
                    expected_value: "400".to_string(),
                },
                passed: true,
                actual_value: Some("400".to_string()),
                error_message: None,
            }],
        }]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(res.success);
        assert_eq!(res.files[0].success_count, 1);
        assert_eq!(res.files[0].failed_count, 0);
    }

    #[test]
    fn test_404_with_passing_status_assertion_counts_as_success() {
        let file_content = "GET https://api.example.com/not-found\n";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![HttpResult {
            request_name: None,
            status_code: 404,
            success: true,
            error_message: None,
            duration_ms: 3,
            response_headers: None,
            response_body: Some(r#"{"error":"not found"}"#.to_string()),
            assertion_results: vec![crate::types::AssertionResult {
                assertion: crate::types::Assertion {
                    assertion_type: crate::types::AssertionType::Status,
                    expected_value: "404".to_string(),
                },
                passed: true,
                actual_value: Some("404".to_string()),
                error_message: None,
            }],
        }]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(res.success);
        assert_eq!(res.files[0].success_count, 1);
        assert_eq!(res.files[0].failed_count, 0);
    }

    #[test]
    fn test_500_with_passing_assertions_counts_as_success() {
        let file_content = "GET https://api.example.com/error\n";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![HttpResult {
            request_name: None,
            status_code: 500,
            success: true,
            error_message: None,
            duration_ms: 10,
            response_headers: None,
            response_body: Some("Internal Server Error".to_string()),
            assertion_results: vec![crate::types::AssertionResult {
                assertion: crate::types::Assertion {
                    assertion_type: crate::types::AssertionType::Status,
                    expected_value: "500".to_string(),
                },
                passed: true,
                actual_value: Some("500".to_string()),
                error_message: None,
            }],
        }]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(res.success);
        assert_eq!(res.files[0].success_count, 1);
        assert_eq!(res.files[0].failed_count, 0);
    }

    #[test]
    fn test_non_2xx_with_failing_assertions_counts_as_failure() {
        let file_content = "GET https://api.example.com/wrong-error\n";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![HttpResult {
            request_name: None,
            status_code: 500,
            success: false, // assertion failed
            error_message: None,
            duration_ms: 8,
            response_headers: None,
            response_body: None,
            assertion_results: vec![crate::types::AssertionResult {
                assertion: crate::types::Assertion {
                    assertion_type: crate::types::AssertionType::Status,
                    expected_value: "400".to_string(),
                },
                passed: false,
                actual_value: Some("500".to_string()),
                error_message: Some("Expected status 400, got 500".to_string()),
            }],
        }]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(!res.success);
        assert_eq!(res.files[0].success_count, 0);
        assert_eq!(res.files[0].failed_count, 1);
    }

    #[test]
    fn test_non_2xx_with_multiple_passing_assertions_counts_as_success() {
        let file_content = "GET https://api.example.com/bad-request\n";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![HttpResult {
            request_name: None,
            status_code: 400,
            success: true,
            error_message: None,
            duration_ms: 5,
            response_headers: None,
            response_body: Some(r#"{"error":"validation failed"}"#.to_string()),
            assertion_results: vec![
                crate::types::AssertionResult {
                    assertion: crate::types::Assertion {
                        assertion_type: crate::types::AssertionType::Status,
                        expected_value: "400".to_string(),
                    },
                    passed: true,
                    actual_value: Some("400".to_string()),
                    error_message: None,
                },
                crate::types::AssertionResult {
                    assertion: crate::types::Assertion {
                        assertion_type: crate::types::AssertionType::Body,
                        expected_value: "validation failed".to_string(),
                    },
                    passed: true,
                    actual_value: Some(r#"{"error":"validation failed"}"#.to_string()),
                    error_message: None,
                },
            ],
        }]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(res.success);
        assert_eq!(res.files[0].success_count, 1);
        assert_eq!(res.files[0].failed_count, 0);
    }

    #[test]
    fn test_non_2xx_with_mixed_assertions_counts_as_failure() {
        let file_content = "GET https://api.example.com/bad-request\n";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        // Status assertion passes but body assertion fails
        let mock = MockHttpExecutor::new(vec![HttpResult {
            request_name: None,
            status_code: 400,
            success: false, // not all assertions passed
            error_message: None,
            duration_ms: 5,
            response_headers: None,
            response_body: Some(r#"{"error":"unexpected error"}"#.to_string()),
            assertion_results: vec![
                crate::types::AssertionResult {
                    assertion: crate::types::Assertion {
                        assertion_type: crate::types::AssertionType::Status,
                        expected_value: "400".to_string(),
                    },
                    passed: true,
                    actual_value: Some("400".to_string()),
                    error_message: None,
                },
                crate::types::AssertionResult {
                    assertion: crate::types::Assertion {
                        assertion_type: crate::types::AssertionType::Body,
                        expected_value: "validation failed".to_string(),
                    },
                    passed: false,
                    actual_value: Some(r#"{"error":"unexpected error"}"#.to_string()),
                    error_message: Some("Expected body to contain 'validation failed'".to_string()),
                },
            ],
        }]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(!res.success);
        assert_eq!(res.files[0].success_count, 0);
        assert_eq!(res.files[0].failed_count, 1);
    }

    #[test]
    fn test_2xx_with_failing_assertions_counts_as_failure() {
        let file_content = "GET https://api.example.com/test\n";
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        // 200 OK but assertion expects different status
        let mock = MockHttpExecutor::new(vec![HttpResult {
            request_name: None,
            status_code: 200,
            success: false, // assertion failed
            error_message: None,
            duration_ms: 5,
            response_headers: None,
            response_body: Some("OK".to_string()),
            assertion_results: vec![crate::types::AssertionResult {
                assertion: crate::types::Assertion {
                    assertion_type: crate::types::AssertionType::Status,
                    expected_value: "201".to_string(),
                },
                passed: false,
                actual_value: Some("200".to_string()),
                error_message: Some("Expected status 201, got 200".to_string()),
            }],
        }]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(!res.success);
        assert_eq!(res.files[0].success_count, 0);
        assert_eq!(res.files[0].failed_count, 1);
    }

    #[test]
    fn test_multiple_requests_mixed_assertion_success() {
        let file_content = r#"
GET https://api.example.com/ok
###
GET https://api.example.com/bad-request
###
GET https://api.example.com/not-found
"#;
        let temp_file = create_temp_http_file(file_content);
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let mock = MockHttpExecutor::new(vec![
            // First request: 200 with passing assertion
            HttpResult {
                request_name: None,
                status_code: 200,
                success: true,
                error_message: None,
                duration_ms: 5,
                response_headers: None,
                response_body: Some("OK".to_string()),
                assertion_results: vec![crate::types::AssertionResult {
                    assertion: crate::types::Assertion {
                        assertion_type: crate::types::AssertionType::Status,
                        expected_value: "200".to_string(),
                    },
                    passed: true,
                    actual_value: Some("200".to_string()),
                    error_message: None,
                }],
            },
            // Second request: 400 with passing assertion (expected bad request)
            HttpResult {
                request_name: None,
                status_code: 400,
                success: true, // all assertions passed
                error_message: None,
                duration_ms: 5,
                response_headers: None,
                response_body: Some(r#"{"error":"bad request"}"#.to_string()),
                assertion_results: vec![crate::types::AssertionResult {
                    assertion: crate::types::Assertion {
                        assertion_type: crate::types::AssertionType::Status,
                        expected_value: "400".to_string(),
                    },
                    passed: true,
                    actual_value: Some("400".to_string()),
                    error_message: None,
                }],
            },
            // Third request: 404 with failing assertion (expected 200)
            HttpResult {
                request_name: None,
                status_code: 404,
                success: false, // assertion failed
                error_message: None,
                duration_ms: 5,
                response_headers: None,
                response_body: None,
                assertion_results: vec![crate::types::AssertionResult {
                    assertion: crate::types::Assertion {
                        assertion_type: crate::types::AssertionType::Status,
                        expected_value: "200".to_string(),
                    },
                    passed: false,
                    actual_value: Some("404".to_string()),
                    error_message: Some("Expected status 200, got 404".to_string()),
                }],
            },
        ]);

        let result = process_http_files_with_executor(
            &[file_path],
            false,
            None,
            None,
            false,
            false,
            &|req, v, i| mock.execute(req, v, i),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(!res.success); // overall failure because one request failed
        assert_eq!(res.files[0].success_count, 2); // 200 and 400 both passed
        assert_eq!(res.files[0].failed_count, 1); // 404 with wrong assertion failed
    }
}
