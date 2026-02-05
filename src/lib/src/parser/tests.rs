use super::*;
use crate::types::{AssertionType, ConditionType};
use std::fs;
use std::io::Write;
use tempfile::TempDir;

fn create_test_file(dir: &TempDir, name: &str, content: &str) -> String {
    let file_path = dir.path().join(name);
    let mut file = fs::File::create(&file_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file_path.to_str().unwrap().to_string()
}

#[test]
fn test_parse_simple_get_request() {
    let temp_dir = TempDir::new().unwrap();
    let content = "GET https://api.example.com/users";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].url, "https://api.example.com/users");
}

#[test]
fn test_parse_request_with_name() {
    let temp_dir = TempDir::new().unwrap();
    let content = "# @name getUsers\nGET https://api.example.com/users";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].name, Some("getUsers".to_string()));
}

#[test]
fn test_parse_request_with_headers() {
    let temp_dir = TempDir::new().unwrap();
    let content = "POST https://api.example.com/users\nContent-Type: application/json\nAuthorization: Bearer token123";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].headers.len(), 2);
    assert_eq!(requests[0].headers[0].name, "Content-Type");
    assert_eq!(requests[0].headers[0].value, "application/json");
    assert_eq!(requests[0].headers[1].name, "Authorization");
    assert_eq!(requests[0].headers[1].value, "Bearer token123");
}

#[test]
fn test_parse_request_with_body() {
    let temp_dir = TempDir::new().unwrap();
    // Simple body without headers - body is anything that doesn't match header format
    let content = "POST https://api.example.com/users\n\nbody content here";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert!(requests[0].body.is_some());
    assert_eq!(requests[0].body.as_ref().unwrap(), "body content here");
}

#[test]
fn test_parse_request_with_json_body() {
    let temp_dir = TempDir::new().unwrap();
    // Once we have a line without colon (JSON opener), body mode starts
    let content = "POST https://api.example.com/users\nContent-Type: application/json\n\n{\n\"name\":\"John\"\n}";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].headers.len(), 1);
    assert!(requests[0].body.is_some());
    let body = requests[0].body.as_ref().unwrap();
    assert!(body.contains("name") && body.contains("John"));
}

#[test]
fn test_parse_multiple_requests() {
    let temp_dir = TempDir::new().unwrap();
    let content = "GET https://api.example.com/users\n\n###\n\nPOST https://api.example.com/users";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[1].method, "POST");
}

#[test]
fn test_parse_request_with_timeout() {
    let temp_dir = TempDir::new().unwrap();
    let content = "# @timeout 5000ms\nGET https://api.example.com/users";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].timeout, Some(5000));
}

#[test]
fn test_parse_timeout_with_seconds() {
    let temp_dir = TempDir::new().unwrap();
    let content = "# @timeout 5s\nGET https://api.example.com/users";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests[0].timeout, Some(5000));
}

#[test]
fn test_parse_timeout_with_minutes() {
    let temp_dir = TempDir::new().unwrap();
    let content = "# @timeout 2m\nGET https://api.example.com/users";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests[0].timeout, Some(120000));
}

#[test]
fn test_parse_connection_timeout() {
    let temp_dir = TempDir::new().unwrap();
    let content = "# @connection-timeout 3000ms\nGET https://api.example.com/users";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests[0].connection_timeout, Some(3000));
}

#[test]
fn test_parse_depends_on() {
    let temp_dir = TempDir::new().unwrap();
    let content = "# @dependsOn login\nGET https://api.example.com/profile";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests[0].depends_on, Some("login".to_string()));
}

#[test]
fn test_parse_if_condition_status() {
    let temp_dir = TempDir::new().unwrap();
    let content = "# @if login.response.status 200\nGET https://api.example.com/profile";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests[0].conditions.len(), 1);
    assert_eq!(requests[0].conditions[0].request_name, "login");
    assert!(matches!(
        requests[0].conditions[0].condition_type,
        ConditionType::Status
    ));
    assert_eq!(requests[0].conditions[0].expected_value, "200");
    assert!(!requests[0].conditions[0].negate);
}

#[test]
fn test_parse_if_not_condition() {
    let temp_dir = TempDir::new().unwrap();
    let content = "# @if-not login.response.status 404\nGET https://api.example.com/profile";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests[0].conditions.len(), 1);
    assert!(requests[0].conditions[0].negate);
}

#[test]
fn test_parse_if_condition_body_jsonpath() {
    let temp_dir = TempDir::new().unwrap();
    let content = "# @if login.response.body.$.token valid\nGET https://api.example.com/profile";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests[0].conditions.len(), 1);
    assert!(matches!(
        requests[0].conditions[0].condition_type,
        ConditionType::BodyJsonPath(_)
    ));
}

#[test]
fn test_parse_assertions() {
    let temp_dir = TempDir::new().unwrap();
    let content = "GET https://api.example.com/users\nEXPECTED_RESPONSE_STATUS 200\nEXPECTED_RESPONSE_BODY John";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests[0].assertions.len(), 2);
    assert!(matches!(
        requests[0].assertions[0].assertion_type,
        AssertionType::Status
    ));
    assert_eq!(requests[0].assertions[0].expected_value, "200");
    assert!(matches!(
        requests[0].assertions[1].assertion_type,
        AssertionType::Body
    ));
    assert_eq!(requests[0].assertions[1].expected_value, "John");
}

#[test]
fn test_parse_assertion_with_prefix() {
    let temp_dir = TempDir::new().unwrap();
    let content = "GET https://api.example.com/users\n> EXPECTED_RESPONSE_STATUS 200";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests[0].assertions.len(), 1);
    assert_eq!(requests[0].assertions[0].expected_value, "200");
}

#[test]
fn test_parse_variable_definition() {
    let temp_dir = TempDir::new().unwrap();
    let content = "@host=api.example.com\nGET https://{{host}}/users";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests[0].url, "https://api.example.com/users");
}

#[test]
fn test_parse_ignores_intellij_script_blocks() {
    let temp_dir = TempDir::new().unwrap();
    let content = "GET https://api.example.com/users\n> {%\nclient.test(\"test\", function() {});\n%}\nPOST https://api.example.com/users";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 2);
}

#[test]
fn test_parse_ignores_comments() {
    let temp_dir = TempDir::new().unwrap();
    let content = "# This is a comment\n// Another comment\nGET https://api.example.com/users";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
}

#[test]
fn test_parse_all_http_methods() {
    let temp_dir = TempDir::new().unwrap();
    let methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];

    for method in &methods {
        let content = format!("{} https://api.example.com/resource", method);
        let file_path = create_test_file(&temp_dir, &format!("{}.http", method), &content);

        let requests = parse_http_file(&file_path, None).unwrap();
        assert_eq!(requests[0].method, *method);
    }
}

#[test]
fn test_parse_request_with_empty_body_lines() {
    let temp_dir = TempDir::new().unwrap();
    let content = "POST http://example.com\nContent-Type: application/json\n\n\n{}\n\n### Next request\nGET http://example.com";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();
    assert_eq!(requests.len(), 2);
    assert!(requests[0].body.is_some());
    assert!(requests[0].body.as_ref().unwrap().contains("{}"));
}

#[test]
fn test_parse_quoted_assertion_body() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"GET http://example.com
> EXPECTED_RESPONSE_BODY "quoted value""#;
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].assertions.len(), 1);
    assert_eq!(requests[0].assertions[0].expected_value, "quoted value");
}

#[test]
fn test_parse_quoted_assertion_headers() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"GET http://example.com
> EXPECTED_RESPONSE_HEADERS "Content-Type: application/json""#;
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].assertions.len(), 1);
    assert_eq!(
        requests[0].assertions[0].expected_value,
        "Content-Type: application/json"
    );
}

#[test]
fn test_parse_variable_update() {
    let temp_dir = TempDir::new().unwrap();
    let content = "@var1 = initial\n@var1 = updated\nGET http://example.com/{{var1}}";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].url.contains("updated"));
}

#[test]
fn test_parse_invalid_if_directive() {
    // This test ensures invalid @if directive doesn't crash the parser
    let temp_dir = TempDir::new().unwrap();
    let content = "# @if invalid_format\nGET http://example.com";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].conditions.len(), 0);
}

#[test]
fn test_parse_invalid_if_not_directive() {
    // This test ensures invalid @if-not directive doesn't crash the parser
    let temp_dir = TempDir::new().unwrap();
    let content = "# @if-not invalid_format\nGET http://example.com";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].conditions.len(), 0);
}

#[test]
fn test_parse_name_with_double_slash_comment() {
    let temp_dir = TempDir::new().unwrap();
    let content = "// @name test_request\nGET http://example.com";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].name, Some("test_request".to_string()));
}

#[test]
fn test_parse_timeout_with_double_slash_comment() {
    let temp_dir = TempDir::new().unwrap();
    let content = "// @timeout 5000ms\nGET http://example.com";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].timeout, Some(5000));
}

#[test]
fn test_parse_connection_timeout_with_double_slash_comment() {
    let temp_dir = TempDir::new().unwrap();
    let content = "// @connection-timeout 3000ms\nGET http://example.com";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].connection_timeout, Some(3000));
}

#[test]
fn test_parse_depends_on_with_double_slash_comment() {
    let temp_dir = TempDir::new().unwrap();
    let content = "// @dependsOn login\nGET http://example.com";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].depends_on, Some("login".to_string()));
}

#[test]
fn test_parse_if_condition_with_double_slash_comment() {
    let temp_dir = TempDir::new().unwrap();
    let content = "// @if login.response.status == 200\nGET http://example.com";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].conditions.len(), 1);
}

#[test]
fn test_parse_if_not_condition_with_double_slash_comment() {
    let temp_dir = TempDir::new().unwrap();
    let content = "// @if-not login.response.status == 404\nGET http://example.com";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].conditions.len(), 1);
    assert!(requests[0].conditions[0].negate);
}

#[test]
fn test_parse_intellij_script_block_ignored() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"GET https://api.example.com/users
> {%
    client.global.set("token", response.body.token);
%}
"#;
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, "GET");
    // Script block should be ignored, not parsed as body
    assert!(requests[0].body.is_none());
}

#[test]
fn test_parse_intellij_script_block_single_line() {
    let temp_dir = TempDir::new().unwrap();
    let content = "GET https://api.example.com/users\n> {% client.test(response); %}";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    // Script block ending on same line should also be ignored
}

#[test]
fn test_parse_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let content = "";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 0);
}

#[test]
fn test_parse_comments_only_file() {
    let temp_dir = TempDir::new().unwrap();
    let content = "# This is a comment\n// This is also a comment\n# Another comment";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 0);
}

#[test]
fn test_parse_variable_override() {
    let temp_dir = TempDir::new().unwrap();
    let content = "@baseUrl = https://api.example.com\n@baseUrl = https://api.test.com\nGET {{baseUrl}}/users";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    // The second variable assignment should override the first
    assert_eq!(requests[0].url, "https://api.test.com/users");
}

#[test]
fn test_parse_variable_with_variable_reference() {
    let temp_dir = TempDir::new().unwrap();
    let content = "@host = api.example.com\n@baseUrl = https://{{host}}\nGET {{baseUrl}}/users";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].url, "https://api.example.com/users");
}

#[test]
fn test_parse_request_with_all_directives() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"# @name fullRequest
# @timeout 30s
# @connection-timeout 10s
# @dependsOn previousRequest
# @if previousRequest.response.status == 200
GET https://api.example.com/users"#;
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].name, Some("fullRequest".to_string()));
    assert_eq!(requests[0].timeout, Some(30_000));
    assert_eq!(requests[0].connection_timeout, Some(10_000));
    assert_eq!(requests[0].depends_on, Some("previousRequest".to_string()));
    assert_eq!(requests[0].conditions.len(), 1);
}

#[test]
fn test_parse_multiple_conditions() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"# @if auth.response.status == 200
# @if-not auth.response.body.$.error == "blocked"
GET https://api.example.com/protected"#;
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].conditions.len(), 2);
    assert!(!requests[0].conditions[0].negate);
    assert!(requests[0].conditions[1].negate);
}

#[test]
fn test_parse_assertion_without_quotes() {
    let temp_dir = TempDir::new().unwrap();
    let content = "GET https://api.example.com/status\n> EXPECTED_RESPONSE_STATUS 200";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].assertions.len(), 1);
    assert_eq!(
        requests[0].assertions[0].assertion_type,
        AssertionType::Status
    );
    assert_eq!(requests[0].assertions[0].expected_value, "200");
}

#[test]
fn test_parse_body_assertion_with_quotes() {
    let temp_dir = TempDir::new().unwrap();
    let content = "GET https://api.example.com/health\n> EXPECTED_RESPONSE_BODY \"healthy\"";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].assertions.len(), 1);
    assert_eq!(
        requests[0].assertions[0].assertion_type,
        AssertionType::Body
    );
    assert_eq!(requests[0].assertions[0].expected_value, "healthy");
}

#[test]
fn test_parse_http_content_directly() {
    let content = "GET https://api.example.com/users\nAuthorization: Bearer token";
    let requests = parse_http_content(content, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].headers.len(), 1);
}

#[test]
fn test_parse_preserves_body_whitespace() {
    let temp_dir = TempDir::new().unwrap();
    let content = "POST https://api.example.com/data\n\nline1\nline2\nline3";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    let body = requests[0].body.as_ref().unwrap();
    assert!(body.contains("line1"));
    assert!(body.contains("line2"));
    assert!(body.contains("line3"));
}

#[test]
fn test_parse_pre_delay() {
    let temp_dir = TempDir::new().unwrap();
    let content = "# @pre-delay 500\nGET https://api.example.com/users";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].pre_delay_ms, Some(500));
    assert_eq!(requests[0].post_delay_ms, None);
}

#[test]
fn test_parse_post_delay() {
    let temp_dir = TempDir::new().unwrap();
    let content = "# @post-delay 1000\nGET https://api.example.com/users";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].pre_delay_ms, None);
    assert_eq!(requests[0].post_delay_ms, Some(1000));
}

#[test]
fn test_parse_both_delays() {
    let temp_dir = TempDir::new().unwrap();
    let content = "# @pre-delay 250\n# @post-delay 750\nGET https://api.example.com/users";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].pre_delay_ms, Some(250));
    assert_eq!(requests[0].post_delay_ms, Some(750));
}

#[test]
fn test_parse_delay_with_double_slash_comment() {
    let temp_dir = TempDir::new().unwrap();
    let content = "// @pre-delay 500\n// @post-delay 1000\nGET https://api.example.com/users";
    let file_path = create_test_file(&temp_dir, "test.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].pre_delay_ms, Some(500));
    assert_eq!(requests[0].post_delay_ms, Some(1000));
}

#[test]
fn test_parse_delay_with_all_directives() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"# @name delayedRequest
# @timeout 30s
# @pre-delay 100
# @post-delay 200
# @dependsOn previousRequest
GET https://api.example.com/users"#;
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].name, Some("delayedRequest".to_string()));
    assert_eq!(requests[0].timeout, Some(30_000));
    assert_eq!(requests[0].pre_delay_ms, Some(100));
    assert_eq!(requests[0].post_delay_ms, Some(200));
    assert_eq!(requests[0].depends_on, Some("previousRequest".to_string()));
}
