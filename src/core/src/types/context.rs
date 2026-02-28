use super::request::HttpRequest;
use super::result::HttpResult;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RequestContext {
    pub name: String,
    pub request: HttpRequest,
    pub result: Option<HttpResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HttpFileResults {
    pub filename: String,
    pub success_count: u32,
    pub failed_count: u32,
    pub skipped_count: u32,
    pub result_contexts: Vec<RequestContext>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessorResults {
    pub success: bool,
    pub files: Vec<HttpFileResults>,
}
