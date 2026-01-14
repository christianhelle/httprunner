use super::assertion::AssertionResult;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpResult {
    pub request_name: Option<String>,
    pub status_code: u16,
    pub success: bool,
    pub error_message: Option<String>,
    pub duration_ms: u64,
    pub response_headers: Option<HashMap<String, String>>,
    pub response_body: Option<String>,
    pub assertion_results: Vec<AssertionResult>,
}
