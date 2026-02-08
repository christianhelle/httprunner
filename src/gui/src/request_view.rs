use iced::{
    widget::{column, scrollable, text, Column},
    Element, Length,
};
use std::path::Path;

use crate::app::Message;

pub struct RequestView {
    requests: Vec<httprunner_core::HttpRequest>,
}

impl RequestView {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    pub fn load_file(&mut self, path: &Path) {
        if let Some(path_str) = path.to_str() {
            match httprunner_core::parser::parse_http_file(path_str, None) {
                Ok(requests) => {
                    self.requests = requests;
                }
                Err(e) => {
                    eprintln!("Failed to parse HTTP file: {}", e);
                    self.requests = Vec::new();
                }
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut col = Column::new().spacing(10).padding(10);

        col = col.push(text("Request Details").size(20));

        if self.requests.is_empty() {
            col = col.push(text("No requests found in this file."));
        } else {
            for (idx, request) in self.requests.iter().enumerate() {
                let header = if let Some(name) = &request.name {
                    format!("{} - {} {}", idx + 1, request.method, name)
                } else {
                    format!("{} - {} {}", idx + 1, request.method, request.url)
                };

                col = col.push(text(header).size(16));
                col = col.push(text(format!("Method: {}", request.method)));
                col = col.push(text(format!("URL: {}", request.url)));

                if !request.headers.is_empty() {
                    col = col.push(text("Headers:"));
                    for header in &request.headers {
                        col = col.push(text(format!("  {}: {}", header.name, header.value)));
                    }
                }

                if let Some(body) = &request.body {
                    col = col.push(text("Body:"));
                    col = col.push(text(body).size(12));
                }

                col = col.push(text("───────────────"));
            }
        }

        scrollable(col).into()
    }
}

impl Default for RequestView {
    fn default() -> Self {
        Self::new()
    }
}
