// Library exports for httprunner
// This allows the GUI and CLI to share the same core logic

pub mod assertions;
pub mod colors;
pub mod conditions;
pub mod discovery;
pub mod environment;
pub mod functions;
pub mod logging;
pub mod parser;
pub mod report;
pub mod runner;
pub mod serializer;
pub mod types;
pub mod variables;

// Processor is only used by CLI (sync/blocking), not available on WASM
#[cfg(not(target_arch = "wasm32"))]
pub mod processor;

// Re-export commonly used types
pub use types::{HttpRequest, HttpResult};

// Platform-specific exports
#[cfg(target_arch = "wasm32")]
pub use runner::execute_http_request_async;
