use serde::{Deserialize, Serialize};
use std::fmt;

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

impl fmt::Display for AssertionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssertionType::Status => write!(f, "Status Code"),
            AssertionType::Body => write!(f, "Response Body"),
            AssertionType::Headers => write!(f, "Response Headers"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResult {
    pub assertion: Assertion,
    pub passed: bool,
    pub actual_value: Option<String>,
    pub error_message: Option<String>,
}
