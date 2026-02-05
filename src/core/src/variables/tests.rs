use super::*;
use crate::types::{
    HttpRequest, HttpResult, RequestContext, RequestVariableSource, RequestVariableTarget,
};
use std::collections::HashMap;

#[test]
fn test_parse_request_variable_with_braces() {
    let reference = "{{login.response.body.$.token}}";
    let result = parse_request_variable(reference).unwrap();

    assert_eq!(result.request_name, "login");
    assert!(matches!(result.source, RequestVariableSource::Response));
    assert!(matches!(result.target, RequestVariableTarget::Body));
    assert_eq!(result.path, "$.token");
}

#[test]
fn test_parse_request_variable_without_braces() {
    let reference = "login.response.body.$.token";
    let result = parse_request_variable(reference).unwrap();

    assert_eq!(result.request_name, "login");
    assert_eq!(result.path, "$.token");
}

#[test]
fn test_parse_request_variable_headers() {
    let reference = "{{login.response.headers.Authorization}}";
    let result = parse_request_variable(reference).unwrap();

    assert!(matches!(result.target, RequestVariableTarget::Headers));
    assert_eq!(result.path, "Authorization");
}

#[test]
fn test_parse_request_variable_request_source() {
    let reference = "{{createUser.request.body.*}}";
    let result = parse_request_variable(reference).unwrap();

    assert!(matches!(result.source, RequestVariableSource::Request));
    assert_eq!(result.path, "*");
}

#[test]
fn test_parse_request_variable_invalid_format() {
    let reference = "{{invalid}}";
    let result = parse_request_variable(reference);

    assert!(result.is_err());
}

#[test]
fn test_parse_request_variable_invalid_source() {
    let reference = "{{login.invalid.body.$.token}}";
    let result = parse_request_variable(reference);

    assert!(result.is_err());
}

#[test]
fn test_parse_request_variable_invalid_target() {
    let reference = "{{login.response.invalid.$.token}}";
    let result = parse_request_variable(reference);

    assert!(result.is_err());
}

#[test]
fn test_extract_json_property_simple() {
    let json = r#"{"username":"john","age":30}"#;
    let result = extract_json_property(json, "username").unwrap();

    assert_eq!(result, Some("john".to_string()));
}

#[test]
fn test_extract_json_property_nested() {
    let json = r#"{"user":{"name":"john","email":"john@example.com"}}"#;
    let result = extract_json_property(json, "user.name").unwrap();

    assert_eq!(result, Some("john".to_string()));
}

#[test]
fn test_extract_json_property_array_index() {
    let json = r#"{"users":[{"name":"john"},{"name":"jane"}]}"#;
    let result = extract_json_property(json, "users[0].name").unwrap();

    assert_eq!(result, Some("john".to_string()));
}

#[test]
fn test_extract_json_property_not_found() {
    let json = r#"{"username":"john"}"#;
    let result = extract_json_property(json, "nonexistent").unwrap();

    assert_eq!(result, None);
}

#[test]
fn test_extract_json_property_number() {
    let json = r#"{"id":123,"active":true}"#;
    let result = extract_json_property(json, "id").unwrap();

    assert_eq!(result, Some("123".to_string()));
}

#[test]
fn test_extract_json_property_boolean() {
    let json = r#"{"active":true}"#;
    let result = extract_json_property(json, "active").unwrap();

    assert_eq!(result, Some("true".to_string()));
}

#[test]
fn test_extract_json_property_object() {
    let json = r#"{"user":{"name":"john","age":30}}"#;
    let result = extract_json_property(json, "user").unwrap();

    assert!(result.unwrap().contains("\"name\""));
}

#[test]
fn test_extract_json_property_array() {
    let json = r#"{"tags":["rust","http","testing"]}"#;
    let result = extract_json_property(json, "tags").unwrap();

    assert!(result.unwrap().contains("rust"));
}

#[test]
fn test_substitute_request_variables_simple() {
    let input = "Bearer {{login.response.body.$.token}}";

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

    let result = substitute_request_variables(input, &context).unwrap();

    assert_eq!(result, "Bearer secret123");
}

#[test]
fn test_substitute_request_variables_multiple() {
    let input = "https://{{config.response.body.$.host}}/users/{{user.response.body.$.id}}";

    let context = vec![
        RequestContext {
            name: "config".to_string(),
            request: HttpRequest {
                name: Some("config".to_string()),
                method: "GET".to_string(),
                url: "https://api.example.com/config".to_string(),
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
                request_name: Some("config".to_string()),
                status_code: 200,
                success: true,
                error_message: None,
                duration_ms: 50,
                response_headers: None,
                response_body: Some(r#"{"host":"api.example.com"}"#.to_string()),
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

    let result = substitute_request_variables(input, &context).unwrap();

    assert_eq!(result, "https://api.example.com/users/789");
}

#[test]
fn test_substitute_request_variables_not_found() {
    let input = "Bearer {{login.response.body.$.token}}";
    let context = vec![];

    let result = substitute_request_variables(input, &context).unwrap();

    // Should remain unchanged when variable not found
    assert_eq!(result, input);
}

#[test]
fn test_substitute_request_variables_from_headers() {
    let input = "{{login.response.headers.Set-Cookie}}";

    let mut headers = HashMap::new();
    headers.insert("Set-Cookie".to_string(), "session=abc123".to_string());

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
            response_headers: Some(headers),
            response_body: None,
            assertion_results: vec![],
        }),
    }];

    let result = substitute_request_variables(input, &context).unwrap();

    assert_eq!(result, "session=abc123");
}

#[test]
fn test_substitute_request_variables_from_request_body() {
    let input = "{{createUser.request.body.*}}";

    let context = vec![RequestContext {
        name: "createUser".to_string(),
        request: HttpRequest {
            name: Some("createUser".to_string()),
            method: "POST".to_string(),
            url: "https://api.example.com/users".to_string(),
            headers: vec![],
            body: Some(r#"{"name":"John","age":30}"#.to_string()),
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

    let result = substitute_request_variables(input, &context).unwrap();

    assert_eq!(result, r#"{"name":"John","age":30}"#);
}

#[test]
fn test_substitute_request_variables_nested_json() {
    let input = "{{getUser.response.body.$.profile.address.city}}";

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
            response_body: Some(
                r#"{"profile":{"address":{"city":"New York","zip":"10001"}}}"#.to_string(),
            ),
            assertion_results: vec![],
        }),
    }];

    let result = substitute_request_variables(input, &context).unwrap();

    assert_eq!(result, "New York");
}

#[test]
fn test_substitute_preserves_non_variable_text() {
    let input = "Prefix {{var.response.body.$.id}} Suffix";
    let context = vec![];

    let result = substitute_request_variables(input, &context).unwrap();

    assert_eq!(result, "Prefix {{var.response.body.$.id}} Suffix");
}

#[test]
fn test_substitute_handles_incomplete_braces() {
    let input = "{{incomplete";
    let context = vec![];

    let result = substitute_request_variables(input, &context).unwrap();

    assert_eq!(result, "{{incomplete");
}

#[test]
fn test_extract_json_property_invalid_array_index() {
    let json = r#"{"data": [1, 2, 3]}"#;
    let result = extract_json_property(json, "data[abc]");
    assert!(result.is_err());
}

#[test]
fn test_extract_json_property_array_not_array() {
    let json = r#"{"data": "not an array"}"#;
    let result = extract_json_property(json, "data[0]");
    assert!(result.is_err());
}

#[test]
fn test_extract_json_property_array_out_of_bounds() {
    let json = r#"{"data": [1, 2]}"#;
    let result = extract_json_property(json, "data[10]").unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_extract_json_property_nested_arrays() {
    let json = r#"{"data": [{"items": [1, 2, 3]}, {"items": [4, 5, 6]}]}"#;
    let result = extract_json_property(json, "data[1].items[2]").unwrap();
    assert_eq!(result, Some("6".to_string()));
}

#[test]
fn test_extract_json_property_string_value() {
    let json = r#"{"name": "test"}"#;
    let result = extract_json_property(json, "name").unwrap();
    assert_eq!(result, Some("test".to_string()));
}

#[test]
fn test_extract_json_property_null_value() {
    let json = r#"{"value": null}"#;
    let result = extract_json_property(json, "value").unwrap();
    assert_eq!(result, Some("null".to_string()));
}

#[test]
fn test_extract_json_property_nested_object() {
    let json = r#"{"user": {"profile": {"name": "John"}}}"#;
    let result = extract_json_property(json, "user.profile.name").unwrap();
    assert_eq!(result, Some("John".to_string()));
}

#[test]
fn test_extract_json_property_array_of_strings() {
    let json = r#"{"tags": ["rust", "testing", "code"]}"#;
    let result = extract_json_property(json, "tags[1]").unwrap();
    assert_eq!(result, Some("\"testing\"".to_string()));
}

#[test]
fn test_extract_json_property_complex_nested() {
    let json = r#"{"levels": {"l1": {"l2": {"l3": "deep"}}}}"#;
    let result = extract_json_property(json, "levels.l1.l2.l3").unwrap();
    assert_eq!(result, Some("deep".to_string()));
}

#[test]
fn test_extract_json_property_with_escape_sequences() {
    let json = r#"{"message": "Hello \"World\""}"#;
    let result = extract_json_property(json, "message").unwrap();
    assert_eq!(result, Some(r#"Hello \"World\""#.to_string()));
}

#[test]
fn test_extract_json_property_empty_string() {
    let json = r#"{"value": ""}"#;
    let result = extract_json_property(json, "value").unwrap();
    assert_eq!(result, Some("".to_string()));
}

#[test]
fn test_extract_json_property_array_with_nested_objects() {
    let json = r#"{"items": [{"id": 1, "name": "first"}, {"id": 2, "name": "second"}]}"#;
    let result = extract_json_property(json, "items[1].id").unwrap();
    assert_eq!(result, Some("2".to_string()));
}

#[test]
fn test_extract_json_property_deeply_nested_path() {
    let json = r#"{"a": {"b": {"c": {"d": {"e": "value"}}}}}"#;
    let result = extract_json_property(json, "a.b.c.d.e").unwrap();
    assert_eq!(result, Some("value".to_string()));
}

#[test]
fn test_extract_json_property_array_with_comma_in_string() {
    let json = r#"{"items": ["first,value", "second,value"]}"#;
    let result = extract_json_property(json, "items[0]").unwrap();
    assert_eq!(result, Some("\"first,value\"".to_string()));
}

#[test]
fn test_extract_json_property_boolean_false() {
    let json = r#"{"enabled": false}"#;
    let result = extract_json_property(json, "enabled").unwrap();
    assert_eq!(result, Some("false".to_string()));
}

#[test]
fn test_extract_json_property_number_float() {
    let json = r#"{"price": 19.99}"#;
    let result = extract_json_property(json, "price").unwrap();
    assert_eq!(result, Some("19.99".to_string()));
}

#[test]
fn test_extract_json_property_empty_array() {
    let json = r#"{"items": []}"#;
    let result = extract_json_property(json, "items[0]").unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_substitute_request_variables_invalid_format() {
    let context = vec![];
    // This has 3+ dots but is invalid format (missing parts)
    let result = substitute_request_variables("{{invalid.format.with.dots}}", &context).unwrap();
    assert_eq!(result, "{{invalid.format.with.dots}}");
}

#[test]
fn test_substitute_request_variables_extraction_error() {
    let context = vec![];
    // Valid format but request not found - should preserve the placeholder
    let result = substitute_request_variables("{{req.response.body.$.data}}", &context).unwrap();
    assert_eq!(result, "{{req.response.body.$.data}}");
}
