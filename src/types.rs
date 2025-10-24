use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct RequestVariable {
    #[allow(dead_code)]
    pub reference: String,
    pub request_name: String,
    pub source: RequestVariableSource,
    pub target: RequestVariableTarget,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestVariableSource {
    Request,
    Response,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestVariableTarget {
    Body,
    Headers,
}

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
}

#[derive(Debug, Clone)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Assertion {
    pub assertion_type: AssertionType,
    pub expected_value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssertionType {
    ResponseStatus,
    ResponseBody,
    ResponseHeaders,
}

#[derive(Debug)]
pub struct AssertionResult {
    pub assertion: Assertion,
    pub passed: bool,
    pub actual_value: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct RequestContext {
    pub name: String,
    pub request: HttpRequest,
    pub result: Option<HttpResult>,
}
