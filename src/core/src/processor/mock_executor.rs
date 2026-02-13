use crate::types::{HttpRequest, HttpResult};
use anyhow::Result;
use std::sync::{Arc, Mutex};

pub struct MockHttpExecutor {
    responses: Arc<Mutex<Vec<HttpResult>>>,
    call_count: Arc<Mutex<usize>>,
    executed_requests: Arc<Mutex<Vec<HttpRequest>>>,
}

impl MockHttpExecutor {
    pub fn new(responses: Vec<HttpResult>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
            call_count: Arc::new(Mutex::new(0)),
            executed_requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }

    #[allow(dead_code)]
    pub fn get_executed_requests(&self) -> Vec<HttpRequest> {
        self.executed_requests.lock().unwrap().clone()
    }

    pub fn execute(
        &self,
        request: &HttpRequest,
        _verbose: bool,
        _insecure: bool,
    ) -> Result<HttpResult> {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;

        self.executed_requests.lock().unwrap().push(request.clone());

        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            Ok(HttpResult {
                request_name: request.name.clone(),
                status_code: 200,
                success: true,
                error_message: None,
                duration_ms: 1,
                response_headers: None,
                response_body: Some(r#"{"status":"ok"}"#.to_string()),
                assertion_results: Vec::new(),
            })
        } else {
            Ok(responses.remove(0))
        }
    }
}
