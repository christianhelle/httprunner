use crate::types::{HttpRequest, HttpResult};
use anyhow::Result;

/// Trait for executing HTTP requests.
/// This is the Rust equivalent of a C# interface, allowing for dependency injection
/// and test doubles (mocks/stubs) to be used in testing.
pub trait HttpExecutor {
    /// Executes an HTTP request and returns the result.
    ///
    /// # Parameters
    ///
    /// - `request`: the HTTP request to execute
    /// - `verbose`: whether to include verbose response details
    /// - `insecure`: whether to accept invalid TLS certificates
    ///
    /// # Returns
    ///
    /// An `HttpResult` containing the response details and any assertion results
    fn execute(&self, request: &HttpRequest, verbose: bool, insecure: bool) -> Result<HttpResult>;
}
