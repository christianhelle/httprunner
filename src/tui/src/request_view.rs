use crossterm::event::{KeyCode, KeyEvent};
use httprunner_lib::parser::parse_http_file;
use httprunner_lib::types::HttpRequest;
use std::path::Path;

pub struct RequestView {
    requests: Vec<HttpRequest>,
    selected_index: usize,
    run_request: bool,
    error_message: Option<String>,
}

impl RequestView {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
            selected_index: 0,
            run_request: false,
            error_message: None,
        }
    }

    pub fn load_file(&mut self, path: &Path) {
        self.error_message = None;
        if let Some(path_str) = path.to_str() {
            match parse_http_file(path_str, None) {
                Ok(requests) => {
                    self.requests = requests;
                    self.selected_index = 0;
                }
                Err(e) => {
                    self.requests.clear();
                    self.error_message = Some(format!("Failed to parse file: {}", e));
                }
            }
        } else {
            self.error_message = Some("Invalid file path".to_string());
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        self.run_request = false;

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index < self.requests.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
            }
            KeyCode::Enter => {
                self.run_request = true;
            }
            _ => {}
        }
    }

    pub fn requests(&self) -> &[HttpRequest] {
        &self.requests
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn get_selected_index(&self) -> Option<usize> {
        if self.requests.is_empty() {
            None
        } else {
            Some(self.selected_index)
        }
    }

    pub fn should_run_request(&self) -> bool {
        self.run_request
    }

    pub fn error_message(&self) -> Option<&String> {
        self.error_message.as_ref()
    }
}
