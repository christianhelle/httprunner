use crate::types::{Assertion, AssertionType, Condition, HttpRequest};
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn serialize_http_request(request: &HttpRequest) -> String {
    let mut output = String::new();

    output.push_str("###\n");

    if let Some(name) = &request.name {
        output.push_str(&format!("# @name {}\n", name));
    }

    if let Some(timeout) = request.timeout {
        output.push_str(&format!("# @timeout {}ms\n", timeout));
    }

    if let Some(connection_timeout) = request.connection_timeout {
        output.push_str(&format!("# @connection-timeout {}ms\n", connection_timeout));
    }

    if let Some(depends_on) = &request.depends_on {
        output.push_str(&format!("# @dependsOn {}\n", depends_on));
    }

    for condition in &request.conditions {
        let directive = if condition.negate {
            "# @if-not"
        } else {
            "# @if"
        };
        output.push_str(&format!("{} {}\n", directive, format_condition(condition)));
    }

    if let Some(pre_delay_ms) = request.pre_delay_ms {
        output.push_str(&format!("# @pre-delay {}\n", pre_delay_ms));
    }

    if let Some(post_delay_ms) = request.post_delay_ms {
        output.push_str(&format!("# @post-delay {}\n", post_delay_ms));
    }

    output.push_str(&format!("{} {}\n", request.method, request.url));

    for header in &request.headers {
        output.push_str(&format!("{}: {}\n", header.name, header.value));
    }

    for assertion in &request.assertions {
        output.push_str(&format!("{}\n", format_assertion(assertion)));
    }

    if let Some(body) = &request.body {
        output.push('\n');
        output.push_str(body);
        if !body.ends_with('\n') {
            output.push('\n');
        }
    }

    output
}

pub fn serialize_http_requests(requests: &[HttpRequest]) -> String {
    requests
        .iter()
        .map(serialize_http_request)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn write_http_file(path: &Path, requests: &[HttpRequest]) -> Result<()> {
    let content = serialize_http_requests(requests);
    fs::write(path, content)?;
    Ok(())
}

fn format_condition(condition: &Condition) -> String {
    use crate::types::ConditionType;

    match &condition.condition_type {
        ConditionType::BodyJsonPath(jsonpath) => {
            format!(
                "{}.response.body.{} {}",
                condition.request_name, jsonpath, condition.expected_value
            )
        }
        ConditionType::Status => {
            format!(
                "{}.response.status {}",
                condition.request_name, condition.expected_value
            )
        }
    }
}

fn format_assertion(assertion: &Assertion) -> String {
    match assertion.assertion_type {
        AssertionType::Status => format!("> EXPECTED_RESPONSE_STATUS {}", assertion.expected_value),
        AssertionType::Body => format!("> EXPECTED_RESPONSE_BODY {}", assertion.expected_value),
        AssertionType::Headers => {
            format!("> EXPECTED_RESPONSE_HEADERS {}", assertion.expected_value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ConditionType, Header};

    fn assert_request_matches(actual: &HttpRequest, expected: &HttpRequest) {
        assert_eq!(actual.name, expected.name);
        assert_eq!(actual.method, expected.method);
        assert_eq!(actual.url, expected.url);
        assert_eq!(
            actual
                .headers
                .iter()
                .map(|header| (header.name.as_str(), header.value.as_str()))
                .collect::<Vec<_>>(),
            expected
                .headers
                .iter()
                .map(|header| (header.name.as_str(), header.value.as_str()))
                .collect::<Vec<_>>()
        );
        assert_eq!(actual.body, expected.body);
        assert_eq!(actual.assertions.len(), expected.assertions.len());
        for (actual_assertion, expected_assertion) in
            actual.assertions.iter().zip(expected.assertions.iter())
        {
            assert_eq!(
                actual_assertion.assertion_type,
                expected_assertion.assertion_type
            );
            assert_eq!(
                actual_assertion.expected_value,
                expected_assertion.expected_value
            );
        }
        assert_eq!(actual.timeout, expected.timeout);
        assert_eq!(actual.connection_timeout, expected.connection_timeout);
        assert_eq!(actual.depends_on, expected.depends_on);
        assert_eq!(actual.conditions.len(), expected.conditions.len());
        for (actual_condition, expected_condition) in
            actual.conditions.iter().zip(expected.conditions.iter())
        {
            assert_eq!(
                actual_condition.request_name,
                expected_condition.request_name
            );
            assert_eq!(
                actual_condition.condition_type,
                expected_condition.condition_type
            );
            assert_eq!(
                actual_condition.expected_value,
                expected_condition.expected_value
            );
            assert_eq!(actual_condition.negate, expected_condition.negate);
        }
        assert_eq!(actual.pre_delay_ms, expected.pre_delay_ms);
        assert_eq!(actual.post_delay_ms, expected.post_delay_ms);
        assert_eq!(actual.variables.len(), expected.variables.len());
        for (actual_var, expected_var) in actual.variables.iter().zip(expected.variables.iter()) {
            assert_eq!(actual_var.name, expected_var.name);
            assert_eq!(actual_var.value, expected_var.value);
        }
    }

    #[test]
    fn test_serialize_simple_request() {
        let request = HttpRequest {
            name: None,
            method: "GET".to_string(),
            url: "https://httpbin.org/get".to_string(),
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
        };

        let serialized = serialize_http_request(&request);
        assert!(serialized.contains("###"));
        assert!(serialized.contains("GET https://httpbin.org/get"));
    }

    #[test]
    fn test_serialize_request_with_name() {
        let request = HttpRequest {
            name: Some("test-request".to_string()),
            method: "POST".to_string(),
            url: "https://httpbin.org/post".to_string(),
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
        };

        let serialized = serialize_http_request(&request);
        assert!(serialized.contains("# @name test-request"));
    }

    #[test]
    fn test_serialize_request_with_headers_and_body() {
        let request = HttpRequest {
            name: None,
            method: "POST".to_string(),
            url: "https://httpbin.org/post".to_string(),
            headers: vec![
                Header {
                    name: "Content-Type".to_string(),
                    value: "application/json".to_string(),
                },
                Header {
                    name: "Authorization".to_string(),
                    value: "Bearer token123".to_string(),
                },
            ],
            body: Some(r#"{"key": "value"}"#.to_string()),
            assertions: vec![],
            variables: vec![],
            timeout: None,
            connection_timeout: None,
            depends_on: None,
            conditions: vec![],
            pre_delay_ms: None,
            post_delay_ms: None,
        };

        let serialized = serialize_http_request(&request);
        assert!(serialized.contains("Content-Type: application/json"));
        assert!(serialized.contains("Authorization: Bearer token123"));
        assert!(serialized.contains(r#"{"key": "value"}"#));
    }

    #[test]
    fn test_serialize_request_preserves_parser_semantics() {
        let request = HttpRequest {
            name: Some("create-user".to_string()),
            method: "POST".to_string(),
            url: "https://httpbin.org/post".to_string(),
            headers: vec![Header {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            }],
            body: Some(r#"{"name":"Jane"}"#.to_string()),
            assertions: vec![
                Assertion {
                    assertion_type: AssertionType::Status,
                    expected_value: "201".to_string(),
                },
                Assertion {
                    assertion_type: AssertionType::Body,
                    expected_value: "created user".to_string(),
                },
                Assertion {
                    assertion_type: AssertionType::Headers,
                    expected_value: "Content-Type: application/json".to_string(),
                },
            ],
            variables: vec![],
            timeout: Some(5000),
            connection_timeout: Some(1000),
            depends_on: Some("login".to_string()),
            conditions: vec![
                Condition {
                    request_name: "login".to_string(),
                    condition_type: ConditionType::Status,
                    expected_value: "200".to_string(),
                    negate: false,
                },
                Condition {
                    request_name: "login".to_string(),
                    condition_type: ConditionType::BodyJsonPath("$.error".to_string()),
                    expected_value: "blocked".to_string(),
                    negate: true,
                },
            ],
            pre_delay_ms: Some(250),
            post_delay_ms: Some(750),
        };

        let serialized = serialize_http_request(&request);
        let reparsed = crate::parser::parse_http_content(&serialized, None).unwrap();

        assert_eq!(reparsed.len(), 1);
        let reparsed = &reparsed[0];
        assert_eq!(reparsed.name, request.name);
        assert_eq!(reparsed.method, request.method);
        assert_eq!(reparsed.url, request.url);
        assert_eq!(reparsed.timeout, request.timeout);
        assert_eq!(reparsed.connection_timeout, request.connection_timeout);
        assert_eq!(reparsed.depends_on, request.depends_on);
        assert_eq!(reparsed.pre_delay_ms, request.pre_delay_ms);
        assert_eq!(reparsed.post_delay_ms, request.post_delay_ms);
        assert_eq!(reparsed.headers.len(), 1);
        assert_eq!(reparsed.body, request.body);
        assert_eq!(reparsed.assertions.len(), 3);
        assert!(matches!(
            reparsed.assertions[0].assertion_type,
            AssertionType::Status
        ));
        assert_eq!(reparsed.assertions[0].expected_value, "201");
        assert!(matches!(
            reparsed.assertions[1].assertion_type,
            AssertionType::Body
        ));
        assert_eq!(reparsed.assertions[1].expected_value, "created user");
        assert!(matches!(
            reparsed.assertions[2].assertion_type,
            AssertionType::Headers
        ));
        assert_eq!(
            reparsed.assertions[2].expected_value,
            "Content-Type: application/json"
        );
        assert_eq!(reparsed.conditions.len(), 2);
        assert!(matches!(
            reparsed.conditions[0].condition_type,
            ConditionType::Status
        ));
        assert!(!reparsed.conditions[0].negate);
        assert!(matches!(
            reparsed.conditions[1].condition_type,
            ConditionType::BodyJsonPath(_)
        ));
        assert!(reparsed.conditions[1].negate);
    }

    #[test]
    fn test_serialize_multiple_requests_preserves_directive_boundaries() {
        let requests = vec![
            HttpRequest {
                name: Some("login".to_string()),
                method: "POST".to_string(),
                url: "https://api.example.com/login".to_string(),
                headers: vec![Header {
                    name: "Content-Type".to_string(),
                    value: "application/json".to_string(),
                }],
                body: Some("{\"username\":\"admin\",\"password\":\"secret\"}\n".to_string()),
                assertions: vec![Assertion {
                    assertion_type: AssertionType::Status,
                    expected_value: "200".to_string(),
                }],
                variables: vec![],
                timeout: Some(30_000),
                connection_timeout: None,
                depends_on: None,
                conditions: vec![],
                pre_delay_ms: None,
                post_delay_ms: None,
            },
            HttpRequest {
                name: Some("admin-dashboard".to_string()),
                method: "GET".to_string(),
                url: "https://api.example.com/admin/dashboard".to_string(),
                headers: vec![Header {
                    name: "Authorization".to_string(),
                    value: "Bearer {{login.response.body.$.token}}".to_string(),
                }],
                body: None,
                assertions: vec![Assertion {
                    assertion_type: AssertionType::Headers,
                    expected_value: "Content-Type: application/json".to_string(),
                }],
                variables: vec![],
                timeout: None,
                connection_timeout: Some(5_000),
                depends_on: Some("login".to_string()),
                conditions: vec![
                    Condition {
                        request_name: "login".to_string(),
                        condition_type: ConditionType::Status,
                        expected_value: "200".to_string(),
                        negate: false,
                    },
                    Condition {
                        request_name: "login".to_string(),
                        condition_type: ConditionType::BodyJsonPath("$.role".to_string()),
                        expected_value: "guest".to_string(),
                        negate: true,
                    },
                ],
                pre_delay_ms: Some(250),
                post_delay_ms: Some(500),
            },
        ];

        let serialized = serialize_http_requests(&requests);
        let reparsed = crate::parser::parse_http_content(&serialized, None).unwrap();

        assert_eq!(reparsed.len(), requests.len());
        for (actual, expected) in reparsed.iter().zip(requests.iter()) {
            assert_request_matches(actual, expected);
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::parser::parse_http_file;
    use std::fs;

    #[test]
    fn test_roundtrip_serialization() {
        let test_content = r#"### Test Request
# @name my-test
GET https://httpbin.org/get
Content-Type: application/json

### Another Request
POST https://httpbin.org/post
Content-Type: application/json
Authorization: Bearer token123

{
  "key": "value"
}
"#;

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_roundtrip_integration.http");
        fs::write(&test_file, test_content).unwrap();

        let requests = parse_http_file(test_file.to_str().unwrap(), None).unwrap();
        assert_eq!(requests.len(), 2);

        assert_eq!(requests[0].name, Some("my-test".to_string()));
        assert_eq!(requests[0].method, "GET");
        assert_eq!(requests[0].url, "https://httpbin.org/get");
        assert_eq!(requests[0].headers.len(), 1);
        assert_eq!(requests[0].headers[0].name, "Content-Type");

        assert_eq!(requests[1].method, "POST");
        assert_eq!(requests[1].url, "https://httpbin.org/post");
        assert_eq!(requests[1].headers.len(), 2);
        assert!(requests[1].body.is_some());

        let serialized = serialize_http_requests(&requests);
        assert!(serialized.contains("# @name my-test"));
        assert!(serialized.contains("GET https://httpbin.org/get"));
        assert!(serialized.contains("POST https://httpbin.org/post"));
        assert!(serialized.contains("Content-Type: application/json"));

        let output_file = temp_dir.join("test_roundtrip_output.http");
        write_http_file(&output_file, &requests).unwrap();

        let reparsed = parse_http_file(output_file.to_str().unwrap(), None).unwrap();
        assert_eq!(reparsed.len(), 2);
        assert_eq!(reparsed[0].method, "GET");
        assert_eq!(reparsed[1].method, "POST");

        let _ = fs::remove_file(&test_file);
        let _ = fs::remove_file(&output_file);
    }
}

#[cfg(test)]
mod timeout_tests {
    use super::*;
    use crate::parser::parse_http_file;
    use std::fs;

    #[test]
    fn test_timeout_serialization_roundtrip() {
        let test_content = r#"###
# @name test-timeout
# @timeout 5000ms
# @connection-timeout 3000ms
GET https://httpbin.org/get
"#;

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_timeout.http");
        fs::write(&test_file, test_content).unwrap();

        let requests = parse_http_file(test_file.to_str().unwrap(), None).unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].timeout, Some(5000));
        assert_eq!(requests[0].connection_timeout, Some(3000));

        let serialized = serialize_http_requests(&requests);
        println!("Serialized:\n{}", serialized);

        let output_file = temp_dir.join("test_timeout_output.http");
        write_http_file(&output_file, &requests).unwrap();

        let reparsed = parse_http_file(output_file.to_str().unwrap(), None).unwrap();
        assert_eq!(reparsed.len(), 1);
        assert_eq!(reparsed[0].timeout, Some(5000));
        assert_eq!(reparsed[0].connection_timeout, Some(3000));

        let _ = fs::remove_file(&test_file);
        let _ = fs::remove_file(&output_file);
    }
}
