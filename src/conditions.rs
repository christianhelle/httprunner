use crate::request_variables;
use crate::types::{Condition, ConditionType, RequestContext};
use anyhow::Result;

#[derive(Debug)]
pub struct ConditionEvaluationResult {
    pub condition_met: bool,
    pub actual_value: Option<String>,
    pub expected_value: String,
    pub condition_type: String,
    pub negated: bool,
}

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

/// Evaluates conditions with detailed results for verbose output
pub fn evaluate_conditions_verbose(
    conditions: &[Condition],
    context: &[RequestContext],
) -> Result<(bool, Vec<ConditionEvaluationResult>)> {
    if conditions.is_empty() {
        return Ok((true, vec![]));
    }

    let mut results = Vec::new();
    let mut all_met = true;

    for condition in conditions {
        let result = evaluate_single_condition_verbose(condition, context)?;
        if !result.condition_met {
            all_met = false;
        }
        results.push(result);
    }

    Ok((all_met, results))
}

/// Evaluates a single condition
fn evaluate_single_condition(condition: &Condition, context: &[RequestContext]) -> Result<bool> {
    let result = evaluate_single_condition_verbose(condition, context)?;
    Ok(result.condition_met)
}

/// Evaluates a single condition with detailed result
fn evaluate_single_condition_verbose(
    condition: &Condition,
    context: &[RequestContext],
) -> Result<ConditionEvaluationResult> {
    // Find the request context by name
    let target_context = context
        .iter()
        .find(|ctx| ctx.name == condition.request_name);

    let Some(ctx) = target_context else {
        // Referenced request not found or not executed yet
        return Ok(ConditionEvaluationResult {
            condition_met: false,
            actual_value: None,
            expected_value: condition.expected_value.clone(),
            condition_type: format_condition_type(&condition.condition_type),
            negated: condition.negate,
        });
    };

    // Check if the request has a result
    let result = match &ctx.result {
        Some(r) => r,
        None => {
            return Ok(ConditionEvaluationResult {
                condition_met: false,
                actual_value: None,
                expected_value: condition.expected_value.clone(),
                condition_type: format_condition_type(&condition.condition_type),
                negated: condition.negate,
            });
        }
    };

    let (actual_value, base_condition_met) = match &condition.condition_type {
        ConditionType::Status => {
            // Compare status code
            let expected_status = condition.expected_value.trim();
            let actual_status = result.status_code.to_string();
            let met = actual_status == expected_status;
            (Some(actual_status), met)
        }
        ConditionType::BodyJsonPath(json_path) => {
            // Extract value using JSONPath and compare
            if let Some(ref body) = result.response_body {
                let extracted_value = extract_json_value(body, json_path)?;
                if let Some(value) = extracted_value {
                    let met = value.trim() == condition.expected_value.trim();
                    (Some(value), met)
                } else {
                    (Some("<not found>".to_string()), false)
                }
            } else {
                (Some("<no body>".to_string()), false)
            }
        }
    };

    // Apply negation if @if-not
    let condition_met = if condition.negate {
        !base_condition_met
    } else {
        base_condition_met
    };

    Ok(ConditionEvaluationResult {
        condition_met,
        actual_value,
        expected_value: condition.expected_value.clone(),
        condition_type: format_condition_type(&condition.condition_type),
        negated: condition.negate,
    })
}

fn format_condition_type(condition_type: &ConditionType) -> String {
    match condition_type {
        ConditionType::Status => "status".to_string(),
        ConditionType::BodyJsonPath(path) => format!("body.{}", path),
    }
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
