use std::path::PathBuf;
use walkdir::WalkDir;

pub struct FileTree {
    root_path: PathBuf,
    http_files: Vec<PathBuf>,
    expanded_dirs: std::collections::HashSet<PathBuf>,
}

impl FileTree {
    pub fn new(root_path: PathBuf) -> Self {
        let mut http_files = Vec::new();

        // Discover all .http files
        for entry in WalkDir::new(&root_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file()
                && let Some(ext) = entry.path().extension()
                    && ext == "http" {
                        http_files.push(entry.path().to_path_buf());
                    }
        }

        // Sort files by path
        http_files.sort();

        Self {
            root_path,
            http_files,
            expanded_dirs: std::collections::HashSet::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<PathBuf> {
        let mut selected_file = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Group files by directory
            let mut dir_files: std::collections::BTreeMap<Option<PathBuf>, Vec<PathBuf>> =
                std::collections::BTreeMap::new();

            for file in &self.http_files {
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

            if self.http_files.is_empty() {
                ui.label("No .http files found in this directory.");
                ui.label("Use File -> Open Directory to choose a different folder.");
            }
        });

        selected_file
    }
}
