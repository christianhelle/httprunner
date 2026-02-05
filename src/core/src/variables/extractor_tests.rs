use super::extractor::*;
use crate::types::{
    Header, HttpRequest, HttpResult, RequestContext, RequestVariable, RequestVariableSource,
    RequestVariableTarget,
};
use std::collections::HashMap;

fn create_test_context() -> Vec<RequestContext> {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token123".to_string());
    headers.insert("content-type".to_string(), "application/json".to_string());

    vec![RequestContext {
        name: "login".to_string(),
        request: HttpRequest {
            name: Some("login".to_string()),
            method: "POST".to_string(),
            url: "https://api.example.com/login".to_string(),
            headers: vec![crate::types::Header {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            }],
            body: Some(r#"{"username":"admin","password":"secret"}"#.to_string()),
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
            response_body: Some(r#"{"token":"secret123","userId":42}"#.to_string()),
            assertion_results: vec![],
        }),
    }]
}

#[test]
fn test_extract_request_variable_value_response_body_json() {
    let context = create_test_context();
    let request_var = RequestVariable {
        reference: "{{login.response.body.$.token}}".to_string(),
        request_name: "login".to_string(),
        source: RequestVariableSource::Response,
        target: RequestVariableTarget::Body,
        path: "$.token".to_string(),
    };

    let result = extract_request_variable_value(&request_var, &context).unwrap();
    assert_eq!(result, Some("secret123".to_string()));
}

#[test]
fn test_extract_request_variable_value_response_body_wildcard() {
    let context = create_test_context();
    let request_var = RequestVariable {
        reference: "{{login.response.body.*}}".to_string(),
        request_name: "login".to_string(),
        source: RequestVariableSource::Response,
        target: RequestVariableTarget::Body,
        path: "*".to_string(),
    };

    let result = extract_request_variable_value(&request_var, &context).unwrap();
    assert_eq!(
        result,
        Some(r#"{"token":"secret123","userId":42}"#.to_string())
    );
}

#[test]
fn test_extract_request_variable_value_response_headers() {
    let context = create_test_context();
    let request_var = RequestVariable {
        reference: "{{login.response.headers.Authorization}}".to_string(),
        request_name: "login".to_string(),
        source: RequestVariableSource::Response,
        target: RequestVariableTarget::Headers,
        path: "Authorization".to_string(),
    };

    let result = extract_request_variable_value(&request_var, &context).unwrap();
    assert_eq!(result, Some("Bearer token123".to_string()));
}

#[test]
fn test_extract_request_variable_value_response_headers_case_insensitive() {
    let context = create_test_context();
    let request_var = RequestVariable {
        reference: "{{login.response.headers.CONTENT-TYPE}}".to_string(),
        request_name: "login".to_string(),
        source: RequestVariableSource::Response,
        target: RequestVariableTarget::Headers,
        path: "CONTENT-TYPE".to_string(),
    };

    let result = extract_request_variable_value(&request_var, &context).unwrap();
    assert_eq!(result, Some("application/json".to_string()));
}

#[test]
fn test_extract_request_variable_value_request_body() {
    let context = create_test_context();
    let request_var = RequestVariable {
        reference: "{{login.request.body.*}}".to_string(),
        request_name: "login".to_string(),
        source: RequestVariableSource::Request,
        target: RequestVariableTarget::Body,
        path: "*".to_string(),
    };

    let result = extract_request_variable_value(&request_var, &context).unwrap();
    assert_eq!(
        result,
        Some(r#"{"username":"admin","password":"secret"}"#.to_string())
    );
}

#[test]
fn test_extract_request_variable_value_request_headers() {
    let context = create_test_context();
    let request_var = RequestVariable {
        reference: "{{login.request.headers.Content-Type}}".to_string(),
        request_name: "login".to_string(),
        source: RequestVariableSource::Request,
        target: RequestVariableTarget::Headers,
        path: "Content-Type".to_string(),
    };

    let result = extract_request_variable_value(&request_var, &context).unwrap();
    assert_eq!(result, Some("application/json".to_string()));
}

#[test]
fn test_extract_request_variable_value_request_not_found() {
    let context = create_test_context();
    let request_var = RequestVariable {
        reference: "{{unknown.response.body.$.token}}".to_string(),
        request_name: "unknown".to_string(),
        source: RequestVariableSource::Response,
        target: RequestVariableTarget::Body,
        path: "$.token".to_string(),
    };

    let result = extract_request_variable_value(&request_var, &context).unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_extract_request_variable_value_no_result() {
    let context = vec![RequestContext {
        name: "pending".to_string(),
        request: HttpRequest {
            name: Some("pending".to_string()),
            method: "GET".to_string(),
            url: "https://api.example.com".to_string(),
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

    let request_var = RequestVariable {
        reference: "{{pending.response.body.$.token}}".to_string(),
        request_name: "pending".to_string(),
        source: RequestVariableSource::Response,
        target: RequestVariableTarget::Body,
        path: "$.token".to_string(),
    };

    let result = extract_request_variable_value(&request_var, &context).unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_extract_request_variable_value_missing_response_body() {
    let context = vec![RequestContext {
        name: "test".to_string(),
        request: HttpRequest {
            name: Some("test".to_string()),
            method: "GET".to_string(),
            url: "https://api.example.com".to_string(),
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
            request_name: Some("test".to_string()),
            status_code: 204,
            success: true,
            error_message: None,
            duration_ms: 50,
            response_headers: None,
            response_body: None,
            assertion_results: vec![],
        }),
    }];

    let request_var = RequestVariable {
        reference: "{{test.response.body.$.value}}".to_string(),
        request_name: "test".to_string(),
        source: RequestVariableSource::Response,
        target: RequestVariableTarget::Body,
        path: "$.value".to_string(),
    };

    let result = extract_request_variable_value(&request_var, &context).unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_extract_request_variable_value_missing_request_body() {
    let context = vec![RequestContext {
        name: "get".to_string(),
        request: HttpRequest {
            name: Some("get".to_string()),
            method: "GET".to_string(),
            url: "https://api.example.com".to_string(),
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

    let request_var = RequestVariable {
        reference: "{{get.request.body.*}}".to_string(),
        request_name: "get".to_string(),
        source: RequestVariableSource::Request,
        target: RequestVariableTarget::Body,
        path: "*".to_string(),
    };

    let result = extract_request_variable_value(&request_var, &context).unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_extract_request_variable_value_missing_header() {
    let context = create_test_context();
    let request_var = RequestVariable {
        reference: "{{login.response.headers.NonExistent}}".to_string(),
        request_name: "login".to_string(),
        source: RequestVariableSource::Response,
        target: RequestVariableTarget::Headers,
        path: "NonExistent".to_string(),
    };

    let result = extract_request_variable_value(&request_var, &context).unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_extract_request_variable_value_empty_context() {
    let context = vec![];
    let request_var = RequestVariable {
        reference: "{{login.response.body.$.token}}".to_string(),
        request_name: "login".to_string(),
        source: RequestVariableSource::Response,
        target: RequestVariableTarget::Body,
        path: "$.token".to_string(),
    };

    let result = extract_request_variable_value(&request_var, &context).unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_extract_request_variable_value_nested_json_path() {
    let context = create_test_context();
    let request_var = RequestVariable {
        reference: "{{login.response.body.$.userId}}".to_string(),
        request_name: "login".to_string(),
        source: RequestVariableSource::Response,
        target: RequestVariableTarget::Body,
        path: "$.userId".to_string(),
    };

    let result = extract_request_variable_value(&request_var, &context).unwrap();
    assert_eq!(result, Some("42".to_string()));
}

#[test]
fn test_extract_request_variable_value_request_header_not_found() {
    let request = HttpRequest {
        name: Some("request1".to_string()),
        method: "POST".to_string(),
        url: "http://example.com".to_string(),
        headers: vec![Header {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }],
        body: Some(r#"{"data": "test"}"#.to_string()),
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
        name: "request1".to_string(),
        request,
        result: None,
    }];

    let request_var = RequestVariable {
        reference: "{{request1.request.headers.Authorization}}".to_string(),
        request_name: "request1".to_string(),
        source: RequestVariableSource::Request,
        target: RequestVariableTarget::Headers,
        path: "Authorization".to_string(),
    };

    let result = extract_request_variable_value(&request_var, &context);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);
}

#[test]
fn test_extract_request_variable_value_response_body_no_jsonpath() {
    let request = HttpRequest {
        name: Some("request1".to_string()),
        method: "GET".to_string(),
        url: "http://example.com".to_string(),
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

    let result_data = HttpResult {
        request_name: Some("request1".to_string()),
        status_code: 200,
        success: true,
        error_message: None,
        duration_ms: 100,
        response_headers: Some(HashMap::new()),
        response_body: Some("plain text response".to_string()),
        assertion_results: vec![],
    };

    let context = vec![RequestContext {
        name: "request1".to_string(),
        request,
        result: Some(result_data),
    }];

    let request_var = RequestVariable {
        reference: "{{request1.response.body.simple}}".to_string(),
        request_name: "request1".to_string(),
        source: RequestVariableSource::Response,
        target: RequestVariableTarget::Body,
        path: "simple".to_string(),
    };

    let extracted = extract_request_variable_value(&request_var, &context);

    assert!(extracted.is_ok());
    assert_eq!(extracted.unwrap(), Some("plain text response".to_string()));
}
