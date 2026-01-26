use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use walkdir::WalkDir;

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
                if entry.file_type().is_file()
                    && let Some(ext) = entry.path().extension()
                    && ext == "http"
                {
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

    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<PathBuf> {
        let mut selected_file = None;

        // Show discovery status
        if self.is_discovering() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(format!(
                    "Discovering .http files... ({})",
                    self.discovered_count()
                ));
            });
            ui.separator();
            // Request repaint to keep updating during discovery
            ui.ctx().request_repaint();
        }

        // Get a snapshot of current files
        let http_files = self
            .http_files
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Group files by directory
            let mut dir_files: std::collections::BTreeMap<Option<PathBuf>, Vec<PathBuf>> =
                std::collections::BTreeMap::new();

            for file in &http_files {
                let parent = file.parent().map(|p| p.to_path_buf());
                dir_files.entry(parent).or_default().push(file.clone());
            }

            // Show files organized by directory
            for (dir, files) in dir_files {
                if let Some(dir_path) = dir {
                    let dir_name = dir_path
                        .strip_prefix(&self.root_path)
                        .unwrap_or(&dir_path)
                        .display()
                        .to_string();

                    let is_expanded = self.expanded_dirs.contains(&dir_path);
                    let header_text = if is_expanded {
                        format!("📂 {}", dir_name)
                    } else {
                        format!("📁 {}", dir_name)
                    };

                    let header = egui::CollapsingHeader::new(header_text)
                        .default_open(true)
                        .show(ui, |ui| {
                            for file in &files {
                                let file_name = file
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown");

                                if ui
                                    .selectable_label(false, format!("📄 {}", file_name))
                                    .clicked()
                                {
                                    selected_file = Some(file.clone());
                                }
                            }
                        });

                    if header.header_response.clicked() {
                        if is_expanded {
                            self.expanded_dirs.remove(&dir_path);
                        } else {
                            self.expanded_dirs.insert(dir_path.clone());
                        }
                    }
                } else {
                    // Files in root directory
                    for file in &files {
                        let file_name = file
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");

                        if ui
                            .selectable_label(false, format!("📄 {}", file_name))
                            .clicked()
                        {
                            selected_file = Some(file.clone());
                        }
                    }
                }
            }

            if http_files.is_empty() && !self.is_discovering() {
                ui.label("No .http files found in this directory.");
                ui.label("Use File -> Open Directory to choose a different folder.");
            }
        });

        selected_file
    }
}
