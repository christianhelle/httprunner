use super::*;
use crate::types::{AssertionType, ConditionType};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};
use tempfile::TempDir;

fn create_test_file(dir: &TempDir, name: &str, content: &str) -> String {
    let file_path = dir.path().join(name);
    let mut file = fs::File::create(&file_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file_path.to_str().unwrap().to_string()
}

fn assert_requests_match(
    actual: &[crate::types::HttpRequest],
    expected: &[crate::types::HttpRequest],
) {
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
}

type ParseBackend = fn(&str, Option<&str>) -> anyhow::Result<Vec<crate::types::HttpRequest>>;

#[derive(Debug)]
struct BenchmarkCase {
    name: &'static str,
    inputs: Vec<String>,
    iterations: usize,
}

#[derive(Debug)]
struct BenchmarkMeasurement {
    scenario: &'static str,
    backend: &'static str,
    iterations: usize,
    total_bytes: usize,
    total_requests: usize,
    elapsed: Duration,
}

impl BenchmarkMeasurement {
    fn seconds(&self) -> f64 {
        self.elapsed.as_secs_f64().max(f64::EPSILON)
    }

    fn mib_per_second(&self) -> f64 {
        (self.total_bytes as f64 / (1024.0 * 1024.0)) / self.seconds()
    }

    fn requests_per_second(&self) -> f64 {
        self.total_requests as f64 / self.seconds()
    }
}

fn run_benchmark_case(
    case: &BenchmarkCase,
    backend: &'static str,
    parse: ParseBackend,
) -> BenchmarkMeasurement {
    let total_bytes_per_iteration = case.inputs.iter().map(String::len).sum::<usize>();
    let mut total_requests = 0;
    let start = Instant::now();

    for _ in 0..case.iterations {
        for input in &case.inputs {
            total_requests += parse(input, None).unwrap().len();
        }
    }

    BenchmarkMeasurement {
        scenario: case.name,
        backend,
        iterations: case.iterations,
        total_bytes: total_bytes_per_iteration * case.iterations,
        total_requests,
        elapsed: start.elapsed(),
    }
}

fn load_example_inputs() -> Vec<String> {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("examples");
    let mut example_paths = fs::read_dir(&examples_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("http"))
        .collect::<Vec<_>>();
    example_paths.sort();

    example_paths
        .into_iter()
        .map(|path| fs::read_to_string(path).unwrap())
        .collect()
}

fn build_synthetic_request_fixture(request_count: usize) -> String {
    let mut content = String::new();

    for index in 0..request_count {
        if !content.is_empty() {
            content.push('\n');
        }

        content.push_str("###\n");
        content.push_str(&format!("# @name request-{index}\n"));

        if index > 0 {
            let previous = index - 1;
            content.push_str(&format!("# @dependsOn request-{previous}\n"));
            content.push_str(&format!("# @if request-{previous}.response.status 200\n"));
        }

        if index % 2 == 0 {
            content.push_str(&format!("GET https://api.example.com/items/{index}\n"));
            content.push_str(&format!("X-Request-Id: {index}\n"));
        } else {
            content.push_str(&format!("POST https://api.example.com/items/{index}\n"));
            content.push_str("Content-Type: application/json\n");
            content.push_str(&format!("X-Request-Id: {index}\n\n"));
            content.push_str(&format!("{{\"id\":{index},\"name\":\"item-{index}\"}}\n"));
        }
    }

    content
}

fn build_large_body_fixture(payload_size_bytes: usize) -> String {
    let payload = "a".repeat(payload_size_bytes);
    format!(
        "POST https://api.example.com/upload\nContent-Type: application/json\n\n{{\"payload\":\"{}\"}}\n",
        payload
    )
}

fn print_benchmark_result(measurement: &BenchmarkMeasurement) {
    println!(
        "{} [{}] iterations={} elapsed={:?} throughput={:.2} MiB/s requests={:.0}/s",
        measurement.scenario,
        measurement.backend,
        measurement.iterations,
        measurement.elapsed,
        measurement.mib_per_second(),
        measurement.requests_per_second()
    );
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
    let methods = [
        "GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS", "TRACE", "CONNECT",
    ];

    for method in &methods {
        let content = format!("{} https://api.example.com/resource", method);
        let file_path = create_test_file(&temp_dir, &format!("{}.http", method), &content);

        let requests = parse_http_file(&file_path, None).unwrap();
        assert_eq!(requests[0].method, *method);
    }
}

#[test]
fn test_parse_request_line_with_http_version_and_trailing_tokens() {
    let temp_dir = TempDir::new().unwrap();
    let content =
        "GET https://api.example.com/users HTTP/1.1 # trailing request-line tokens are ignored";
    let file_path = create_test_file(&temp_dir, "http-version.http", content);

    let requests = parse_http_file(&file_path, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].url, "https://api.example.com/users");
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
fn test_parse_blank_line_switches_from_headers_to_body_mode() {
    let content = r#"POST https://api.example.com/users
Content-Type: application/json

X-Trace-Id: this-stays-in-the-body
{"name":"Jane"}"#;

    let requests = parse_http_content(content, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].headers.len(), 1);
    assert_eq!(requests[0].headers[0].name, "Content-Type");
    assert_eq!(
        requests[0].body.as_deref(),
        Some("X-Trace-Id: this-stays-in-the-body\n{\"name\":\"Jane\"}")
    );
}

#[test]
fn test_parse_at_lines_inside_body_stay_body_text() {
    let content = r#"POST https://api.example.com/users
Content-Type: text/plain

@literal-body-line
@still_body = not_a_variable"#;

    let requests = parse_http_content(content, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0].body.as_deref(),
        Some("@literal-body-line\n@still_body = not_a_variable")
    );
}

#[test]
fn test_parse_directives_after_body_buffer_to_next_request() {
    let content = r#"# @name first
POST https://api.example.com/first
Content-Type: text/plain

first-body
# @name second
# @dependsOn first
# @timeout 5s
GET https://api.example.com/second"#;

    let requests = parse_http_content(content, None).unwrap();

    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].name, Some("first".to_string()));
    assert_eq!(requests[0].timeout, None);
    assert_eq!(requests[0].depends_on, None);
    assert_eq!(requests[0].body.as_deref(), Some("first-body"));
    assert_eq!(requests[1].name, Some("second".to_string()));
    assert_eq!(requests[1].depends_on, Some("first".to_string()));
    assert_eq!(requests[1].timeout, Some(5000));
    assert!(requests[1].body.is_none());
}

#[test]
fn test_parse_body_mode_precedence_for_comments_scripts_assertions_and_requests() {
    let content = r#"POST https://api.example.com/first

body line
# ignored comment
> {%
client.test("ignored", function() {});
%}
> EXPECTED_RESPONSE_STATUS 204
GET https://api.example.com/second"#;

    let requests = parse_http_content(content, None).unwrap();

    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].body.as_deref(), Some("body line"));
    assert_eq!(requests[0].assertions.len(), 1);
    assert_eq!(
        requests[0].assertions[0].assertion_type,
        AssertionType::Status
    );
    assert_eq!(requests[0].assertions[0].expected_value, "204");
    assert_eq!(requests[1].method, "GET");
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
    let temp_dir = TempDir::new().unwrap();
    let content = "# @if invalid_format\nGET http://example.com";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let error = parse_http_file(&file_path, None).unwrap_err();
    let message = format!("{error:#}");
    assert!(message.contains("Invalid @if directive format"));
}

#[test]
fn test_parse_invalid_if_not_directive() {
    let temp_dir = TempDir::new().unwrap();
    let content = "# @if-not invalid_format\nGET http://example.com";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let error = parse_http_file(&file_path, None).unwrap_err();
    let message = format!("{error:#}");
    assert!(message.contains("Invalid @if-not directive format"));
}

#[test]
fn test_parse_invalid_timeout_directive_fails() {
    let temp_dir = TempDir::new().unwrap();
    let content = "# @timeout nope\nGET http://example.com";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let error = parse_http_file(&file_path, None).unwrap_err();
    let message = format!("{error:#}");
    assert!(message.contains("Invalid timeout value"));
}

#[test]
fn test_parse_invalid_connection_timeout_directive_fails() {
    let temp_dir = TempDir::new().unwrap();
    let content = "# @connection-timeout nope\nGET http://example.com";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let error = parse_http_file(&file_path, None).unwrap_err();
    let message = format!("{error:#}");
    assert!(message.contains("Invalid connection-timeout value"));
}

#[test]
fn test_parse_invalid_pre_delay_directive_fails() {
    let temp_dir = TempDir::new().unwrap();
    let content = "# @pre-delay nope\nGET http://example.com";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let error = parse_http_file(&file_path, None).unwrap_err();
    let message = format!("{error:#}");
    assert!(message.contains("Invalid @pre-delay value"));
}

#[test]
fn test_parse_invalid_post_delay_directive_fails() {
    let temp_dir = TempDir::new().unwrap();
    let content = "# @post-delay nope\nGET http://example.com";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let error = parse_http_file(&file_path, None).unwrap_err();
    let message = format!("{error:#}");
    assert!(message.contains("Invalid @post-delay value"));
}

#[test]
fn test_parse_invalid_variable_declaration_fails() {
    let temp_dir = TempDir::new().unwrap();
    let content = "@token\nGET http://example.com";
    let file_path = create_test_file(&temp_dir, "test.http", content);
    let error = parse_http_file(&file_path, None).unwrap_err();
    let message = format!("{error:#}");
    assert!(message.contains("Invalid variable declaration"));
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
    assert_eq!(requests[0].conditions[0].expected_value, "200");
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
    assert_eq!(requests[0].conditions[0].expected_value, "404");
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
fn test_parse_intellij_script_block_closes_on_line_with_content_before_percent_brace() {
    let content = r#"GET https://api.example.com/users
> {%
client.test("resume parsing", function() {
  client.assert(true, "still ignored"); %}
POST https://api.example.com/users
Content-Type: application/json

{"name":"Jane"}"#;

    let requests = parse_http_content(content, None).unwrap();
    let legacy_requests = parse_http_content_with_legacy_backend(content, None).unwrap();

    assert_requests_match(&requests, &legacy_requests);
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[1].method, "POST");
    assert_eq!(requests[1].headers.len(), 1);
    assert_eq!(requests[1].headers[0].name, "Content-Type");
    assert_eq!(requests[1].headers[0].value, "application/json");
    assert_eq!(requests[1].body.as_deref(), Some(r#"{"name":"Jane"}"#));
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
    assert_eq!(requests[0].conditions[0].expected_value, "200");
    assert_eq!(requests[0].conditions[1].expected_value, "blocked");
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

#[test]
fn test_parse_readme_authentication_flow_example() {
    let content = r#"# @name authenticate
POST https://httpbin.org/post
Content-Type: application/json

{
  "username": "admin@example.com",
  "password": "secure123",
  "access_token": "jwt_token_here",
  "refresh_token": "refresh_jwt_here",
  "user_id": "admin_001",
  "role": "administrator"
}

###

# @name get_admin_data
GET https://httpbin.org/get
Authorization: Bearer {{authenticate.response.body.$.json.access_token}}
X-User-Role: {{authenticate.response.body.$.json.role}}
X-User-ID: {{authenticate.response.body.$.json.user_id}}

###

# @name create_audit_log
POST https://httpbin.org/post
Content-Type: application/json

{
  "action": "admin_data_access",
  "user_id": "{{authenticate.response.body.$.json.user_id}}",
  "original_request": {{authenticate.request.body.*}},
  "timestamp": "2025-07-01T21:16:46Z",
  "response_content_type": "{{get_admin_data.response.headers.Content-Type}}"
}"#;

    let requests = parse_http_content(content, None).unwrap();

    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].name, Some("authenticate".to_string()));
    assert_eq!(requests[0].method, "POST");
    assert!(
        requests[0]
            .body
            .as_ref()
            .unwrap()
            .contains("\"access_token\": \"jwt_token_here\"")
    );
    assert_eq!(requests[1].name, Some("get_admin_data".to_string()));
    assert_eq!(requests[1].headers.len(), 3);
    assert_eq!(
        requests[1].headers[0].value,
        "Bearer {{authenticate.response.body.$.json.access_token}}"
    );
    assert_eq!(requests[2].name, Some("create_audit_log".to_string()));
    assert!(
        requests[2]
            .body
            .as_ref()
            .unwrap()
            .contains("{{authenticate.request.body.*}}")
    );
    assert!(
        requests[2]
            .body
            .as_ref()
            .unwrap()
            .contains("{{get_admin_data.response.headers.Content-Type}}")
    );
}

#[test]
fn test_parse_reference_directive_examples() {
    let content = r#"# @name login
# @timeout 30
POST https://api.example.com/auth/login
Content-Type: application/json

{"username": "user", "password": "pass"}

###

# @pre-delay 2000
// @connection-timeout 5
GET https://api.example.com/status

###

# @name getUser
# @dependsOn login
# @if login.response.status 200
GET https://api.example.com/user/profile
Authorization: Bearer {{login.response.body.$.token}}

###

# @if-not getUser.response.status 200
POST https://api.example.com/user/create
Content-Type: application/json

{"username": "newuser"}"#;

    let requests = parse_http_content(content, None).unwrap();

    assert_eq!(requests.len(), 4);
    assert_eq!(requests[0].name, Some("login".to_string()));
    assert_eq!(requests[0].timeout, Some(30_000));
    assert_eq!(
        requests[0].body.as_deref(),
        Some("{\"username\": \"user\", \"password\": \"pass\"}\n\n")
    );
    assert_eq!(requests[1].pre_delay_ms, Some(2000));
    assert_eq!(requests[1].connection_timeout, Some(5000));
    assert_eq!(requests[2].name, Some("getUser".to_string()));
    assert_eq!(requests[2].depends_on, Some("login".to_string()));
    assert_eq!(requests[2].conditions.len(), 1);
    assert!(matches!(
        requests[2].conditions[0].condition_type,
        ConditionType::Status
    ));
    assert_eq!(requests[3].conditions.len(), 1);
    assert!(requests[3].conditions[0].negate);
    assert_eq!(
        requests[3].body.as_deref(),
        Some("{\"username\": \"newuser\"}")
    );
}

#[test]
fn test_parse_reference_multiple_assertions_example() {
    let content = r#"GET https://api.example.com/users/1

EXPECTED_RESPONSE_STATUS 200
EXPECTED_RESPONSE_BODY "John Doe"
EXPECTED_RESPONSE_BODY "john@example.com"
EXPECTED_RESPONSE_HEADERS "Content-Type: application/json"
EXPECTED_RESPONSE_HEADERS "Cache-Control: no-cache""#;

    let requests = parse_http_content(content, None).unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].assertions.len(), 5);
    assert_eq!(
        requests[0].assertions[0].assertion_type,
        AssertionType::Status
    );
    assert_eq!(requests[0].assertions[0].expected_value, "200");
    assert_eq!(
        requests[0].assertions[1].assertion_type,
        AssertionType::Body
    );
    assert_eq!(requests[0].assertions[1].expected_value, "John Doe");
    assert_eq!(
        requests[0].assertions[2].assertion_type,
        AssertionType::Body
    );
    assert_eq!(requests[0].assertions[2].expected_value, "john@example.com");
    assert_eq!(
        requests[0].assertions[3].assertion_type,
        AssertionType::Headers
    );
    assert_eq!(
        requests[0].assertions[3].expected_value,
        "Content-Type: application/json"
    );
    assert_eq!(
        requests[0].assertions[4].assertion_type,
        AssertionType::Headers
    );
    assert_eq!(
        requests[0].assertions[4].expected_value,
        "Cache-Control: no-cache"
    );
}

#[test]
#[ignore = "benchmark helper; run with cargo test -p httprunner-core benchmark_parser_backends --release -- --ignored --nocapture"]
fn benchmark_parser_backends() {
    let cases = vec![
        BenchmarkCase {
            name: "examples-directory",
            inputs: load_example_inputs(),
            iterations: 1000,
        },
        BenchmarkCase {
            name: "synthetic-1000-requests",
            inputs: vec![build_synthetic_request_fixture(1000)],
            iterations: 100,
        },
        BenchmarkCase {
            name: "single-request-10mb-body",
            inputs: vec![build_large_body_fixture(10 * 1024 * 1024)],
            iterations: 10,
        },
    ];

    for case in &cases {
        for input in &case.inputs {
            let legacy = parse_http_content_with_legacy_backend(input, None).unwrap();
            let pest = parse_http_content(input, None).unwrap();
            assert_requests_match(&pest, &legacy);
        }

        let legacy = run_benchmark_case(case, "legacy", parse_http_content_with_legacy_backend);
        let pest = run_benchmark_case(case, "pest", parse_http_content);

        print_benchmark_result(&legacy);
        print_benchmark_result(&pest);

        let regression = 100.0 * (1.0 - (pest.mib_per_second() / legacy.mib_per_second()));
        println!(
            "{} pest throughput regression: {regression:.1}% (positive means slower)",
            case.name
        );
    }
}
