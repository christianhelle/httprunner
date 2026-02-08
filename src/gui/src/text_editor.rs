use iced::{
    widget::{scrollable, text, text_editor, Column},
    Element, Length,
};
use std::path::{Path, PathBuf};

use crate::app::Message;

/// Text editor component for editing .http files
pub struct TextEditor {
    /// Text editor content state
    content: text_editor::Content,
    /// Path to the currently loaded file
    current_file: Option<PathBuf>,
    /// Whether the content has been modified since last save
    has_changes: bool,
}

impl TextEditor {
    /// Create a new text editor instance
    pub fn new() -> Self {
        Self {
            content: text_editor::Content::new(),
            current_file: None,
            has_changes: false,
        }
    }

    /// Load a .http file into the editor
    pub fn load_file(&mut self, path: &Path) {
        match std::fs::read_to_string(path) {
            Ok(file_content) => {
                self.content = text_editor::Content::with_text(&file_content);
                self.current_file = Some(path.to_path_buf());
                self.has_changes = false;
            }
            Err(e) => {
                eprintln!("Failed to load file {}: {}", path.display(), e);
            }
        }
    }

    /// Set content programmatically
    pub fn set_content(&mut self, new_content: String) {
        self.content = text_editor::Content::with_text(&new_content);
        self.has_changes = true;
    }

    /// Get current content
    #[allow(dead_code)]
    pub fn get_content(&self) -> String {
        self.content.text()
    }

    /// Save the current content to the loaded file
    pub fn save_to_file(&mut self) -> anyhow::Result<()> {
        if let Some(path) = &self.current_file {
            std::fs::write(path, self.content.text())?;
            self.has_changes = false;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No file loaded"))
        }
    }

    /// Display the text editor UI
    pub fn view(&self) -> Element<'_, Message> {
        let mut col = Column::new().spacing(5).padding(10);

        if let Some(path) = &self.current_file {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            
            let status = if self.has_changes { " (modified)" } else { "" };
            col = col.push(text(format!("📝 {}{}", file_name, status)));
        } else {
            col = col.push(text("No file loaded"));
        }

        let editor = text_editor(&self.content)
            .height(Length::Fill);

        col = col.push(editor);

        scrollable(col).into()
    }
}

impl Default for TextEditor {
    fn default() -> Self {
        Self::new()
    }
}
