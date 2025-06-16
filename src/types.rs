

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<Header>,
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct HttpResult {
    pub status_code: u16,
    pub success: bool,
    pub error_message: Option<String>,
    pub duration_ms: u64,
    pub response_headers: Option<Vec<Header>>,
    pub response_body: Option<String>,
}

impl HttpRequest {
    pub fn new(method: String, url: String) -> Self {
        Self {
            method,
            url,
            headers: Vec::new(),
            body: None,
        }
    }

    pub fn add_header(&mut self, name: String, value: String) {
        self.headers.push(Header { name, value });
    }

    pub fn set_body(&mut self, body: String) {
        self.body = Some(body);
    }
}

impl HttpResult {
    pub fn new(status_code: u16, duration_ms: u64) -> Self {
        let success = status_code >= 200 && status_code < 300;
        Self {
            status_code,
            success,
            error_message: None,
            duration_ms,
            response_headers: None,
            response_body: None,
        }
    }

    pub fn with_error(error_message: String, duration_ms: u64) -> Self {
        Self {
            status_code: 0,
            success: false,
            error_message: Some(error_message),
            duration_ms,
            response_headers: None,
            response_body: None,
        }
    }

    pub fn with_headers(&mut self, headers: Vec<Header>) {
        self.response_headers = Some(headers);
    }

    pub fn with_body(&mut self, body: String) {
        self.response_body = Some(body);
    }
}