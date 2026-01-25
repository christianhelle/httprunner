use crossterm::event::{KeyCode, KeyEvent};
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct FileTree {
    root: PathBuf,
    files: Vec<PathBuf>,
    selected_index: usize,
}

impl FileTree {
    pub fn new(root: PathBuf) -> Self {
        let mut files = Vec::new();

        // Scan for .http files
        for entry in WalkDir::new(&root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("http") {
                files.push(path.to_path_buf());
            }
        }

        files.sort();

        Self {
            root,
            files,
            selected_index: 0,
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index < self.files.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
            }
            KeyCode::Home => {
                self.selected_index = 0;
            }
            KeyCode::End => {
                self.selected_index = self.files.len().saturating_sub(1);
            }
            _ => {}
        }
    }

    pub fn get_selected_file(&self) -> Option<PathBuf> {
        self.files.get(self.selected_index).cloned()
    }

    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }
}
