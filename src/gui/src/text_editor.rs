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
            // A request is selected if:
            // 1. The cursor is within the request (start_pos == end_pos and within range)
            // 2. The selection overlaps with the request
            if (start_pos >= *req_start && start_pos < *req_end)
                || (end_pos > *req_start && end_pos <= *req_end)
                || (start_pos <= *req_start && end_pos >= *req_end)
            {
                selected_indices.push(idx);
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
        let mut boundaries = Vec::new();
        let lines: Vec<&str> = self.content.lines().collect();
        let mut char_pos = 0;
        let mut current_request_start: Option<usize> = None;

        for line in lines.iter() {
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

            // Add line length + newline character to char_pos
            char_pos += line.len() + 1; // +1 for the newline
        }

        // Handle the last request
        if let Some(start) = current_request_start {
            boundaries.push((start, char_pos));
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
