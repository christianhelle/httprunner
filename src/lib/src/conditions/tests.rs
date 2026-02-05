use super::*;
use crate::types::{Condition, ConditionType, HttpRequest, HttpResult, RequestContext};
use std::collections::HashMap;

#[test]
fn test_evaluate_conditions_empty() {
    let conditions = vec![];
    let context = vec![];
    assert!(evaluate_conditions(&conditions, &context).unwrap());
}

#[test]
fn test_evaluate_status_condition_success() {
    let condition = Condition {
        request_name: "request1".to_string(),
        condition_type: ConditionType::Status,
        expected_value: "200".to_string(),
        negate: false,
    };

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

    let result = HttpResult {
        request_name: Some("request1".to_string()),
        status_code: 200,
        success: true,
        error_message: None,
        duration_ms: 100,
        response_headers: Some(HashMap::new()),
        response_body: Some("test body".to_string()),
        assertion_results: vec![],
    };

    let context = vec![RequestContext {
        name: "request1".to_string(),
        request,
        result: Some(result),
    }];

    assert!(evaluate_conditions(&[condition], &context).unwrap());
}

#[test]
fn test_evaluate_status_condition_failure() {
    let condition = Condition {
        request_name: "request1".to_string(),
        condition_type: ConditionType::Status,
        expected_value: "404".to_string(),
        negate: false,
    };

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

    let result = HttpResult {
        request_name: Some("request1".to_string()),
        status_code: 200,
        success: true,
        error_message: None,
        duration_ms: 100,
        response_headers: Some(HashMap::new()),
        response_body: Some("test body".to_string()),
        assertion_results: vec![],
    };

    let context = vec![RequestContext {
        name: "request1".to_string(),
        request,
        result: Some(result),
    }];

    assert!(!evaluate_conditions(&[condition], &context).unwrap());
}

#[test]
fn test_check_dependency_success() {
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

    let result = HttpResult {
        request_name: Some("request1".to_string()),
        status_code: 200,
        success: true,
        error_message: None,
        duration_ms: 100,
        response_headers: Some(HashMap::new()),
        response_body: Some("test body".to_string()),
        assertion_results: vec![],
    };

    let context = vec![RequestContext {
        name: "request1".to_string(),
        request,
        result: Some(result),
    }];

    assert!(check_dependency(&Some("request1".to_string()), &context));
}

#[test]
fn test_check_dependency_not_200() {
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

    let result = HttpResult {
        request_name: Some("request1".to_string()),
        status_code: 404,
        success: false,
        error_message: None,
        duration_ms: 100,
        response_headers: Some(HashMap::new()),
        response_body: Some("not found".to_string()),
        assertion_results: vec![],
    };

    let context = vec![RequestContext {
        name: "request1".to_string(),
        request,
        result: Some(result),
    }];

    assert!(!check_dependency(&Some("request1".to_string()), &context));
}

#[test]
fn test_check_dependency_201_created() {
    let request = HttpRequest {
        name: Some("request1".to_string()),
        method: "POST".to_string(),
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

    let result = HttpResult {
        request_name: Some("request1".to_string()),
        status_code: 201,
        success: true,
        error_message: None,
        duration_ms: 100,
        response_headers: Some(HashMap::new()),
        response_body: Some(r#"{"id": 1}"#.to_string()),
        assertion_results: vec![],
    };

    let context = vec![RequestContext {
        name: "request1".to_string(),
        request,
        result: Some(result),
    }];

    assert!(check_dependency(&Some("request1".to_string()), &context));
}

#[test]
fn test_check_dependency_none() {
    let context = vec![];
    assert!(check_dependency(&None, &context));
}

#[test]
fn test_evaluate_body_jsonpath_condition_success() {
    let condition = Condition {
        request_name: "request1".to_string(),
        condition_type: ConditionType::BodyJsonPath("$.username".to_string()),
        expected_value: "testuser".to_string(),
        negate: false,
    };

    let request = HttpRequest {
        name: Some("request1".to_string()),
        method: "POST".to_string(),
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

    let result = HttpResult {
        request_name: Some("request1".to_string()),
        status_code: 200,
        success: true,
        error_message: None,
        duration_ms: 100,
        response_headers: Some(HashMap::new()),
        response_body: Some(r#"{"username": "testuser", "id": 123}"#.to_string()),
        assertion_results: vec![],
    };

    let context = vec![RequestContext {
        name: "request1".to_string(),
        request,
        result: Some(result),
    }];

    assert!(evaluate_conditions(&[condition], &context).unwrap());
}

#[test]
fn test_evaluate_body_jsonpath_condition_failure() {
    let condition = Condition {
        request_name: "request1".to_string(),
        condition_type: ConditionType::BodyJsonPath("$.username".to_string()),
        expected_value: "wronguser".to_string(),
        negate: false,
    };

    let request = HttpRequest {
        name: Some("request1".to_string()),
        method: "POST".to_string(),
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

    let result = HttpResult {
        request_name: Some("request1".to_string()),
        status_code: 200,
        success: true,
        error_message: None,
        duration_ms: 100,
        response_headers: Some(HashMap::new()),
        response_body: Some(r#"{"username": "testuser", "id": 123}"#.to_string()),
        assertion_results: vec![],
    };

    let context = vec![RequestContext {
        name: "request1".to_string(),
        request,
        result: Some(result),
    }];

    assert!(!evaluate_conditions(&[condition], &context).unwrap());
}

#[test]
fn test_evaluate_multiple_conditions_all_met() {
    let conditions = vec![
        Condition {
            request_name: "request1".to_string(),
            condition_type: ConditionType::Status,
            expected_value: "200".to_string(),
            negate: false,
        },
        Condition {
            request_name: "request1".to_string(),
            condition_type: ConditionType::BodyJsonPath("$.username".to_string()),
            expected_value: "testuser".to_string(),
            negate: false,
        },
    ];

    let request = HttpRequest {
        name: Some("request1".to_string()),
        method: "POST".to_string(),
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

    let result = HttpResult {
        request_name: Some("request1".to_string()),
        status_code: 200,
        success: true,
        error_message: None,
        duration_ms: 100,
        response_headers: Some(HashMap::new()),
        response_body: Some(r#"{"username": "testuser", "id": 123}"#.to_string()),
        assertion_results: vec![],
    };

    let context = vec![RequestContext {
        name: "request1".to_string(),
        request,
        result: Some(result),
    }];

    assert!(evaluate_conditions(&conditions, &context).unwrap());
}

#[test]
fn test_evaluate_multiple_conditions_one_fails() {
    let conditions = vec![
        Condition {
            request_name: "request1".to_string(),
            condition_type: ConditionType::Status,
            expected_value: "200".to_string(),
            negate: false,
        },
        Condition {
            request_name: "request1".to_string(),
            condition_type: ConditionType::BodyJsonPath("$.username".to_string()),
            expected_value: "wronguser".to_string(),
            negate: false,
        },
    ];

    let request = HttpRequest {
        name: Some("request1".to_string()),
        method: "POST".to_string(),
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

    let result = HttpResult {
        request_name: Some("request1".to_string()),
        status_code: 200,
        success: true,
        error_message: None,
        duration_ms: 100,
        response_headers: Some(HashMap::new()),
        response_body: Some(r#"{"username": "testuser", "id": 123}"#.to_string()),
        assertion_results: vec![],
    };

    let context = vec![RequestContext {
        name: "request1".to_string(),
        request,
        result: Some(result),
    }];

    assert!(!evaluate_conditions(&conditions, &context).unwrap());
}

#[test]
fn test_evaluate_status_condition_negated_success() {
    let condition = Condition {
        request_name: "request1".to_string(),
        condition_type: ConditionType::Status,
        expected_value: "404".to_string(),
        negate: true,
    };

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

    let result = HttpResult {
        request_name: Some("request1".to_string()),
        status_code: 200,
        success: true,
        error_message: None,
        duration_ms: 100,
        response_headers: Some(HashMap::new()),
        response_body: Some("test body".to_string()),
        assertion_results: vec![],
    };

    let context = vec![RequestContext {
        name: "request1".to_string(),
        request,
        result: Some(result),
    }];

    assert!(evaluate_conditions(&[condition], &context).unwrap());
}

#[test]
fn test_evaluate_status_condition_negated_failure() {
    let condition = Condition {
        request_name: "request1".to_string(),
        condition_type: ConditionType::Status,
        expected_value: "200".to_string(),
        negate: true,
    };

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

    let result = HttpResult {
        request_name: Some("request1".to_string()),
        status_code: 200,
        success: true,
        error_message: None,
        duration_ms: 100,
        response_headers: Some(HashMap::new()),
        response_body: Some("test body".to_string()),
        assertion_results: vec![],
    };

    let context = vec![RequestContext {
        name: "request1".to_string(),
        request,
        result: Some(result),
    }];

    assert!(!evaluate_conditions(&[condition], &context).unwrap());
}

#[test]
fn test_evaluate_body_jsonpath_condition_negated_success() {
    let condition = Condition {
        request_name: "request1".to_string(),
        condition_type: ConditionType::BodyJsonPath("$.username".to_string()),
        expected_value: "wronguser".to_string(),
        negate: true,
    };

    let request = HttpRequest {
        name: Some("request1".to_string()),
        method: "POST".to_string(),
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

    let result = HttpResult {
        request_name: Some("request1".to_string()),
        status_code: 200,
        success: true,
        error_message: None,
        duration_ms: 100,
        response_headers: Some(HashMap::new()),
        response_body: Some(r#"{"username": "testuser", "id": 123}"#.to_string()),
        assertion_results: vec![],
    };

    let context = vec![RequestContext {
        name: "request1".to_string(),
        request,
        result: Some(result),
    }];

    assert!(evaluate_conditions(&[condition], &context).unwrap());
}

#[test]
fn test_evaluate_body_jsonpath_condition_negated_failure() {
    let condition = Condition {
        request_name: "request1".to_string(),
        condition_type: ConditionType::BodyJsonPath("$.username".to_string()),
        expected_value: "testuser".to_string(),
        negate: true,
    };

    let request = HttpRequest {
        name: Some("request1".to_string()),
        method: "POST".to_string(),
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

    let result = HttpResult {
        request_name: Some("request1".to_string()),
        status_code: 200,
        success: true,
        error_message: None,
        duration_ms: 100,
        response_headers: Some(HashMap::new()),
        response_body: Some(r#"{"username": "testuser", "id": 123}"#.to_string()),
        assertion_results: vec![],
    };

    let context = vec![RequestContext {
        name: "request1".to_string(),
        request,
        result: Some(result),
    }];

    assert!(!evaluate_conditions(&[condition], &context).unwrap());
}

#[test]
fn test_evaluate_conditions_verbose_empty() {
    let conditions = vec![];
    let context = vec![];
    let (all_met, results) = evaluate_conditions_verbose(&conditions, &context).unwrap();
    assert!(all_met);
    assert!(results.is_empty());
}

#[test]
fn test_evaluate_conditions_verbose_request_not_found() {
    let condition = Condition {
        request_name: "nonexistent".to_string(),
        condition_type: ConditionType::Status,
        expected_value: "200".to_string(),
        negate: false,
    };

    let context = vec![];
    let (all_met, results) = evaluate_conditions_verbose(&[condition], &context).unwrap();

    assert!(!all_met);
    assert_eq!(results.len(), 1);
    assert!(!results[0].condition_met);
    assert_eq!(results[0].actual_value, None);
}

#[test]
fn test_evaluate_conditions_verbose_result_is_none() {
    let condition = Condition {
        request_name: "request1".to_string(),
        condition_type: ConditionType::Status,
        expected_value: "200".to_string(),
        negate: false,
    };

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

    let context = vec![RequestContext {
        name: "request1".to_string(),
        request,
        result: None,
    }];

    let (all_met, results) = evaluate_conditions_verbose(&[condition], &context).unwrap();

    assert!(!all_met);
    assert_eq!(results.len(), 1);
    assert!(!results[0].condition_met);
    assert_eq!(results[0].actual_value, None);
}

#[test]
fn test_evaluate_conditions_verbose_no_response_body() {
    let condition = Condition {
        request_name: "request1".to_string(),
        condition_type: ConditionType::BodyJsonPath("$.test".to_string()),
        expected_value: "value".to_string(),
        negate: false,
    };

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

    let result = HttpResult {
        request_name: Some("request1".to_string()),
        status_code: 200,
        success: true,
        error_message: None,
        duration_ms: 100,
        response_headers: Some(HashMap::new()),
        response_body: None,
        assertion_results: vec![],
    };

    let context = vec![RequestContext {
        name: "request1".to_string(),
        request,
        result: Some(result),
    }];

    let (all_met, results) = evaluate_conditions_verbose(&[condition], &context).unwrap();

    assert!(!all_met);
    assert_eq!(results.len(), 1);
    assert!(!results[0].condition_met);
    assert_eq!(results[0].actual_value, Some("<no body>".to_string()));
}

#[test]
fn test_evaluate_conditions_verbose_json_path_not_found() {
    let condition = Condition {
        request_name: "request1".to_string(),
        condition_type: ConditionType::BodyJsonPath("$.nonexistent".to_string()),
        expected_value: "value".to_string(),
        negate: false,
    };

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

    let result = HttpResult {
        request_name: Some("request1".to_string()),
        status_code: 200,
        success: true,
        error_message: None,
        duration_ms: 100,
        response_headers: Some(HashMap::new()),
        response_body: Some(r#"{"test": "data"}"#.to_string()),
        assertion_results: vec![],
    };

    let context = vec![RequestContext {
        name: "request1".to_string(),
        request,
        result: Some(result),
    }];

    let (all_met, results) = evaluate_conditions_verbose(&[condition], &context).unwrap();

    assert!(!all_met);
    assert_eq!(results.len(), 1);
    assert!(!results[0].condition_met);
    assert_eq!(results[0].actual_value, Some("<not found>".to_string()));
}


