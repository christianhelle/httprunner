use super::json_exporter::export_json_to_dir;
use crate::types::{
    Assertion, AssertionResult, AssertionType, Header, HttpFileResults, HttpRequest, HttpResult,
    ProcessorResults, RequestContext,
};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

fn sample_request(name: &str, method: &str, url: &str) -> HttpRequest {
    HttpRequest {
        name: Some(name.to_string()),
        method: method.to_string(),
        url: url.to_string(),
        headers: vec![],
        body: None,
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

fn sample_result(status: u16, success: bool, duration: u64) -> HttpResult {
    HttpResult {
        request_name: Some("test".to_string()),
        status_code: status,
        success,
        error_message: None,
        duration_ms: duration,
        response_headers: None,
        response_body: None,
        assertion_results: vec![],
    }
}


#[test]
fn export_json_creates_file() {
    let tmp = TempDir::new().unwrap();

    let context = RequestContext {
        name: "json_test_1".to_string(),
        request: sample_request("json_test_1", "GET", "https://example.com"),
        result: Some(sample_result(200, true, 100)),
    };

    let results = ProcessorResults {
        success: true,
        files: vec![HttpFileResults {
            filename: "test.http".to_string(),
            success_count: 1,
            failed_count: 0,
            skipped_count: 0,
            result_contexts: vec![context],
        }],
    };

    let filename = export_json_to_dir(&results, Some(tmp.path())).unwrap();
    assert!(filename.starts_with("httprunner_results_"));
    assert!(filename.ends_with(".json"));

    let filepath = tmp.path().join(&filename);
    let content = fs::read_to_string(&filepath).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["files"][0]["filename"], "test.http");
    assert_eq!(parsed["files"][0]["success_count"], 1);
}

#[test]
fn export_json_includes_request_details() {
    let tmp = TempDir::new().unwrap();

    let mut request = sample_request("json_req_detail", "POST", "https://api.example.com/users");
    request.headers = vec![Header {
        name: "Content-Type".to_string(),
        value: "application/json".to_string(),
    }];
    request.body = Some(r#"{"name":"John"}"#.to_string());

    let context = RequestContext {
        name: "json_req_detail".to_string(),
        request,
        result: Some(sample_result(201, true, 150)),
    };

    let results = ProcessorResults {
        success: true,
        files: vec![HttpFileResults {
            filename: "test.http".to_string(),
            success_count: 1,
            failed_count: 0,
            skipped_count: 0,
            result_contexts: vec![context],
        }],
    };

    let filename = export_json_to_dir(&results, Some(tmp.path())).unwrap();
    let content = fs::read_to_string(tmp.path().join(&filename)).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    let req = &parsed["files"][0]["result_contexts"][0]["request"];
    assert_eq!(req["method"], "POST");
    assert_eq!(req["url"], "https://api.example.com/users");
    assert_eq!(req["headers"][0]["name"], "Content-Type");
    assert_eq!(req["body"], r#"{"name":"John"}"#);
}

#[test]
fn export_json_includes_response_details() {
    let tmp = TempDir::new().unwrap();

    let mut result = sample_result(200, true, 250);
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    result.response_headers = Some(headers);
    result.response_body = Some(r#"{"id":1}"#.to_string());

    let context = RequestContext {
        name: "json_resp_detail".to_string(),
        request: sample_request("json_resp_detail", "GET", "https://api.example.com"),
        result: Some(result),
    };

    let results = ProcessorResults {
        success: true,
        files: vec![HttpFileResults {
            filename: "test.http".to_string(),
            success_count: 1,
            failed_count: 0,
            skipped_count: 0,
            result_contexts: vec![context],
        }],
    };

    let filename = export_json_to_dir(&results, Some(tmp.path())).unwrap();
    let content = fs::read_to_string(tmp.path().join(&filename)).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    let res = &parsed["files"][0]["result_contexts"][0]["result"];
    assert_eq!(res["status_code"], 200);
    assert_eq!(res["success"], true);
    assert_eq!(res["duration_ms"], 250);
    assert_eq!(res["response_body"], r#"{"id":1}"#);
    assert_eq!(
        res["response_headers"]["Content-Type"],
        "application/json"
    );
}

#[test]
fn export_json_includes_assertion_results() {
    let tmp = TempDir::new().unwrap();

    let mut result = sample_result(200, true, 100);
    result.assertion_results = vec![AssertionResult {
        assertion: Assertion {
            assertion_type: AssertionType::Status,
            expected_value: "200".to_string(),
        },
        passed: true,
        actual_value: Some("200".to_string()),
        error_message: None,
    }];

    let context = RequestContext {
        name: "json_assert_test".to_string(),
        request: sample_request("json_assert_test", "GET", "https://api.example.com"),
        result: Some(result),
    };

    let results = ProcessorResults {
        success: true,
        files: vec![HttpFileResults {
            filename: "test.http".to_string(),
            success_count: 1,
            failed_count: 0,
            skipped_count: 0,
            result_contexts: vec![context],
        }],
    };

    let filename = export_json_to_dir(&results, Some(tmp.path())).unwrap();
    let content = fs::read_to_string(tmp.path().join(&filename)).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    let assertion = &parsed["files"][0]["result_contexts"][0]["result"]["assertion_results"][0];
    assert_eq!(assertion["passed"], true);
    assert_eq!(assertion["actual_value"], "200");
    assert_eq!(assertion["assertion"]["assertion_type"], "Status");
}

#[test]
fn export_json_handles_failed_requests() {
    let tmp = TempDir::new().unwrap();

    let mut result = sample_result(500, false, 300);
    result.error_message = Some("Internal Server Error".to_string());

    let context = RequestContext {
        name: "json_failed_test".to_string(),
        request: sample_request("json_failed_test", "GET", "https://api.example.com"),
        result: Some(result),
    };

    let results = ProcessorResults {
        success: false,
        files: vec![HttpFileResults {
            filename: "test.http".to_string(),
            success_count: 0,
            failed_count: 1,
            skipped_count: 0,
            result_contexts: vec![context],
        }],
    };

    let filename = export_json_to_dir(&results, Some(tmp.path())).unwrap();
    let content = fs::read_to_string(tmp.path().join(&filename)).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(parsed["success"], false);
    let res = &parsed["files"][0]["result_contexts"][0]["result"];
    assert_eq!(res["success"], false);
    assert_eq!(res["error_message"], "Internal Server Error");
}

#[test]
fn export_json_handles_skipped_requests() {
    let tmp = TempDir::new().unwrap();

    let context = RequestContext {
        name: "json_skipped_test".to_string(),
        request: sample_request("json_skipped_test", "GET", "https://api.example.com"),
        result: None,
    };

    let results = ProcessorResults {
        success: true,
        files: vec![HttpFileResults {
            filename: "test.http".to_string(),
            success_count: 0,
            failed_count: 0,
            skipped_count: 1,
            result_contexts: vec![context],
        }],
    };

    let filename = export_json_to_dir(&results, Some(tmp.path())).unwrap();
    let content = fs::read_to_string(tmp.path().join(&filename)).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(parsed["files"][0]["result_contexts"][0]["result"].is_null());
}

#[test]
fn export_json_handles_multiple_files() {
    let tmp = TempDir::new().unwrap();

    let results = ProcessorResults {
        success: true,
        files: vec![
            HttpFileResults {
                filename: "api.http".to_string(),
                success_count: 2,
                failed_count: 0,
                skipped_count: 0,
                result_contexts: vec![
                    RequestContext {
                        name: "json_multi_req1".to_string(),
                        request: sample_request(
                            "json_multi_req1",
                            "GET",
                            "https://api.example.com/1",
                        ),
                        result: Some(sample_result(200, true, 100)),
                    },
                    RequestContext {
                        name: "json_multi_req2".to_string(),
                        request: sample_request(
                            "json_multi_req2",
                            "POST",
                            "https://api.example.com/2",
                        ),
                        result: Some(sample_result(201, true, 150)),
                    },
                ],
            },
            HttpFileResults {
                filename: "auth.http".to_string(),
                success_count: 1,
                failed_count: 0,
                skipped_count: 0,
                result_contexts: vec![RequestContext {
                    name: "json_multi_req3".to_string(),
                    request: sample_request(
                        "json_multi_req3",
                        "POST",
                        "https://api.example.com/login",
                    ),
                    result: Some(sample_result(200, true, 200)),
                }],
            },
        ],
    };

    let filename = export_json_to_dir(&results, Some(tmp.path())).unwrap();
    let content = fs::read_to_string(tmp.path().join(&filename)).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(parsed["files"].as_array().unwrap().len(), 2);
    assert_eq!(parsed["files"][0]["filename"], "api.http");
    assert_eq!(parsed["files"][1]["filename"], "auth.http");
    assert_eq!(
        parsed["files"][0]["result_contexts"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn export_json_produces_valid_json() {
    let tmp = TempDir::new().unwrap();

    let results = ProcessorResults {
        success: true,
        files: vec![],
    };

    let filename = export_json_to_dir(&results, Some(tmp.path())).unwrap();
    let content = fs::read_to_string(tmp.path().join(&filename)).unwrap();

    // Must parse as valid JSON
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
    assert!(parsed.is_ok());
}
