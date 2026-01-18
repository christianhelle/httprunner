use crate::types::ConditionType;

pub fn format_condition_type(condition_type: &ConditionType) -> String {
    match condition_type {
        ConditionType::Status => "status".to_string(),
        ConditionType::BodyJsonPath(path) => format!("body.{}", path),
    }
}
