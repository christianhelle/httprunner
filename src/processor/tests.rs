use super::formatter::*;
use super::substitution::*;
use crate::types::{Header, HttpRequest, HttpResult, RequestContext};

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
