use crossterm::event::{KeyCode, KeyEvent};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use walkdir::WalkDir;

pub struct FileTree {
    root: PathBuf,
    files: Arc<Mutex<Vec<PathBuf>>>,
    selected_index: usize,
    is_discovering: Arc<Mutex<bool>>,
    discovered_count: Arc<Mutex<usize>>,
}

impl FileTree {
    pub fn new(root: PathBuf) -> Self {
        let files = Arc::new(Mutex::new(Vec::new()));
        let is_discovering = Arc::new(Mutex::new(true));
        let discovered_count = Arc::new(Mutex::new(0usize));

        // Clone for the background thread
        let files_clone = Arc::clone(&files);
        let discovering_clone = Arc::clone(&is_discovering);
        let count_clone = Arc::clone(&discovered_count);
        let path_clone = root.clone();

        // Start async discovery in background thread
        thread::spawn(move || {
            let mut temp_files = Vec::new();

            for entry in WalkDir::new(&path_clone)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("http") {
                    let file_path = path.to_path_buf();
                    temp_files.push(file_path.clone());

                    // Update shared state incrementally
                    if let Ok(mut files) = files_clone.lock() {
                        files.push(file_path);
                        files.sort();
                    }
                    if let Ok(mut count) = count_clone.lock() {
                        *count = temp_files.len();
                    }
                }
            }

            // Mark discovery as complete
            if let Ok(mut discovering) = discovering_clone.lock() {
                *discovering = false;
            }
        });

        Self {
            root,
            files,
            selected_index: 0,
            is_discovering,
            discovered_count,
        }
    }

    pub fn is_discovering(&self) -> bool {
        self.is_discovering
            .lock()
            .map(|guard| *guard)
            .unwrap_or(false)
    }

    pub fn discovered_count(&self) -> usize {
        self.discovered_count
            .lock()
            .map(|guard| *guard)
            .unwrap_or(0)
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        let files_len = self.files.lock().map(|f| f.len()).unwrap_or(0);

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index < files_len.saturating_sub(1) {
                    self.selected_index += 1;
                }
            }
            KeyCode::Home => {
                self.selected_index = 0;
            }
            KeyCode::End => {
                self.selected_index = files_len.saturating_sub(1);
            }
            _ => {}
        }
    }

    pub fn get_selected_file(&self) -> Option<PathBuf> {
        self.files
            .lock()
            .ok()
            .and_then(|files| files.get(self.selected_index).cloned())
    }

    pub fn files(&self) -> Vec<PathBuf> {
        self.files
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }
}
