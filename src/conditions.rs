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

/// Determines whether every condition in `conditions` is satisfied for the given request context.
///
/// # Returns
///
/// `true` if all conditions are met or if `conditions` is empty, `false` otherwise.
///
/// # Examples
///
/// ```
/// # use crate::conditions::evaluate_conditions;
/// # use crate::{Condition, RequestContext};
/// let conditions: Vec<Condition> = vec![];
/// let context: Vec<RequestContext> = vec![];
/// assert!(evaluate_conditions(&conditions, &context).unwrap());
/// ```
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

/// Evaluates all provided conditions and returns whether every condition was met along with detailed per-condition results.
///
/// # Returns
///
/// `Ok((true, vec![]))` if `conditions` is empty. Otherwise `Ok((all_met, results))` where `all_met` is `true` only if every condition evaluated as met, and `results` is a `Vec<ConditionEvaluationResult>` with one entry per condition describing its outcome.
///
/// # Examples
///
/// ```
/// let (all_met, results) = evaluate_conditions_verbose(&conditions, &context)?;
/// // `all_met` is true only if every condition in `conditions` was satisfied
/// // `results` contains detailed evaluation for each condition
/// ```
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

/// Determine whether a single `Condition` is satisfied using the provided request contexts.
///
/// This evaluates the condition against the available `RequestContext` entries and applies any negation (`@if-not`) specified by the condition.
///
/// # Returns
///
/// `true` if the condition is satisfied, `false` otherwise.
///
/// # Examples
///
/// ```no_run
/// // Example usage (types and construction depend on your test helpers):
/// // let condition = Condition::status_equals("request-name", "200");
/// // let contexts: Vec<RequestContext> = ...;
/// // assert!(evaluate_single_condition(&condition, &contexts).unwrap());
/// ```
fn evaluate_single_condition(condition: &Condition, context: &[RequestContext]) -> Result<bool> {
    let result = evaluate_single_condition_verbose(condition, context)?;
    Ok(result.condition_met)
}

/// Evaluates a single condition against the provided request contexts and returns a detailed result.
///
/// If the referenced request is missing or has no result, the returned `ConditionEvaluationResult` has
/// `condition_met = false` and `actual_value = None`. For `Status` conditions, `actual_value` is the
/// response status code as a string. For `BodyJsonPath` conditions, `actual_value` is the extracted
/// value when found, `"<not found>"` when the path yields no value, or `"<no body>"` when the response
/// body is absent. The final `condition_met` in the result reflects `condition.negate` (i.e., it is
/// inverted when `@if-not` is used).
///
/// # Examples
///
/// ```
/// # use crate::{Condition, ConditionType, RequestContext, HttpResult};
/// let cond = Condition {
///     request_name: "login".into(),
///     condition_type: ConditionType::Status,
///     expected_value: "200".into(),
///     negate: false,
/// };
/// let ctx = RequestContext {
///     name: "login".into(),
///     result: Some(HttpResult { status_code: 200, response_body: None }),
/// };
/// let res = evaluate_single_condition_verbose(&cond, &[ctx]).unwrap();
/// assert!(res.condition_met);
/// assert_eq!(res.actual_value, Some("200".to_string()));
/// ```
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

/// Converts a `ConditionType` into a concise, human-readable string.
///
/// The returned string is `"status"` for `ConditionType::Status` or `"body.<path>"`
/// for `ConditionType::BodyJsonPath(path)`.
///
/// # Examples
///
/// ```
/// # use crate::ConditionType;
/// assert_eq!(format_condition_type(&ConditionType::Status), "status");
/// assert_eq!(format_condition_type(&ConditionType::BodyJsonPath("$.user.name".into())), "body.$.user.name");
/// ```
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
            // Check if the request succeeded (returned any 2xx status code)
            if let Some(ref result) = ctx.result {
                return result.success;
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
        };

        let result = HttpResult {
            request_name: Some("request1".to_string()),
            status_code: 201,
            success: true, // 201 Created is a success
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
