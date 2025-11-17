use crate::request_variables;
use crate::types::{Condition, ConditionType, RequestContext};
use anyhow::Result;

/// Evaluates if all conditions for a request are met
pub fn evaluate_conditions(conditions: &[Condition], context: &[RequestContext]) -> Result<bool> {
    if conditions.is_empty() {
        return Ok(true);
    }

    for condition in conditions {
        if !evaluate_single_condition(condition, context)? {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Evaluates a single condition
fn evaluate_single_condition(condition: &Condition, context: &[RequestContext]) -> Result<bool> {
    // Find the request context by name
    let target_context = context
        .iter()
        .find(|ctx| ctx.name == condition.request_name);

    let Some(ctx) = target_context else {
        // Referenced request not found or not executed yet
        return Ok(false);
    };

    // Check if the request has a result
    let result = match &ctx.result {
        Some(r) => r,
        None => return Ok(false), // Request failed, condition not met
    };

    let condition_met = match &condition.condition_type {
        ConditionType::Status => {
            // Compare status code
            let expected_status = condition.expected_value.trim();
            let actual_status = result.status_code.to_string();
            actual_status == expected_status
        }
        ConditionType::BodyJsonPath(json_path) => {
            // Extract value using JSONPath and compare
            if let Some(ref body) = result.response_body {
                let extracted_value = extract_json_value(body, json_path)?;
                if let Some(value) = extracted_value {
                    value.trim() == condition.expected_value.trim()
                } else {
                    false
                }
            } else {
                false
            }
        }
    };

    // Apply negation if @if-not
    Ok(if condition.negate {
        !condition_met
    } else {
        condition_met
    })
}

/// Extracts a value from JSON using a JSONPath expression
fn extract_json_value(json_body: &str, json_path: &str) -> Result<Option<String>> {
    // Handle $.property format
    if let Some(property) = json_path.strip_prefix("$.") {
        return request_variables::extract_json_property(json_body, property);
    }

    Ok(None)
}

/// Checks if a request's dependencies are met (for @dependsOn)
pub fn check_dependency(depends_on: &Option<String>, context: &[RequestContext]) -> bool {
    if let Some(dep_name) = depends_on {
        // Find the dependent request
        let target_context = context.iter().find(|ctx| ctx.name == *dep_name);

        if let Some(ctx) = target_context {
            // Check if the request succeeded (returned HTTP 200)
            if let Some(ref result) = ctx.result {
                return result.status_code == 200;
            }
        }
        // Dependency not found or not executed
        return false;
    }

    // No dependency, always satisfied
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{HttpRequest, HttpResult};
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

        assert!(evaluate_conditions(&vec![condition], &context).unwrap());
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

        assert!(!evaluate_conditions(&vec![condition], &context).unwrap());
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

        assert!(evaluate_conditions(&vec![condition], &context).unwrap());
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

        assert!(!evaluate_conditions(&vec![condition], &context).unwrap());
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
        // @if-not status 404 should pass when status is 200
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

        assert!(evaluate_conditions(&vec![condition], &context).unwrap());
    }

    #[test]
    fn test_evaluate_status_condition_negated_failure() {
        // @if-not status 200 should fail when status is 200
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

        assert!(!evaluate_conditions(&vec![condition], &context).unwrap());
    }

    #[test]
    fn test_evaluate_body_jsonpath_condition_negated_success() {
        // @if-not body value should pass when value doesn't match
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

        assert!(evaluate_conditions(&vec![condition], &context).unwrap());
    }

    #[test]
    fn test_evaluate_body_jsonpath_condition_negated_failure() {
        // @if-not body value should fail when value matches
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

        assert!(!evaluate_conditions(&vec![condition], &context).unwrap());
    }
}
