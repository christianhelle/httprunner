use std::path::{Path, PathBuf};

/// Represents a request being edited
#[derive(Clone, Debug)]
pub struct EditableRequest {
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub timeout: String,
    pub connection_timeout: String,
    pub depends_on: String,
    pub assertions: Vec<httprunner_core::types::Assertion>,
    pub variables: Vec<httprunner_core::types::Variable>,
    pub conditions: Vec<httprunner_core::types::Condition>,
    pub pre_delay_ms: Option<u64>,
    pub post_delay_ms: Option<u64>,
}

impl Default for EditableRequest {
    fn default() -> Self {
        Self {
            name: String::new(),
            method: "GET".to_string(),
            url: String::new(),
            headers: vec![],
            body: String::new(),
            timeout: String::new(),
            connection_timeout: String::new(),
            depends_on: String::new(),
            assertions: vec![],
            variables: vec![],
            conditions: vec![],
            pre_delay_ms: None,
            post_delay_ms: None,
        }
    }
}

impl From<&httprunner_core::HttpRequest> for EditableRequest {
    fn from(request: &httprunner_core::HttpRequest) -> Self {
        Self {
            name: request.name.clone().unwrap_or_default(),
            method: request.method.clone(),
            url: request.url.clone(),
            headers: request
                .headers
                .iter()
                .map(|h| (h.name.clone(), h.value.clone()))
                .collect(),
            body: request.body.clone().unwrap_or_default(),
            timeout: request.timeout.map(|t| t.to_string()).unwrap_or_default(),
            connection_timeout: request
                .connection_timeout
                .map(|t| t.to_string())
                .unwrap_or_default(),
            depends_on: request.depends_on.clone().unwrap_or_default(),
            assertions: request.assertions.clone(),
            variables: request.variables.clone(),
            conditions: request.conditions.clone(),
            pre_delay_ms: request.pre_delay_ms,
            post_delay_ms: request.post_delay_ms,
        }
    }
}

impl EditableRequest {
    pub fn to_http_request(&self) -> httprunner_core::HttpRequest {
        use httprunner_core::types::Header;

        httprunner_core::HttpRequest {
            name: if self.name.is_empty() {
                None
            } else {
                Some(self.name.clone())
            },
            method: self.method.clone(),
            url: self.url.clone(),
            headers: self
                .headers
                .iter()
                .map(|(name, value)| Header {
                    name: name.clone(),
                    value: value.clone(),
                })
                .collect(),
            body: if self.body.is_empty() {
                None
            } else {
                Some(self.body.clone())
            },
            assertions: self.assertions.clone(),
            variables: self.variables.clone(),
            timeout: Self::parse_timeout(&self.timeout),
            connection_timeout: Self::parse_timeout(&self.connection_timeout),
            depends_on: if self.depends_on.is_empty() {
                None
            } else {
                Some(self.depends_on.clone())
            },
            conditions: self.conditions.clone(),
            pre_delay_ms: self.pre_delay_ms,
            post_delay_ms: self.post_delay_ms,
        }
    }

    /// Parse timeout value. Returns None if empty or invalid.
    /// This is acceptable for GUI usage - invalid values are simply ignored.
    fn parse_timeout(value: &str) -> Option<u64> {
        if value.is_empty() {
            None
        } else {
            value.parse().ok()
        }
    }
}

pub struct RequestEditor {
    requests: Vec<httprunner_core::HttpRequest>,
    editing_index: Option<usize>,
    editing_request: Option<EditableRequest>,
    current_file: Option<PathBuf>,
    has_changes: bool,
}

impl RequestEditor {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
            editing_index: None,
            editing_request: None,
            current_file: None,
            has_changes: false,
        }
    }

    pub fn load_file(&mut self, path: &Path) {
        if let Some(path_str) = path.to_str() {
            let parsed_requests =
                httprunner_core::parser::parse_http_file(path_str, None).unwrap_or_default();
            self.requests = parsed_requests;
            self.current_file = Some(path.to_path_buf());
            self.editing_index = None;
            self.editing_request = None;
            self.has_changes = false;
        }
    }

    pub fn start_editing(&mut self, index: usize) {
        if let Some(request) = self.requests.get(index) {
            self.editing_request = Some(EditableRequest::from(request));
            self.editing_index = Some(index);
        }
    }

    pub fn start_new_request(&mut self) {
        self.editing_request = Some(EditableRequest::default());
        self.editing_index = None;
    }

    pub fn cancel_editing(&mut self) {
        self.editing_request = None;
        self.editing_index = None;
    }

    pub fn save_current_edit(&mut self) -> bool {
        if let Some(editable) = &self.editing_request {
            let new_request = editable.to_http_request();

            if let Some(index) = self.editing_index {
                if index < self.requests.len() {
                    self.requests[index] = new_request;
                    self.has_changes = true;
                }
            } else {
                self.requests.push(new_request);
                self.has_changes = true;
            }

            self.editing_request = None;
            self.editing_index = None;
            return true;
        }
        false
    }

    pub fn delete_request(&mut self, index: usize) {
        if index < self.requests.len() {
            self.requests.remove(index);
            self.has_changes = true;

            // If we were editing this request, cancel the edit
            if self.editing_index == Some(index) {
                self.editing_request = None;
                self.editing_index = None;
            }
        }
    }

    pub fn save_to_file(&mut self) -> anyhow::Result<()> {
        if let Some(path) = &self.current_file {
            httprunner_core::serializer::write_http_file(path, &self.requests)?;
            self.has_changes = false;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No file loaded"))
        }
    }

    pub fn get_requests(&self) -> &[httprunner_core::HttpRequest] {
        &self.requests
    }

    pub fn get_editing_request_mut(&mut self) -> Option<&mut EditableRequest> {
        self.editing_request.as_mut()
    }

    pub fn is_editing(&self) -> bool {
        self.editing_request.is_some()
    }

    pub fn has_changes(&self) -> bool {
        self.has_changes
    }
}

#[cfg(test)]
mod tests {
    use super::EditableRequest;
    use httprunner_core::types::{
        Assertion, AssertionType, Condition, ConditionType, Header, HttpRequest, Variable,
    };

    #[test]
    fn test_editable_request_preserves_hidden_request_semantics() {
        let request = HttpRequest {
            name: Some("login".to_string()),
            method: "POST".to_string(),
            url: "https://example.com/login".to_string(),
            headers: vec![Header {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            }],
            body: Some(r#"{"username":"demo"}"#.to_string()),
            assertions: vec![Assertion {
                assertion_type: AssertionType::Status,
                expected_value: "200".to_string(),
            }],
            variables: vec![Variable {
                name: "token".to_string(),
                value: "{{login.response.body.$.token}}".to_string(),
            }],
            timeout: Some(5000),
            connection_timeout: Some(1000),
            depends_on: Some("setup".to_string()),
            conditions: vec![Condition {
                request_name: "setup".to_string(),
                condition_type: ConditionType::BodyJsonPath("$.ready".to_string()),
                expected_value: "true".to_string(),
                negate: false,
            }],
            pre_delay_ms: Some(100),
            post_delay_ms: Some(200),
        };

        let editable = EditableRequest::from(&request);
        let round_tripped = editable.to_http_request();

        assert_eq!(round_tripped.name, request.name);
        assert_eq!(round_tripped.method, request.method);
        assert_eq!(round_tripped.url, request.url);
        assert_eq!(round_tripped.headers.len(), request.headers.len());
        assert_eq!(round_tripped.body, request.body);
        assert_eq!(round_tripped.timeout, request.timeout);
        assert_eq!(round_tripped.connection_timeout, request.connection_timeout);
        assert_eq!(round_tripped.depends_on, request.depends_on);
        assert_eq!(round_tripped.pre_delay_ms, request.pre_delay_ms);
        assert_eq!(round_tripped.post_delay_ms, request.post_delay_ms);
        assert_eq!(round_tripped.assertions.len(), 1);
        assert!(matches!(
            round_tripped.assertions[0].assertion_type,
            AssertionType::Status
        ));
        assert_eq!(round_tripped.assertions[0].expected_value, "200");
        assert_eq!(round_tripped.variables.len(), 1);
        assert_eq!(round_tripped.variables[0].name, "token");
        assert_eq!(
            round_tripped.variables[0].value,
            "{{login.response.body.$.token}}"
        );
        assert_eq!(round_tripped.conditions.len(), 1);
        assert_eq!(round_tripped.conditions[0].request_name, "setup");
        assert!(matches!(
            round_tripped.conditions[0].condition_type,
            ConditionType::BodyJsonPath(_)
        ));
        assert_eq!(round_tripped.conditions[0].expected_value, "true");
    }
}
