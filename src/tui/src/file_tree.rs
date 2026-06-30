use crossterm::event::{KeyCode, KeyEvent};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

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
            let temp_files =
                httprunner_core::discovery::discover_http_file_paths(&path_clone, |count| {
                    if let Ok(mut c) = count_clone.lock() {
                        *c = count;
                    }
                });

            if let Ok(mut files) = files_clone.lock() {
                *files = temp_files;
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
            KeyCode::Up | KeyCode::Char('k') if self.selected_index > 0 => {
                self.selected_index -= 1;
            }
            KeyCode::Down | KeyCode::Char('j')
                if self.selected_index < files_len.saturating_sub(1) =>
            {
                self.selected_index += 1;
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

    pub fn with_files<T>(&self, f: impl FnOnce(&[PathBuf]) -> T) -> Option<T> {
        self.files.lock().ok().map(|files| f(&files))
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }
}
