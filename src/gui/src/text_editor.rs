use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

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
    #[cfg(not(target_arch = "wasm32"))]
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

    /// Set content directly (for WASM where file loading doesn't work)
    #[cfg(target_arch = "wasm32")]
    pub fn load_file(&mut self, _path: &Path) {
        // Try to load from LocalStorage first
        use web_sys::window;

        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(saved_content)) = storage.get_item("httprunner_editor_content") {
                    self.content = saved_content;
                    self.has_changes = false;
                }
            }
        }
    }

    /// Set content programmatically
    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.has_changes = true;
    }

    /// Get current content
    pub fn get_content(&self) -> &str {
        &self.content
    }

    /// Save the current content to the loaded file
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_to_file(&mut self) -> anyhow::Result<()> {
        if let Some(path) = &self.current_file {
            std::fs::write(path, &self.content)?;
            self.has_changes = false;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No file loaded"))
        }
    }

    /// Save editor content to LocalStorage on WASM
    #[cfg(target_arch = "wasm32")]
    pub fn save_to_file(&mut self) -> anyhow::Result<()> {
        use web_sys::window;

        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                // Save editor content to LocalStorage under a specific key
                storage
                    .set_item("httprunner_editor_content", &self.content)
                    .map_err(|e| anyhow::anyhow!("Failed to save to LocalStorage: {:?}", e))?;
                self.has_changes = false;
                Ok(())
            } else {
                Err(anyhow::anyhow!("LocalStorage is not available"))
            }
        } else {
            Err(anyhow::anyhow!("Window object is not available"))
        }
    }

    /// Display the text editor UI and handle user interactions
    pub fn show(&mut self, ui: &mut egui::Ui, file: &Option<PathBuf>) {
        #[cfg(not(target_arch = "wasm32"))]
        if file.is_none() {
            ui.label("No file selected. Select a .http file from the left panel.");
            return;
        }

        #[cfg(target_arch = "wasm32")]
        if file.is_none() && self.content.is_empty() {
            ui.vertical(|ui| {
                ui.heading("✏️ Paste your HTTP requests here");
                ui.add_space(10.0);
                ui.label("You can paste the contents of an .http file below:");
                ui.label("Example:");
                ui.monospace("GET https://api.example.com/users");
                ui.monospace("Accept: application/json");
                ui.add_space(10.0);
            });
        }

        let original_content = self.content.clone();

        CodeEditor::default()
            .with_rows(30)
            .with_fontsize(14.0)
            .with_theme(ColorTheme::GITHUB_DARK)
            .with_syntax(http_syntax())
            .with_numlines(true)
            .show(ui, &mut self.content);

        if self.content != original_content {
            self.has_changes = true;
        }

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("💾 Save").clicked()
                && let Err(e) = self.save_to_file()
            {
                eprintln!("Failed to save file: {}", e);
            }

            if self.has_changes {
                ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "● Unsaved changes");
            }
        });
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
