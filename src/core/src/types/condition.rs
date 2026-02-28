use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Condition {
    pub request_name: String,
    pub condition_type: ConditionType,
    pub expected_value: String,
    pub negate: bool, // true for @if-not, false for @if
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ConditionType {
    Status,               // Check response status code
    BodyJsonPath(String), // Check JSONPath expression in response body
}
