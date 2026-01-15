use iced::widget::{button, text, Column};
use iced::Element;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub enum Message {
    FileClicked(PathBuf),
}

pub struct FileTree {
    root_path: PathBuf,
    http_files: Vec<PathBuf>,
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
                && ext == "http"
            {
                http_files.push(entry.path().to_path_buf());
            }
        }

        // Sort files by path
        http_files.sort();

        Self {
            root_path,
            http_files,
        }
    }

    pub fn update(&mut self, message: Message) -> Option<PathBuf> {
        match message {
            Message::FileClicked(path) => Some(path),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let mut content: Column<Message> = Column::new().spacing(5);

        if self.http_files.is_empty() {
            content = content
                .push(text("No .http files found in this directory."))
                .push(text("Use File -> Open Directory to choose a different folder."));
        } else {
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

                    content = content.push(text(format!("📂 {}", dir_name)).size(14));

                    for file in &files {
                        let file_name = file
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");

                        let file_button = button(text(format!("  📄 {}", file_name)))
                            .on_press(Message::FileClicked(file.clone()));

                        content = content.push(file_button);
                    }
                } else {
                    // Files in root directory
                    for file in &files {
                        let file_name = file
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");

                        let file_button = button(text(format!("📄 {}", file_name)))
                            .on_press(Message::FileClicked(file.clone()));

                        content = content.push(file_button);
                    }
                }
            }
        }

        content.into()
    }
}
