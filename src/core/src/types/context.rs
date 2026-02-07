use super::request::HttpRequest;
use super::result::HttpResult;

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub name: String,
    pub request: HttpRequest,
    pub result: Option<HttpResult>,
}

impl RequestContext {
    /// Creates a new RequestContext, deriving the name from the request or a fallback index.
    pub fn new(request: HttpRequest, result: Option<HttpResult>, request_count: u32) -> Self {
        let name = request
            .name
            .clone()
            .unwrap_or_else(|| format!("request_{}", request_count));
        Self {
            name,
            request,
            result,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpFileResults {
    pub filename: String,
    pub success_count: u32,
    pub failed_count: u32,
    pub skipped_count: u32,
    pub result_contexts: Vec<RequestContext>,
}

#[derive(Debug, Clone)]
pub struct ProcessorResults {
    pub success: bool,
    pub files: Vec<HttpFileResults>,
}
