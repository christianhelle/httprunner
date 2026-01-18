use crate::types::{Condition, ConditionType};

pub fn parse_condition(value: &str, negate: bool) -> Option<Condition> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let reference = parts[0];
    let expected_value = parts[1..].join(" ");

    let ref_parts: Vec<&str> = reference.split('.').collect();

    if ref_parts.len() < 3 {
        return None;
    }

    let request_name = ref_parts[0].to_string();

    if ref_parts.len() == 3 && ref_parts[1] == "response" && ref_parts[2] == "status" {
        return Some(Condition {
            request_name,
            condition_type: ConditionType::Status,
            expected_value,
            negate,
        });
    }

    if ref_parts.len() >= 4 && ref_parts[1] == "response" && ref_parts[2] == "body" {
        let json_path = ref_parts[3..].join(".");
        return Some(Condition {
            request_name,
            condition_type: ConditionType::BodyJsonPath(json_path),
            expected_value,
            negate,
        });
    }

    None
}
