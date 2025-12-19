use crate::types::{Assertion, AssertionResult, AssertionType, HttpResult};

/// Evaluate all assertions against an HTTP result
///
/// Takes a list of assertions and evaluates each one against the provided
/// HTTP result, returning a list of assertion results indicating pass/fail
/// status and detailed error messages.
///
/// # Arguments
///
/// * `assertions` - List of assertions to evaluate
/// * `result` - The HTTP result to evaluate assertions against
///
/// # Returns
///
/// A vector of `AssertionResult` containing the outcome of each assertion
pub fn evaluate_assertions(assertions: &[Assertion], result: &HttpResult) -> Vec<AssertionResult> {
    assertions
        .iter()
        .map(|assertion| evaluate_assertion(assertion, result))
        .collect()
}

/// Evaluate a single assertion against an HTTP result
///
/// Checks if the assertion condition is met based on the assertion type
/// (status code, response body, or response headers).
///
/// # Arguments
///
/// * `assertion` - The assertion to evaluate
/// * `result` - The HTTP result to check against
///
/// # Returns
///
/// An `AssertionResult` indicating whether the assertion passed and details
pub(crate) fn evaluate_assertion(assertion: &Assertion, result: &HttpResult) -> AssertionResult {
    match assertion.assertion_type {
        AssertionType::Status => {
            let expected_status = match assertion.expected_value.parse::<u16>() {
                Ok(status) => status,
                Err(_) => {
                    return AssertionResult {
                        assertion: assertion.clone(),
                        passed: false,
                        actual_value: Some(result.status_code.to_string()),
                        error_message: Some("Invalid expected status code format".to_string()),
                    };
                }
            };

            let passed = result.status_code == expected_status;
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                actual_value: Some(result.status_code.to_string()),
                error_message: if !passed {
                    Some(format!(
                        "Expected status {}, got {}",
                        expected_status, result.status_code
                    ))
                } else {
                    None
                },
            }
        }

        AssertionType::Body => {
            if let Some(ref body) = result.response_body {
                let passed = body.contains(&assertion.expected_value);
                AssertionResult {
                    assertion: assertion.clone(),
                    passed,
                    actual_value: Some(body.clone()),
                    error_message: if !passed {
                        Some(format!(
                            "Expected body to contain '{}'",
                            assertion.expected_value
                        ))
                    } else {
                        None
                    },
                }
            } else {
                AssertionResult {
                    assertion: assertion.clone(),
                    passed: false,
                    actual_value: Some(String::new()),
                    error_message: Some("No response body available".to_string()),
                }
            }
        }

        AssertionType::Headers => {
            if let Some(ref headers) = result.response_headers {
                let colon_pos = assertion.expected_value.find(':');
                if colon_pos.is_none() {
                    return AssertionResult {
                        assertion: assertion.clone(),
                        passed: false,
                        actual_value: Some(format_headers(headers)),
                        error_message: Some(
                            "Invalid header format, expected 'Name: Value'".to_string(),
                        ),
                    };
                }

                let colon_pos = colon_pos.unwrap();
                let expected_name = assertion.expected_value[..colon_pos].trim();
                let expected_value = assertion.expected_value[colon_pos + 1..].trim();

                let mut found = false;
                for (name, value) in headers {
                    if name.eq_ignore_ascii_case(expected_name) && value.contains(expected_value) {
                        found = true;
                        break;
                    }
                }

                AssertionResult {
                    assertion: assertion.clone(),
                    passed: found,
                    actual_value: Some(format_headers(headers)),
                    error_message: if !found {
                        Some(format!(
                            "Expected header '{}' with value containing '{}'",
                            expected_name, expected_value
                        ))
                    } else {
                        None
                    },
                }
            } else {
                AssertionResult {
                    assertion: assertion.clone(),
                    passed: false,
                    actual_value: Some(String::new()),
                    error_message: Some("No response headers available".to_string()),
                }
            }
        }
    }
}

fn format_headers(headers: &std::collections::HashMap<String, String>) -> String {
    headers
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<_>>()
        .join(", ")
}
