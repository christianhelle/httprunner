use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assertion {
    pub assertion_type: AssertionType,
    pub expected_value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssertionType {
    Status,
    Body,
    Headers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResult {
    pub assertion: Assertion,
    pub passed: bool,
    pub actual_value: Option<String>,
    pub error_message: Option<String>,
}
