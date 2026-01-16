use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Action to perform after showing the text editor UI
pub enum TextEditorAction {
    /// Run a specific request by index
    RunRequest(usize),
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

        // Use egui_code_editor for syntax highlighting
        CodeEditor::default()
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

        // Add buttons below the editor
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("💾 Save").clicked() {
                if let Err(e) = self.save_to_file() {
                    eprintln!("Failed to save file: {}", e);
                }
            }

            // Note: Cursor position tracking not yet implemented
            // For now, this runs the first request in the file
            if ui.button("▶ Run First Request").clicked() {
                if let Some(request_index) = self.find_first_request() {
                    action = TextEditorAction::RunRequest(request_index);
                }
            }

            if self.has_changes {
                ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "● Unsaved changes");
            }
        });

        action
    }

    /// Find the first request in the file
    /// TODO: Implement proper cursor position tracking to find request at cursor
    fn find_first_request(&self) -> Option<usize> {
        // Parse current editor content by writing to a temporary file
        // This ensures we parse what the user sees, not the saved file
        use std::io::Write;
        
        // Create a secure temporary file with automatic cleanup
        let mut temp_file = match tempfile::NamedTempFile::new() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to create temporary file for parsing: {}", e);
                return None;
            }
        };
        
        // Write current content to the temporary file
        if let Err(e) = temp_file.write_all(self.content.as_bytes()) {
            eprintln!("Failed to write to temporary file: {}", e);
            return None;
        }
        
        // Get the path and parse
        let temp_path = temp_file.path();
        if let Some(temp_path_str) = temp_path.to_str() {
            if let Ok(requests) = 
                httprunner_lib::parser::parse_http_file(temp_path_str, None)
            {
                if !requests.is_empty() {
                    return Some(0);
                }
            }
        }
        
        // Temporary file is automatically cleaned up when temp_file goes out of scope
        None
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
            "GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS", "CONNECT", "TRACE",
            // Common headers
            "Content-Type", "Authorization", "Accept", "User-Agent", "Host", "Connection",
            "Cache-Control", "Cookie", "Set-Cookie",
            // httprunner specific
            "ASSERT", "VAR",
        ]
        .into_iter()
        .collect(),
        types: vec![
            "http", "https", "HTTP/1.1", "HTTP/2", "HTTP/3",
            "application/json", "application/xml", "text/html", "text/plain",
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
