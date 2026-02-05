use super::assertion::Assertion;
use super::condition::Condition;
use super::variable::Variable;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub name: Option<String>,
    pub method: String,
    pub url: String,
    pub headers: Vec<Header>,
    pub body: Option<String>,
    pub assertions: Vec<Assertion>,
    #[allow(dead_code)]
    pub variables: Vec<Variable>,
    pub timeout: Option<u64>,            // Read timeout in milliseconds
    pub connection_timeout: Option<u64>, // Connection timeout in milliseconds
    pub depends_on: Option<String>,      // Request name this depends on (for @dependsOn)
    pub conditions: Vec<Condition>,      // Conditions for execution (for @if)
    pub pre_delay_ms: Option<u64>,       // Delay before executing request (for @pre-delay)
    pub post_delay_ms: Option<u64>,      // Delay after executing request (for @post-delay)
}

#[derive(Debug, Clone)]
pub struct Header {
    pub name: String,
    pub value: String,
}
