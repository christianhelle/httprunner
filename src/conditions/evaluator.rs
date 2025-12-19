use super::formatter::format_condition_type;
use super::json_extractor::extract_json_value;
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

fn evaluate_single_condition(condition: &Condition, context: &[RequestContext]) -> Result<bool> {
    let result = evaluate_single_condition_verbose(condition, context)?;
    Ok(result.condition_met)
}

fn evaluate_single_condition_verbose(
    condition: &Condition,
    context: &[RequestContext],
) -> Result<ConditionEvaluationResult> {
    let target_context = context
        .iter()
        .find(|ctx| ctx.name == condition.request_name);

    let Some(ctx) = target_context else {
        return Ok(ConditionEvaluationResult {
            condition_met: false,
            actual_value: None,
            expected_value: condition.expected_value.clone(),
            condition_type: format_condition_type(&condition.condition_type),
            negated: condition.negate,
        });
    };

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
            let expected_status = condition.expected_value.trim();
            let actual_status = result.status_code.to_string();
            let met = actual_status == expected_status;
            (Some(actual_status), met)
        }
        ConditionType::BodyJsonPath(json_path) => {
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
