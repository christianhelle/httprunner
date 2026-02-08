use iced::{
    widget::{column, scrollable, text, Column},
    Color, Element, Length,
};
use serde::{Deserialize, Serialize};

use crate::app::Message;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ExecutionResult {
    Success {
        method: String,
        url: String,
        status: u16,
        duration_ms: u64,
        request_body: Option<String>,
        response_body: String,
        assertion_results: Vec<httprunner_core::types::AssertionResult>,
    },
    Failure {
        method: String,
        url: String,
        error: String,
    },
    Running {
        message: String,
    },
}

pub struct ResultsView {
    results: Vec<ExecutionResult>,
    compact_mode: bool,
}

impl ResultsView {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            compact_mode: true,
        }
    }

    pub fn get_all_results(&self) -> Vec<ExecutionResult> {
        self.results
            .iter()
            .filter(|r| !matches!(r, ExecutionResult::Running { .. }))
            .cloned()
            .collect()
    }

    pub fn restore_results(&mut self, saved_results: Vec<ExecutionResult>) {
        self.results = saved_results;
    }

    pub fn set_results(&mut self, results: Vec<httprunner_core::HttpResult>) {
        self.results.clear();
        for result in results {
            if result.success {
                self.results.push(ExecutionResult::Success {
                    method: result.request_name.clone().unwrap_or_else(|| "GET".to_string()),
                    url: String::new(), // URL not available in HttpResult
                    status: result.status_code,
                    duration_ms: result.duration_ms,
                    request_body: None,
                    response_body: result.response_body.clone().unwrap_or_default(),
                    assertion_results: result.assertion_results.clone(),
                });
            } else if let Some(error) = result.error_message {
                self.results.push(ExecutionResult::Failure {
                    method: result.request_name.clone().unwrap_or_else(|| "GET".to_string()),
                    url: String::new(),
                    error: error.clone(),
                });
            }
        }
    }

    pub fn set_compact_mode(&mut self, compact: bool) {
        self.compact_mode = compact;
    }

    pub fn is_compact_mode(&self) -> bool {
        self.compact_mode
    }

    pub fn toggle_compact_mode(&mut self) {
        self.compact_mode = !self.compact_mode;
    }

    pub fn view(&self) -> Element<Message> {
        let mut col = Column::new().spacing(10).padding(10);

        col = col.push(text("Results").size(20));

        if self.results.is_empty() {
            col = col.push(text("No results yet. Run requests to see results."));
        } else {
            for (idx, result) in self.results.iter().enumerate() {
                match result {
                    ExecutionResult::Success {
                        method,
                        url,
                        status,
                        duration_ms,
                        request_body,
                        response_body,
                        assertion_results,
                    } => {
                        col = col.push(text(format!("✓ {} {}", method, url)));
                        col = col.push(text(format!("Status: {} | Duration: {}ms", status, duration_ms)));

                        if !self.compact_mode {
                            if let Some(req_body) = request_body {
                                col = col.push(text("Request Body:"));
                                col = col.push(text(req_body).size(12));
                            }
                            col = col.push(text("Response Body:"));
                            col = col.push(text(response_body).size(12));

                            if !assertion_results.is_empty() {
                                col = col.push(text("Assertions:"));
                                for assertion in assertion_results {
                                    let result_text = if assertion.passed {
                                        format!("  ✓ {:?}: {}", assertion.assertion.assertion_type, assertion.assertion.expected_value)
                                    } else {
                                        format!("  ✗ {:?}: {} ({})", 
                                            assertion.assertion.assertion_type, 
                                            assertion.assertion.expected_value,
                                            assertion.error_message.as_ref().unwrap_or(&"failed".to_string()))
                                    };
                                    col = col.push(text(result_text));
                                }
                            }
                        }
                    }
                    ExecutionResult::Failure { method, url, error } => {
                        col = col.push(text(format!("✗ {} {}", method, url)));
                        col = col.push(text(format!("Error: {}", error)));
                    }
                    ExecutionResult::Running { message } => {
                        col = col.push(text(format!("⏳ {}", message)));
                    }
                }

                col = col.push(text("───────────────"));
            }
        }

        scrollable(col).into()
    }
}

impl Default for ResultsView {
    fn default() -> Self {
        Self::new()
    }
}
