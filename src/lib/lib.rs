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
pub mod processor;
pub mod report;
pub mod runner;
pub mod serializer;
pub mod types;
pub mod variables;

// Re-export commonly used types
pub use types::{HttpRequest, HttpResult};
