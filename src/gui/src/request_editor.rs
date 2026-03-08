use std::path::{Path, PathBuf};

/// Represents a request being edited.
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
                .map(|header| (header.name.clone(), header.value.clone()))
                .collect(),
            body: request.body.clone().unwrap_or_default(),
            timeout: request
                .timeout
                .map(|timeout| timeout.to_string())
                .unwrap_or_default(),
            connection_timeout: request
                .connection_timeout
                .map(|timeout| timeout.to_string())
                .unwrap_or_default(),
            depends_on: request.depends_on.clone().unwrap_or_default(),
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
            assertions: vec![],
            variables: vec![],
            timeout: Self::parse_timeout(&self.timeout),
            connection_timeout: Self::parse_timeout(&self.connection_timeout),
            depends_on: if self.depends_on.is_empty() {
                None
            } else {
                Some(self.depends_on.clone())
            },
            conditions: vec![],
            pre_delay_ms: None,
            post_delay_ms: None,
        }
    }

    fn parse_timeout(value: &str) -> Option<u64> {
        if value.is_empty() {
            None
        } else {
            value.parse().ok()
        }
    }
}

pub struct RequestEditor {
    pub(crate) requests: Vec<httprunner_core::HttpRequest>,
    pub(crate) editing_index: Option<usize>,
    pub(crate) editing_request: Option<EditableRequest>,
    pub(crate) current_file: Option<PathBuf>,
    pub(crate) has_changes: bool,
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
            self.requests =
                httprunner_core::parser::parse_http_file(path_str, None).unwrap_or_default();
            self.current_file = Some(path.to_path_buf());
            self.editing_index = None;
            self.editing_request = None;
            self.has_changes = false;
        }
    }

    pub fn load_content(&mut self, content: &str) {
        self.requests =
            httprunner_core::parser::parse_http_content(content, None).unwrap_or_default();
        self.current_file = None;
        self.editing_index = None;
        self.editing_request = None;
        self.has_changes = false;
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

    pub fn get_editing_request(&self) -> Option<&EditableRequest> {
        self.editing_request.as_ref()
    }

    pub fn current_file(&self) -> Option<&PathBuf> {
        self.current_file.as_ref()
    }

    pub fn mark_saved(&mut self) {
        self.has_changes = false;
    }

    pub fn is_editing(&self) -> bool {
        self.editing_request.is_some()
    }

    pub fn has_changes(&self) -> bool {
        self.has_changes
    }
}
