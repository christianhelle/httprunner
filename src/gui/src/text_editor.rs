use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Action to perform after showing the text editor UI
pub enum TextEditorAction {
    /// Run specific request(s) by index/indices
    RunRequests(Vec<usize>),
    /// No action required
    None,
}

/// Text editor component for editing .http files with syntax highlighting
pub struct TextEditor {
    /// Current file content
    content: String,
    /// Path to the currently loaded file
    current_file: Option<PathBuf>,
    /// Whether the content has been modified since last save
    has_changes: bool,
}

impl TextEditor {
    /// Create a new text editor instance
    pub fn new() -> Self {
        Self {
            content: String::new(),
            current_file: None,
            has_changes: false,
        }
    }

    /// Load a .http file into the editor
    pub fn load_file(&mut self, path: &Path) {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                self.content = content;
                self.current_file = Some(path.to_path_buf());
                self.has_changes = false;
            }
            Err(e) => {
                eprintln!("Failed to load file {}: {}", path.display(), e);
                // Keep existing content and file state on error
            }
        }
    }

    /// Save the current content to the loaded file
    pub fn save_to_file(&mut self) -> anyhow::Result<()> {
        if let Some(path) = &self.current_file {
            std::fs::write(path, &self.content)?;
            self.has_changes = false;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No file loaded"))
        }
    }

    /// Display the text editor UI and handle user interactions
    /// Returns an action to be performed by the parent component
    pub fn show(&mut self, ui: &mut egui::Ui, file: &Option<PathBuf>) -> TextEditorAction {
        let mut action = TextEditorAction::None;

        if file.is_none() {
            ui.label("No file selected. Select a .http file from the left panel.");
            return action;
        }

        // Store original content to detect changes
        let original_content = self.content.clone();

        // Use egui_code_editor for syntax highlighting and capture the output
        let output = CodeEditor::default()
            .with_rows(30)
            .with_fontsize(14.0)
            .with_theme(ColorTheme::GITHUB_DARK)
            .with_syntax(http_syntax())
            .with_numlines(true)
            .show(ui, &mut self.content);

        // Detect changes
        if self.content != original_content {
            self.has_changes = true;
        }

        // Extract cursor position or selection range
        let cursor_range = output.cursor_range;

        // Add buttons below the editor
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("💾 Save").clicked()
                && let Err(e) = self.save_to_file()
            {
                eprintln!("Failed to save file: {}", e);
            }

            // Run selected request(s) based on cursor position or text selection
            if ui.button("▶ Run Selected Request(s)").clicked()
                && let Some(request_indices) = self.find_selected_requests(cursor_range)
                && !request_indices.is_empty()
            {
                action = TextEditorAction::RunRequests(request_indices);
            }

            if self.has_changes {
                ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "● Unsaved changes");
            }
        });

        action
    }

    /// Find the request(s) at the cursor position or within the selection range
    fn find_selected_requests(
        &self,
        cursor_range: Option<egui::text::CCursorRange>,
    ) -> Option<Vec<usize>> {
        // Parse current editor content to get request boundaries
        let request_boundaries = self.get_request_boundaries()?;

        if request_boundaries.is_empty() {
            return None;
        }

        // If no cursor position is available, return the first request
        let Some(cursor_range) = cursor_range else {
            return Some(vec![0]);
        };

        // Get the character range (start and end positions)
        let char_range = cursor_range.as_sorted_char_range();
        let start_pos = char_range.start;
        let end_pos = char_range.end;

        // Find all requests that intersect with the cursor/selection range
        let mut selected_indices = Vec::new();

        for (idx, (req_start, req_end)) in request_boundaries.iter().enumerate() {
            // Check if the cursor/selection overlaps with this request
            let is_cursor = start_pos == end_pos;
            
            if is_cursor {
                // For cursor position, check if it's within the request bounds (inclusive start, exclusive end)
                if start_pos >= *req_start && start_pos < *req_end {
                    selected_indices.push(idx);
                }
            } else {
                // For selection, use standard range overlap: two ranges overlap if:
                // start1 < end2 AND end1 > start2
                if start_pos < *req_end && end_pos > *req_start {
                    selected_indices.push(idx);
                }
            }
        }

        if selected_indices.is_empty() {
            // If cursor is after all requests, select the last one
            // If cursor is before all requests, select the first one
            if start_pos >= request_boundaries.last().unwrap().1 {
                Some(vec![request_boundaries.len() - 1])
            } else {
                Some(vec![0])
            }
        } else {
            Some(selected_indices)
        }
    }

    /// Get the character position boundaries for each request in the file
    /// Returns a vector of (start_pos, end_pos) tuples for each request
    fn get_request_boundaries(&self) -> Option<Vec<(usize, usize)>> {
        // Parse current editor content by writing to a temporary file
        use std::io::Write;

        let mut temp_file = match tempfile::NamedTempFile::new() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to create temporary file for parsing: {}", e);
                return None;
            }
        };

        if let Err(e) = temp_file.write_all(self.content.as_bytes()) {
            eprintln!("Failed to write to temporary file: {}", e);
            return None;
        }

        let temp_path = temp_file.path();
        let Some(temp_path_str) = temp_path.to_str() else {
            return None;
        };

        let Ok(requests) = httprunner_lib::parser::parse_http_file(temp_path_str, None) else {
            return None;
        };

        if requests.is_empty() {
            return None;
        }

        // Now we need to find the boundaries of each request in the content
        // We'll search for HTTP method lines to determine request boundaries
        // Note: We use byte positions in the original content string to handle
        // different line endings (LF vs CRLF) correctly
        let mut boundaries = Vec::new();
        let mut current_request_start: Option<usize> = None;
        let mut char_pos = 0;

        for line in self.content.lines() {
            let trimmed = line.trim();

            // Check if this is an HTTP request line
            if is_http_method_line(trimmed) {
                // If we already had a request start, save the previous request boundary
                if let Some(start) = current_request_start {
                    // The end of the previous request is just before this line
                    boundaries.push((start, char_pos));
                }

                // Start a new request
                current_request_start = Some(char_pos);
            }

            // Find the actual position of the next line in the content
            // This handles both LF (\n) and CRLF (\r\n) line endings correctly
            let line_start = char_pos;
            let line_end = line_start + line.len();
            
            // Find where the next line starts by searching for the line ending
            if let Some(next_line_start) = self.content[line_end..].find(|c| c == '\n') {
                // Found a newline, next line starts after it
                char_pos = line_end + next_line_start + 1;
            } else {
                // No more newlines, this is the last line
                char_pos = self.content.len();
            }
        }

        // Handle the last request - use content length as the end boundary
        if let Some(start) = current_request_start {
            boundaries.push((start, self.content.len()));
        }

        if boundaries.is_empty() {
            None
        } else {
            Some(boundaries)
        }
    }

    /// Check if the editor has unsaved changes
    pub fn has_changes(&self) -> bool {
        self.has_changes
    }
}

/// Custom syntax highlighting for .http files
fn http_syntax() -> Syntax {
    Syntax {
        language: "HTTP",
        case_sensitive: true,
        comment: "#",
        comment_multiline: ["###", "###"],
        hyperlinks: BTreeSet::new(), // Empty set for now
        keywords: vec![
            // HTTP Methods
            "GET",
            "POST",
            "PUT",
            "DELETE",
            "PATCH",
            "HEAD",
            "OPTIONS",
            "CONNECT",
            "TRACE",
            // Common headers
            "Content-Type",
            "Authorization",
            "Accept",
            "User-Agent",
            "Host",
            "Connection",
            "Cache-Control",
            "Cookie",
            "Set-Cookie",
            // httprunner specific
            "ASSERT",
            "VAR",
        ]
        .into_iter()
        .collect(),
        types: vec![
            "http",
            "https",
            "HTTP/1.1",
            "HTTP/2",
            "HTTP/3",
            "application/json",
            "application/xml",
            "text/html",
            "text/plain",
        ]
        .into_iter()
        .collect(),
        special: vec![
            // Status codes
            "200", "201", "204", "301", "302", "400", "401", "403", "404", "500", "502", "503",
        ]
        .into_iter()
        .collect(),
    }
}

/// Check if a line is an HTTP method line (starts with a HTTP method and has a URL)
fn is_http_method_line(line: &str) -> bool {
    let trimmed = line.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    if parts.len() < 2 {
        return false;
    }

    // Check if the first part is a valid HTTP method
    matches!(
        parts[0],
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS" | "CONNECT" | "TRACE"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_http_method_line() {
        // Valid HTTP method lines
        assert!(is_http_method_line("GET https://example.com"));
        assert!(is_http_method_line("POST https://api.example.com/data"));
        assert!(is_http_method_line("  PUT https://example.com  "));
        assert!(is_http_method_line("DELETE /api/users/1"));

        // Invalid lines
        assert!(!is_http_method_line("# Comment"));
        assert!(!is_http_method_line("Content-Type: application/json"));
        assert!(!is_http_method_line("GET"));
        assert!(!is_http_method_line(""));
        assert!(!is_http_method_line("INVALID https://example.com"));
    }

    #[test]
    fn test_get_request_boundaries() {
        let mut editor = TextEditor::new();
        editor.content = r#"# Simple test file
GET https://httpbin.org/status/200

# Test a not found error
GET https://httpbin.org/status/404

POST https://example.com/api
Content-Type: application/json

{"test": "data"}
"#
        .to_string();

        let boundaries = editor.get_request_boundaries();
        assert!(boundaries.is_some());

        let boundaries = boundaries.unwrap();
        
        // Should find 3 requests
        assert_eq!(boundaries.len(), 3);

        // First request starts at "GET https://httpbin.org/status/200"
        assert!(boundaries[0].0 < boundaries[0].1);
        // Second request starts after first request ends
        assert!(boundaries[1].0 >= boundaries[0].1, 
            "Second request start ({}) should be >= first request end ({})", 
            boundaries[1].0, boundaries[0].1);
        assert!(boundaries[1].0 < boundaries[1].1);
        // Third request starts after second request ends
        assert!(boundaries[2].0 >= boundaries[1].1,
            "Third request start ({}) should be >= second request end ({})", 
            boundaries[2].0, boundaries[1].1);
    }

    #[test]
    fn test_get_request_boundaries_no_trailing_newline() {
        let mut editor = TextEditor::new();
        // Content without trailing newline
        editor.content = "GET https://httpbin.org/status/200\nGET https://httpbin.org/status/404".to_string();

        let boundaries = editor.get_request_boundaries();
        assert!(boundaries.is_some());

        let boundaries = boundaries.unwrap();
        assert_eq!(boundaries.len(), 2);

        // Verify the last boundary doesn't exceed content length
        assert_eq!(boundaries[1].1, editor.content.len());
        assert!(boundaries[1].0 < boundaries[1].1);
    }
}
