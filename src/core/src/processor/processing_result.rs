use crate::types::{HttpRequest, HttpResult};

/// Result of processing a single request.
#[derive(Debug)]
pub enum RequestProcessingResult {
    /// Request was skipped due to conditions or dependencies.
    Skipped {
        request: HttpRequest,
        reason: String,
    },
    /// Request was executed successfully or with errors.
    Executed {
        request: HttpRequest,
        result: HttpResult,
    },
    /// Request processing failed with an error.
    Failed { request: HttpRequest, error: String },
}
