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
