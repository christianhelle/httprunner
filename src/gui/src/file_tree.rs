use std::path::PathBuf;
use walkdir::WalkDir;

pub struct FileTree {
    #[allow(dead_code)]
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

    pub fn get_files(&self) -> Vec<PathBuf> {
        self.http_files.clone()
    }
}
