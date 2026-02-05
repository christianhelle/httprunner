use super::formatter::*;
use super::substitution::*;
use crate::types::{Header, HttpRequest, HttpResult, RequestContext};
use std::collections::HashMap;

#[test]
fn test_format_json_if_valid_with_valid_json() {
    let json = r#"{"name":"John","age":30}"#;
    let result = format_json_if_valid(json);

    assert!(result.contains("\"name\""));
    assert!(result.contains("\"John\""));
    assert!(result.contains('\n')); // Pretty-printed
}

#[test]
fn test_format_json_if_valid_with_invalid_json() {
    let invalid = "not json at all";
    let result = format_json_if_valid(invalid);

    assert_eq!(result, invalid);
}

#[test]
fn test_format_json_if_valid_with_malformed_json() {
    let malformed = "{name: John}";
    let result = format_json_if_valid(malformed);

    assert_eq!(result, malformed);
}

#[test]
fn test_format_request_name_with_name() {
    let name = Some("test_request".to_string());
    let result = format_request_name(&name);

    assert_eq!(result, "test_request: ");
}

#[test]
fn test_format_request_name_without_name() {
    let name = None;
    let result = format_request_name(&name);

    assert_eq!(result, "");
}

#[test]
fn test_substitute_request_variables_in_url() {
    let mut request = HttpRequest {
        name: Some("test".to_string()),
        method: "GET".to_string(),
        url: "https://api.example.com/users/{{login.response.body.$.id}}".to_string(),
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

    let context = vec![RequestContext {
        name: "login".to_string(),
        request: HttpRequest {
            name: Some("login".to_string()),
            method: "POST".to_string(),
            url: "https://api.example.com/login".to_string(),
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
        },
        result: Some(HttpResult {
            request_name: Some("login".to_string()),
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 100,
            response_headers: None,
            response_body: Some(r#"{"id":"123","token":"abc"}"#.to_string()),
            assertion_results: vec![],
        }),
    }];

    let result = substitute_request_variables_in_request(&mut request, &context);

    assert!(result.is_ok());
    assert_eq!(request.url, "https://api.example.com/users/123");
}

#[test]
fn test_substitute_request_variables_in_headers() {
    let mut request = HttpRequest {
        name: Some("test".to_string()),
        method: "GET".to_string(),
        url: "https://api.example.com/profile".to_string(),
        headers: vec![Header {
            name: "Authorization".to_string(),
            value: "Bearer {{login.response.body.$.token}}".to_string(),
        }],
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

    let context = vec![RequestContext {
        name: "login".to_string(),
        request: HttpRequest {
            name: Some("login".to_string()),
            method: "POST".to_string(),
            url: "https://api.example.com/login".to_string(),
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
        },
        result: Some(HttpResult {
            request_name: Some("login".to_string()),
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 100,
            response_headers: None,
            response_body: Some(r#"{"token":"secret123"}"#.to_string()),
            assertion_results: vec![],
        }),
    }];

    let result = substitute_request_variables_in_request(&mut request, &context);

    assert!(result.is_ok());
    assert_eq!(request.headers[0].value, "Bearer secret123");
}

#[test]
fn test_substitute_request_variables_in_body() {
    let mut request = HttpRequest {
        name: Some("test".to_string()),
        method: "POST".to_string(),
        url: "https://api.example.com/users".to_string(),
        headers: vec![],
        body: Some(r#"{"userId":"{{getUser.response.body.$.id}}"}"#.to_string()),
        assertions: vec![],
        variables: vec![],
        timeout: None,
        connection_timeout: None,
        depends_on: None,
        conditions: vec![],
        pre_delay_ms: None,
        post_delay_ms: None,
    };

    let context = vec![RequestContext {
        name: "getUser".to_string(),
        request: HttpRequest {
            name: Some("getUser".to_string()),
            method: "GET".to_string(),
            url: "https://api.example.com/users/1".to_string(),
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
        },
        result: Some(HttpResult {
            request_name: Some("getUser".to_string()),
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 100,
            response_headers: None,
            response_body: Some(r#"{"id":"456","name":"Jane"}"#.to_string()),
            assertion_results: vec![],
        }),
    }];

    let result = substitute_request_variables_in_request(&mut request, &context);

    assert!(result.is_ok());
    assert_eq!(request.body.unwrap(), r#"{"userId":"456"}"#);
}

#[test]
fn test_substitute_request_variables_with_no_context() {
    let mut request = HttpRequest {
        name: Some("test".to_string()),
        method: "GET".to_string(),
        url: "https://api.example.com/users/{{login.response.body.$.id}}".to_string(),
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

    let context = vec![];

    let result = substitute_request_variables_in_request(&mut request, &context);

    assert!(result.is_ok());
    // Should remain unchanged when context not found
    assert!(request.url.contains("{{login.response.body.$.id}}"));
}

#[test]
fn test_substitute_multiple_variables() {
    let mut request = HttpRequest {
        name: Some("test".to_string()),
        method: "GET".to_string(),
        url: "https://{{host.response.body.$.domain}}/users/{{user.response.body.$.id}}"
            .to_string(),
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

    let context = vec![
        RequestContext {
            name: "host".to_string(),
            request: HttpRequest {
                name: Some("host".to_string()),
                method: "GET".to_string(),
                url: "https://config.example.com".to_string(),
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
            },
            result: Some(HttpResult {
                request_name: Some("host".to_string()),
                status_code: 200,
                success: true,
                error_message: None,
                duration_ms: 50,
                response_headers: None,
                response_body: Some(r#"{"domain":"api.example.com"}"#.to_string()),
                assertion_results: vec![],
            }),
        },
        RequestContext {
            name: "user".to_string(),
            request: HttpRequest {
                name: Some("user".to_string()),
                method: "GET".to_string(),
                url: "https://api.example.com/me".to_string(),
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
            },
            result: Some(HttpResult {
                request_name: Some("user".to_string()),
                status_code: 200,
                success: true,
                error_message: None,
                duration_ms: 75,
                response_headers: None,
                response_body: Some(r#"{"id":"789"}"#.to_string()),
                assertion_results: vec![],
            }),
        },
    ];

    let result = substitute_request_variables_in_request(&mut request, &context);

    assert!(result.is_ok());
    assert_eq!(request.url, "https://api.example.com/users/789");
}

#[test]
fn test_substitute_functions_in_request_with_body() {
    let mut request = HttpRequest {
        name: Some("Test Request".to_string()),
        method: "GET".to_string(),
        url: "https://api.example.com/users".to_string(),
        headers: vec![],
        body: Some("User ID: guid()".to_string()),
        assertions: vec![],
        variables: vec![],
        timeout: None,
        connection_timeout: None,
        depends_on: None,
        conditions: vec![],
        pre_delay_ms: None,
        post_delay_ms: None,
    };

    let result = substitute_functions_in_request(&mut request);
    assert!(result.is_ok(), "Expected Ok result, got {:?}", result);

    if let Some(body) = &request.body {
        // Should not contain guid() token after substitution
        assert!(
            !body.contains("guid()"),
            "Body should not contain 'guid()' token after substitution: {}",
            body
        );
        // Should start with "User ID: " followed by a 32-character hex string (GUID)
        assert!(
            body.starts_with("User ID: "),
            "Body should start with 'User ID: ': {}",
            body
        );
        let guid_part = &body[9..]; // Skip "User ID: "
        assert_eq!(
            guid_part.len(),
            32,
            "Body should contain a full GUID (32 hex chars): {}",
            guid_part
        );
    }
}

#[test]
fn test_substitute_functions_in_request_with_headers() {
    let mut request = HttpRequest {
        name: Some("Test Request".to_string()),
        method: "GET".to_string(),
        url: "https://api.example.com".to_string(),
        headers: vec![Header {
            name: "X-Request-ID".to_string(),
            value: "guid()".to_string(),
        }],
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

    let result = substitute_functions_in_request(&mut request);
    assert!(result.is_ok(), "Expected Ok result, got {:?}", result);

    assert!(!request.headers.is_empty());
    let header = &request.headers[0];
    assert_eq!(header.name, "X-Request-ID");
    assert!(
        !header.value.contains("guid()"),
        "Header value should not contain 'guid()' token after substitution: {}",
        header.value
    );
    // Should be a 32-character hex string (GUID)
    assert_eq!(
        header.value.len(),
        32,
        "Header value should be a full GUID (32 hex chars): {}",
        header.value
    );
}

#[test]
fn test_substitute_functions_and_variables_together() {
    let mut response_headers = HashMap::new();
    response_headers.insert("Content-Type".to_string(), "application/json".to_string());

    let context = vec![RequestContext {
        name: "previous_request".to_string(),
        request: HttpRequest {
            name: Some("Setup Request".to_string()),
            method: "GET".to_string(),
            url: "https://api.example.com/setup".to_string(),
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
        },
        result: Some(HttpResult {
            request_name: Some("Setup Request".to_string()),
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 100,
            response_headers: Some(response_headers),
            response_body: Some(r#"{"user_id": "12345"}"#.to_string()),
            assertion_results: vec![],
        }),
    }];

    let mut request = HttpRequest {
        name: Some("Test Request".to_string()),
        method: "GET".to_string(),
        url: "https://api.example.com/users/{{previous_request.response.body.$.user_id}}"
            .to_string(),
        headers: vec![Header {
            name: "X-User-ID".to_string(),
            value: "{{previous_request.response.body.$.user_id}}".to_string(),
        }],
        body: Some("User ID: {{previous_request.response.body.$.user_id}}".to_string()),
        assertions: vec![],
        variables: vec![],
        timeout: None,
        connection_timeout: None,
        depends_on: None,
        conditions: vec![],
        pre_delay_ms: None,
        post_delay_ms: None,
    };

    let result = substitute_request_variables_in_request(&mut request, &context);
    assert!(result.is_ok(), "Expected Ok result, got {:?}", result);

    assert_eq!(
        request.url, "https://api.example.com/users/12345",
        "URL should have variable substituted"
    );
    assert_eq!(
        request.headers[0].value, "12345",
        "Header value should have variable substituted"
    );
    assert_eq!(
        request.body.as_ref().unwrap(),
        "User ID: 12345",
        "Body should have variable substituted"
    );
}

#[test]
fn test_format_json_if_valid_with_empty_string() {
    let result = format_json_if_valid("");
    assert_eq!(result, "");
}

#[test]
fn test_substitute_request_variables_with_missing_result() {
    let mut request = HttpRequest {
        name: Some("test".to_string()),
        method: "GET".to_string(),
        url: "https://api.example.com/users/{{login.response.body.$.id}}".to_string(),
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

    let context = vec![RequestContext {
        name: "login".to_string(),
        request: HttpRequest {
            name: Some("login".to_string()),
            method: "POST".to_string(),
            url: "https://api.example.com/login".to_string(),
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
        },
        result: None,
    }];

    let result = substitute_request_variables_in_request(&mut request, &context);

    assert!(result.is_ok());
    // URL should remain unchanged when result is None
    assert!(request.url.contains("{{login.response.body.$.id}}"));
}

#[test]
fn test_substitute_request_variables_in_assertions() {
    use crate::types::{Assertion, AssertionType};

    let mut request = HttpRequest {
        name: Some("test".to_string()),
        method: "GET".to_string(),
        url: "https://api.example.com/verify".to_string(),
        headers: vec![],
        body: None,
        assertions: vec![Assertion {
            assertion_type: AssertionType::Status,
            expected_value: "{{setup.response.body.$.expected_status}}".to_string(),
        }],
        variables: vec![],
        timeout: None,
        connection_timeout: None,
        depends_on: None,
        conditions: vec![],
        pre_delay_ms: None,
        post_delay_ms: None,
    };

    let context = vec![RequestContext {
        name: "setup".to_string(),
        request: HttpRequest {
            name: Some("setup".to_string()),
            method: "GET".to_string(),
            url: "https://api.example.com/setup".to_string(),
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
        },
        result: Some(HttpResult {
            request_name: Some("setup".to_string()),
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 100,
            response_headers: None,
            response_body: Some(r#"{"expected_status":"200"}"#.to_string()),
            assertion_results: vec![],
        }),
    }];

    let result = substitute_request_variables_in_request(&mut request, &context);

    assert!(result.is_ok());
    assert_eq!(request.assertions[0].expected_value, "200");
}

#[test]
fn test_substitute_functions_with_url() {
    let mut request = HttpRequest {
        name: Some("test".to_string()),
        method: "POST".to_string(),
        url: "https://api.example.com/track?id=guid()".to_string(),
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

    let result = substitute_functions_in_request(&mut request);
    assert!(result.is_ok());
    assert!(!request.url.contains("guid()"));
    assert!(request.url.starts_with("https://api.example.com/track?id="));
}

#[test]
fn test_substitute_functions_with_multiple_functions() {
    let mut request = HttpRequest {
        name: Some("test".to_string()),
        method: "POST".to_string(),
        url: "https://api.example.com".to_string(),
        headers: vec![
            Header {
                name: "X-Request-ID".to_string(),
                value: "guid()".to_string(),
            },
            Header {
                name: "X-Random".to_string(),
                value: "string()".to_string(),
            },
        ],
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

    let result = substitute_functions_in_request(&mut request);
    assert!(result.is_ok());
    assert!(!request.headers[0].value.contains("guid()"));
    assert!(!request.headers[1].value.contains("string()"));
    assert_eq!(request.headers[0].value.len(), 32); // GUID is 32 hex chars
    assert_eq!(request.headers[1].value.len(), 20); // string() generates 20 char string
}

#[test]
fn test_substitute_request_variables_with_header_reference() {
    let mut response_headers = HashMap::new();
    response_headers.insert("X-Auth-Token".to_string(), "secret-token-123".to_string());

    let mut request = HttpRequest {
        name: Some("test".to_string()),
        method: "GET".to_string(),
        url: "https://api.example.com/data".to_string(),
        headers: vec![Header {
            name: "Authorization".to_string(),
            value: "Bearer {{login.response.headers.X-Auth-Token}}".to_string(),
        }],
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

    let context = vec![RequestContext {
        name: "login".to_string(),
        request: HttpRequest {
            name: Some("login".to_string()),
            method: "POST".to_string(),
            url: "https://api.example.com/login".to_string(),
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
        },
        result: Some(HttpResult {
            request_name: Some("login".to_string()),
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 100,
            response_headers: Some(response_headers),
            response_body: None,
            assertion_results: vec![],
        }),
    }];

    let result = substitute_request_variables_in_request(&mut request, &context);
    assert!(result.is_ok());
    assert_eq!(request.headers[0].value, "Bearer secret-token-123");
}

#[test]
fn test_substitute_request_variables_with_complex_jsonpath() {
    let mut request = HttpRequest {
        name: Some("test".to_string()),
        method: "GET".to_string(),
        url: "https://api.example.com/items/{{data.response.body.$.users[0].id}}".to_string(),
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

    let context = vec![RequestContext {
        name: "data".to_string(),
        request: HttpRequest {
            name: Some("data".to_string()),
            method: "GET".to_string(),
            url: "https://api.example.com/users".to_string(),
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
        },
        result: Some(HttpResult {
            request_name: Some("data".to_string()),
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 100,
            response_headers: None,
            response_body: Some(
                r#"{"users":[{"id":"user1","name":"John"},{"id":"user2","name":"Jane"}]}"#
                    .to_string(),
            ),
            assertion_results: vec![],
        }),
    }];

    let result = substitute_request_variables_in_request(&mut request, &context);
    assert!(result.is_ok());
    assert_eq!(request.url, "https://api.example.com/items/user1");
}

#[test]
fn test_format_json_if_valid_with_nested_json() {
    let json = r#"{"user":{"name":"John","address":{"city":"NYC","zip":"10001"}}}"#;
    let result = format_json_if_valid(json);

    assert!(result.contains("\"user\""));
    assert!(result.contains("\"name\""));
    assert!(result.contains("\"address\""));
    assert!(result.contains("\"city\""));
    assert!(result.contains('\n'));
}

#[test]
fn test_format_json_if_valid_with_array() {
    let json = r#"[{"id":1,"name":"Item 1"},{"id":2,"name":"Item 2"}]"#;
    let result = format_json_if_valid(json);

    assert!(result.contains("\"id\""));
    assert!(result.contains("\"name\""));
    assert!(result.contains('\n'));
}

#[test]
fn test_substitute_request_variables_empty_body() {
    let mut request = HttpRequest {
        name: Some("test".to_string()),
        method: "GET".to_string(),
        url: "https://api.example.com/users/{{login.response.body.$.id}}".to_string(),
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

    let context = vec![RequestContext {
        name: "login".to_string(),
        request: HttpRequest {
            name: Some("login".to_string()),
            method: "POST".to_string(),
            url: "https://api.example.com/login".to_string(),
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
        },
        result: Some(HttpResult {
            request_name: Some("login".to_string()),
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 100,
            response_headers: None,
            response_body: None,
            assertion_results: vec![],
        }),
    }];

    let result = substitute_request_variables_in_request(&mut request, &context);
    assert!(result.is_ok());
}

#[test]
fn test_substitute_request_variables_with_special_chars_in_json() {
    let mut request = HttpRequest {
        name: Some("test".to_string()),
        method: "POST".to_string(),
        url: "https://api.example.com".to_string(),
        headers: vec![],
        body: Some(r#"{"message":"{{prev.response.body.$.text}}"}"#.to_string()),
        assertions: vec![],
        variables: vec![],
        timeout: None,
        connection_timeout: None,
        depends_on: None,
        conditions: vec![],
        pre_delay_ms: None,
        post_delay_ms: None,
    };

    let context = vec![RequestContext {
        name: "prev".to_string(),
        request: HttpRequest {
            name: Some("prev".to_string()),
            method: "GET".to_string(),
            url: "https://api.example.com/message".to_string(),
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
        },
        result: Some(HttpResult {
            request_name: Some("prev".to_string()),
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 100,
            response_headers: None,
            response_body: Some(r#"{"text":"Hello, World!"}"#.to_string()),
            assertion_results: vec![],
        }),
    }];

    let result = substitute_request_variables_in_request(&mut request, &context);
    assert!(result.is_ok());
    assert_eq!(request.body.unwrap(), r#"{"message":"Hello, World!"}"#);
}

#[test]
fn test_substitute_functions_in_assertions() {
    use crate::types::{Assertion, AssertionType};

    let mut request = HttpRequest {
        name: Some("test".to_string()),
        method: "GET".to_string(),
        url: "https://api.example.com".to_string(),
        headers: vec![],
        body: None,
        assertions: vec![Assertion {
            assertion_type: AssertionType::Body,
            expected_value: "guid()".to_string(),
        }],
        variables: vec![],
        timeout: None,
        connection_timeout: None,
        depends_on: None,
        conditions: vec![],
        pre_delay_ms: None,
        post_delay_ms: None,
    };

    let result = substitute_functions_in_request(&mut request);
    assert!(result.is_ok());
    assert!(!request.assertions[0].expected_value.contains("guid()"));
    assert_eq!(request.assertions[0].expected_value.len(), 32);
}

#[test]
fn test_format_request_name_with_empty_name() {
    let name = Some("".to_string());
    let result = format_request_name(&name);
    assert_eq!(result, ": ");
}

#[test]
fn test_substitute_request_variables_preserve_unmatched_patterns() {
    let mut request = HttpRequest {
        name: Some("test".to_string()),
        method: "GET".to_string(),
        url: "https://api.example.com/{{unknown.var}}".to_string(),
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

    let context = vec![];
    let result = substitute_request_variables_in_request(&mut request, &context);

    assert!(result.is_ok());
    // Unknown variables should be preserved
    assert!(request.url.contains("{{unknown.var}}"));
}
