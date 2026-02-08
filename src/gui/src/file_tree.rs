use iced::{
    widget::{button, column, scrollable, text, Column},
    Element, Length,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use walkdir::WalkDir;

use crate::app::Message;

pub struct FileTree {
    root_path: PathBuf,
    http_files: Arc<Mutex<Vec<PathBuf>>>,
    expanded_dirs: std::collections::HashSet<PathBuf>,
    is_discovering: Arc<Mutex<bool>>,
    discovered_count: Arc<Mutex<usize>>,
}

impl FileTree {
    pub fn new(root_path: PathBuf) -> Self {
        let http_files = Arc::new(Mutex::new(Vec::new()));
        let is_discovering = Arc::new(Mutex::new(true));
        let discovered_count = Arc::new(Mutex::new(0usize));

        // Clone for the background thread
        let files_clone = Arc::clone(&http_files);
        let discovering_clone = Arc::clone(&is_discovering);
        let count_clone = Arc::clone(&discovered_count);
        let path_clone = root_path.clone();

        // Start async discovery in background thread
        thread::spawn(move || {
            let mut temp_files = Vec::new();

            for entry in WalkDir::new(&path_clone)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "http" {
                            let file_path = entry.path().to_path_buf();
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
                }
            }

            // Mark discovery as complete
            if let Ok(mut discovering) = discovering_clone.lock() {
                *discovering = false;
            }
        });

        Self {
            root_path,
            http_files,
            expanded_dirs: std::collections::HashSet::new(),
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

    pub fn view(&self) -> Element<Message> {
        let mut content = Column::new().spacing(5).padding(10);

        // Show discovery status
        if self.is_discovering() {
            content = content.push(text(format!(
                "Discovering .http files... ({})",
                self.discovered_count()
            )));
        }

        // Get a snapshot of current files
        let http_files = self
            .http_files
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default();

        if http_files.is_empty() && !self.is_discovering() {
            content = content
                .push(text("No .http files found."))
                .push(text("Use Open Folder to choose a directory."));
        } else {
            // Group files by directory
            let mut dir_files: std::collections::BTreeMap<Option<PathBuf>, Vec<PathBuf>> =
                std::collections::BTreeMap::new();

            for file in &http_files {
                let parent = file.parent().map(|p| p.to_path_buf());
                dir_files.entry(parent).or_default().push(file.clone());
            }

            // Show files organized by directory
            for (dir, files) in dir_files {
                if let Some(dir_path) = &dir {
                    let dir_name = dir_path
                        .strip_prefix(&self.root_path)
                        .unwrap_or(dir_path)
                        .display()
                        .to_string();

                    content = content.push(text(format!("📁 {}", dir_name)));

                    for file in &files {
                        let file_name = file
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");

                        let file_clone = file.clone();
                        content = content.push(
                            button(text(format!("  📄 {}", file_name)))
                                .on_press(Message::FileSelected(file_clone))
                                .width(Length::Fill),
                        );
                    }
                } else {
                    // Files in root directory
                    for file in &files {
                        let file_name = file
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");

                        let file_clone = file.clone();
                        content = content.push(
                            button(text(format!("📄 {}", file_name)))
                                .on_press(Message::FileSelected(file_clone))
                                .width(Length::Fill),
                        );
                    }
                }
            }
        }

        scrollable(content).into()
    }
}
