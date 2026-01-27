use super::*;
use crate::types::{Header, HttpFileResults, HttpRequest, HttpResult, ProcessorResults, RequestContext};
use std::collections::HashMap;
use std::fs;

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
fn export_results_creates_request_and_response_files() {
    let request = sample_request("test_req_1", "GET", "https://example.com");
    let result = sample_result(200, true, 100);

    let context = RequestContext {
        name: "test_req_1".to_string(),
        request,
        result: Some(result),
    };

    let file_results = HttpFileResults {
        filename: "test.http".to_string(),
        success_count: 1,
        failed_count: 0,
        skipped_count: 0,
        result_contexts: vec![context],
    };

    let results = ProcessorResults {
        success: true,
        files: vec![file_results],
    };

    let export_result = export_results(&results, false).unwrap();

    assert_eq!(export_result.file_names.len(), 2);
    assert!(export_result.failed_file_names.is_empty());
    assert!(export_result.file_names[0].contains("test_req_1_request"));
    assert!(export_result.file_names[1].contains("test_req_1_response"));

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_results_creates_files_with_timestamp() {
    let context = RequestContext {
        name: "timestamped_1".to_string(),
        request: sample_request("timestamped_1", "POST", "https://api.example.com"),
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

    let export_result = export_results(&results, false).unwrap();

    for file_name in &export_result.file_names {
        assert!(file_name.ends_with(".log"));
        assert!(file_name.contains("_"));
    }

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_request_includes_method_and_url() {
    let request = sample_request("get_test_1", "GET", "https://api.example.com/users");
    let context = RequestContext {
        name: "get_test_1".to_string(),
        request,
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

    let export_result = export_results(&results, false).unwrap();
    let request_file = &export_result.file_names[0];
    let content = fs::read_to_string(request_file).unwrap();

    assert!(content.contains("GET https://api.example.com/users"));

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_request_includes_headers() {
    let mut request = sample_request("headers_test_1", "POST", "https://api.example.com");
    request.headers = vec![
        Header {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
        Header {
            name: "Authorization".to_string(),
            value: "Bearer token123".to_string(),
        },
    ];

    let context = RequestContext {
        name: "headers_test_1".to_string(),
        request,
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

    let export_result = export_results(&results, false).unwrap();
    let request_file = &export_result.file_names[0];
    let content = fs::read_to_string(request_file).unwrap();

    assert!(content.contains("Content-Type: application/json"));
    assert!(content.contains("Authorization: Bearer token123"));

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_request_includes_body() {
    let mut request = sample_request("body_test_1", "POST", "https://api.example.com");
    request.body = Some(r#"{"name":"John","age":30}"#.to_string());

    let context = RequestContext {
        name: "body_test_1".to_string(),
        request,
        result: Some(sample_result(201, true, 100)),
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

    let export_result = export_results(&results, false).unwrap();
    let request_file = &export_result.file_names[0];
    let content = fs::read_to_string(request_file).unwrap();

    assert!(content.contains(r#"{"name":"John","age":30}"#));

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_request_formats_json_when_pretty_json_enabled() {
    let mut request = sample_request("pretty_test_1", "POST", "https://api.example.com");
    request.body = Some(r#"{"name":"John","age":30}"#.to_string());

    let context = RequestContext {
        name: "pretty_test_1".to_string(),
        request,
        result: Some(sample_result(201, true, 100)),
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

    let export_result = export_results(&results, true).unwrap();
    let request_file = &export_result.file_names[0];
    let content = fs::read_to_string(request_file).unwrap();

    // Pretty-printed JSON should have newlines and indentation
    assert!(content.contains("{\n"));
    assert!(content.contains("  \"name\":"));
    assert!(content.contains("  \"age\":"));

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_response_includes_status_code() {
    let context = RequestContext {
        name: "status_test_1".to_string(),
        request: sample_request("status_test_1", "GET", "https://api.example.com"),
        result: Some(sample_result(404, false, 100)),
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

    let export_result = export_results(&results, false).unwrap();
    let response_file = &export_result.file_names[1];
    let content = fs::read_to_string(response_file).unwrap();

    assert!(content.contains("HTTP/1.1 404"));

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_response_includes_headers() {
    let mut result = sample_result(200, true, 100);
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("Server".to_string(), "nginx".to_string());
    result.response_headers = Some(headers);

    let context = RequestContext {
        name: "response_headers_1".to_string(),
        request: sample_request("response_headers_1", "GET", "https://api.example.com"),
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

    let export_result = export_results(&results, false).unwrap();
    let response_file = &export_result.file_names[1];
    let content = fs::read_to_string(response_file).unwrap();

    assert!(content.contains("Content-Type: application/json"));
    assert!(content.contains("Server: nginx"));

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_response_includes_body() {
    let mut result = sample_result(200, true, 100);
    result.response_body = Some(r#"{"status":"success","data":{"id":123}}"#.to_string());

    let context = RequestContext {
        name: "response_body_1".to_string(),
        request: sample_request("response_body_1", "GET", "https://api.example.com"),
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

    let export_result = export_results(&results, false).unwrap();
    let response_file = &export_result.file_names[1];
    let content = fs::read_to_string(response_file).unwrap();

    assert!(content.contains(r#"{"status":"success","data":{"id":123}}"#));

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_response_formats_json_when_pretty_json_enabled() {
    let mut result = sample_result(200, true, 100);
    result.response_body = Some(r#"{"status":"ok","count":42}"#.to_string());

    let context = RequestContext {
        name: "pretty_response_1".to_string(),
        request: sample_request("pretty_response_1", "GET", "https://api.example.com"),
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

    let export_result = export_results(&results, true).unwrap();
    let response_file = &export_result.file_names[1];
    let content = fs::read_to_string(response_file).unwrap();

    // Pretty-printed JSON should have newlines and indentation
    assert!(content.contains("{\n"));
    assert!(content.contains("  \"status\":"));

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_handles_missing_response_body() {
    let mut result = sample_result(204, true, 50);
    result.response_body = None;

    let context = RequestContext {
        name: "no_body_1".to_string(),
        request: sample_request("no_body_1", "DELETE", "https://api.example.com/item/1"),
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

    let export_result = export_results(&results, false).unwrap();

    assert_eq!(export_result.file_names.len(), 2);
    assert!(export_result.failed_file_names.is_empty());

    let response_file = &export_result.file_names[1];
    let content = fs::read_to_string(response_file).unwrap();

    assert!(content.contains("HTTP/1.1 204"));
    // Should only have status line, headers, and blank line
    assert!(!content.contains("null"));

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_handles_empty_response_body() {
    let mut result = sample_result(200, true, 50);
    result.response_body = Some("".to_string());

    let context = RequestContext {
        name: "empty_body_1".to_string(),
        request: sample_request("empty_body_1", "GET", "https://api.example.com"),
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

    let export_result = export_results(&results, false).unwrap();
    let response_file = &export_result.file_names[1];
    let content = fs::read_to_string(response_file).unwrap();

    assert!(content.contains("HTTP/1.1 200"));

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_handles_special_characters_in_request_name() {
    let context = RequestContext {
        name: "test-request_special_1".to_string(),
        request: sample_request("test-request_special_1", "GET", "https://api.example.com"),
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

    let export_result = export_results(&results, false).unwrap();

    assert_eq!(export_result.file_names.len(), 2);
    assert!(export_result.file_names[0].contains("test-request_special_1_request"));
    assert!(export_result.file_names[1].contains("test-request_special_1_response"));

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_handles_multiple_requests() {
    let contexts = vec![
        RequestContext {
            name: "multi_req1".to_string(),
            request: sample_request("multi_req1", "GET", "https://api.example.com/1"),
            result: Some(sample_result(200, true, 100)),
        },
        RequestContext {
            name: "multi_req2".to_string(),
            request: sample_request("multi_req2", "POST", "https://api.example.com/2"),
            result: Some(sample_result(201, true, 150)),
        },
        RequestContext {
            name: "multi_req3".to_string(),
            request: sample_request("multi_req3", "DELETE", "https://api.example.com/3"),
            result: Some(sample_result(204, true, 80)),
        },
    ];

    let results = ProcessorResults {
        success: true,
        files: vec![HttpFileResults {
            filename: "test.http".to_string(),
            success_count: 3,
            failed_count: 0,
            skipped_count: 0,
            result_contexts: contexts,
        }],
    };

    let export_result = export_results(&results, false).unwrap();

    // 3 requests × 2 files each (request + response) = 6 files
    assert_eq!(export_result.file_names.len(), 6);
    assert!(export_result.failed_file_names.is_empty());

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_handles_multiple_http_files() {
    let file1_context = RequestContext {
        name: "file1_req_test".to_string(),
        request: sample_request("file1_req_test", "GET", "https://api.example.com"),
        result: Some(sample_result(200, true, 100)),
    };

    let file2_context = RequestContext {
        name: "file2_req_test".to_string(),
        request: sample_request("file2_req_test", "POST", "https://api.example.com"),
        result: Some(sample_result(201, true, 150)),
    };

    let results = ProcessorResults {
        success: true,
        files: vec![
            HttpFileResults {
                filename: "file1.http".to_string(),
                success_count: 1,
                failed_count: 0,
                skipped_count: 0,
                result_contexts: vec![file1_context],
            },
            HttpFileResults {
                filename: "file2.http".to_string(),
                success_count: 1,
                failed_count: 0,
                skipped_count: 0,
                result_contexts: vec![file2_context],
            },
        ],
    };

    let export_result = export_results(&results, false).unwrap();

    // 2 files × 2 exports each (request + response) = 4 files
    assert_eq!(export_result.file_names.len(), 4);
    assert!(export_result.failed_file_names.is_empty());

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_handles_skipped_requests() {
    let context = RequestContext {
        name: "skipped_1".to_string(),
        request: sample_request("skipped_1", "GET", "https://api.example.com"),
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

    let export_result = export_results(&results, false).unwrap();

    // Should still create request file, but response file might be empty or minimal
    assert_eq!(export_result.file_names.len(), 2);

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_handles_non_json_response_body() {
    let mut result = sample_result(200, true, 100);
    result.response_body = Some("Plain text response".to_string());

    let context = RequestContext {
        name: "plain_text_resp".to_string(),
        request: sample_request("plain_text_resp", "GET", "https://example.com"),
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

    let export_result = export_results(&results, true).unwrap();
    let response_file = &export_result.file_names[1];
    let content = fs::read_to_string(response_file).unwrap();

    // Non-JSON should remain unchanged even with pretty_json=true
    assert!(content.contains("Plain text response"));

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_handles_non_json_request_body() {
    let mut request = sample_request("plain_request_1", "POST", "https://example.com");
    request.body = Some("username=test&password=secret".to_string());

    let context = RequestContext {
        name: "plain_request_1".to_string(),
        request,
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

    let export_result = export_results(&results, true).unwrap();
    let request_file = &export_result.file_names[0];
    let content = fs::read_to_string(request_file).unwrap();

    // Non-JSON should remain unchanged even with pretty_json=true
    assert!(content.contains("username=test&password=secret"));

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_results_creates_unique_filenames_for_same_timestamp() {
    let contexts = vec![
        RequestContext {
            name: "unique_req1".to_string(),
            request: sample_request("unique_req1", "GET", "https://api.example.com"),
            result: Some(sample_result(200, true, 100)),
        },
        RequestContext {
            name: "unique_req2".to_string(),
            request: sample_request("unique_req2", "GET", "https://api.example.com"),
            result: Some(sample_result(200, true, 100)),
        },
    ];

    let results = ProcessorResults {
        success: true,
        files: vec![HttpFileResults {
            filename: "test.http".to_string(),
            success_count: 2,
            failed_count: 0,
            skipped_count: 0,
            result_contexts: contexts,
        }],
    };

    let export_result = export_results(&results, false).unwrap();

    // All files should be created with unique names
    assert_eq!(export_result.file_names.len(), 4);
    let unique_names: std::collections::HashSet<_> =
        export_result.file_names.iter().collect();
    assert_eq!(unique_names.len(), 4);

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_handles_large_response_body() {
    let mut result = sample_result(200, true, 1000);
    let large_body = "x".repeat(10000);
    result.response_body = Some(large_body.clone());

    let context = RequestContext {
        name: "large_response_1".to_string(),
        request: sample_request("large_response_1", "GET", "https://api.example.com"),
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

    let export_result = export_results(&results, false).unwrap();
    assert!(export_result.failed_file_names.is_empty());

    let response_file = &export_result.file_names[1];
    let content = fs::read_to_string(response_file).unwrap();

    assert!(content.contains(&large_body));

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_formats_http_headers_with_crlf() {
    let mut request = sample_request("crlf_test_1", "GET", "https://api.example.com");
    request.headers = vec![Header {
        name: "Host".to_string(),
        value: "api.example.com".to_string(),
    }];

    let context = RequestContext {
        name: "crlf_test_1".to_string(),
        request,
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

    let export_result = export_results(&results, false).unwrap();
    let request_file = &export_result.file_names[0];
    let bytes = fs::read(request_file).unwrap();
    let content = String::from_utf8(bytes).unwrap();

    // Check for CRLF line endings
    assert!(content.contains("\r\n"));

    // Cleanup
    for file_name in &export_result.file_names {
        fs::remove_file(file_name).ok();
    }
}

#[test]
fn export_handles_different_http_methods() {
    let methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH"];

    for (i, method) in methods.iter().enumerate() {
        let name = format!("{}_method_test_{}", method.to_lowercase(), i);
        let context = RequestContext {
            name: name.clone(),
            request: sample_request(&name, method, "https://api.example.com"),
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

        let export_result = export_results(&results, false).unwrap();
        let request_file = &export_result.file_names[0];
        let content = fs::read_to_string(request_file).unwrap();

        assert!(content.contains(&format!("{} https://api.example.com", method)));

        // Cleanup
        for file_name in &export_result.file_names {
            fs::remove_file(file_name).ok();
        }
    }
}

#[test]
fn export_handles_various_status_codes() {
    let status_codes = vec![200, 201, 204, 400, 404, 500];

    for (i, status) in status_codes.iter().enumerate() {
        let name = format!("status_test_{}_{}", status, i);
        let context = RequestContext {
            name: name.clone(),
            request: sample_request(&name, "GET", "https://api.example.com"),
            result: Some(sample_result(*status, *status < 400, 100)),
        };

        let results = ProcessorResults {
            success: *status < 400,
            files: vec![HttpFileResults {
                filename: "test.http".to_string(),
                success_count: if *status < 400 { 1 } else { 0 },
                failed_count: if *status >= 400 { 1 } else { 0 },
                skipped_count: 0,
                result_contexts: vec![context],
            }],
        };

        let export_result = export_results(&results, false).unwrap();
        let response_file = &export_result.file_names[1];
        let content = fs::read_to_string(response_file).unwrap();

        assert!(content.contains(&format!("HTTP/1.1 {}", status)));

        // Cleanup
        for file_name in &export_result.file_names {
            fs::remove_file(file_name).ok();
        }
    }
}
