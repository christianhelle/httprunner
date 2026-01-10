use crate::runner::HttpExecutor;
use crate::types::{HttpRequest, HttpResult};
use anyhow::Result;
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;
use std::io::Write;

/// Mock HTTP executor for testing the processor
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
    use super::*;
    use super::super::executor::process_http_files_with_executor;

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

        let mock = MockHttpExecutor::new(vec![create_success_response(Some("testReq".to_string()))]);

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
}
