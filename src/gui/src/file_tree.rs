use crate::app::AppEvent;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc::UnboundedSender;

#[cfg(not(target_arch = "wasm32"))]
use std::thread;
#[cfg(not(target_arch = "wasm32"))]
use walkdir::WalkDir;

#[cfg(not(target_arch = "wasm32"))]
pub fn start_discovery(root_path: PathBuf, sender: UnboundedSender<AppEvent>) {
    let _ = sender.send(AppEvent::DiscoveryStarted);

    thread::spawn(move || {
        for entry in WalkDir::new(&root_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|entry| entry.ok())
        {
            if entry.file_type().is_file()
                && let Some(extension) = entry.path().extension()
                && extension == "http"
            {
                let _ = sender.send(AppEvent::FileDiscovered(entry.path().to_path_buf()));
            }
        }

        let _ = sender.send(AppEvent::DiscoveryFinished);
    });
}

#[cfg(target_arch = "wasm32")]
pub fn start_discovery(_root_path: PathBuf, _sender: UnboundedSender<AppEvent>) {}

pub fn group_files_by_directory(
    root_path: &Path,
    files: &[PathBuf],
) -> BTreeMap<Option<PathBuf>, Vec<PathBuf>> {
    let mut groups: BTreeMap<Option<PathBuf>, Vec<PathBuf>> = BTreeMap::new();

    for file in files {
        let parent = file.parent().map(Path::to_path_buf);
        groups.entry(parent).or_default().push(file.clone());
    }

    for paths in groups.values_mut() {
        paths.sort();
    }

    if let Some(root_files) = groups.get_mut(&Some(root_path.to_path_buf())) {
        let mut files_in_root = Vec::new();
        files_in_root.append(root_files);
        groups.insert(None, files_in_root);
        groups.remove(&Some(root_path.to_path_buf()));
    }

    groups
}

pub fn relative_directory_name(root_path: &Path, directory: &Path) -> String {
    directory
        .strip_prefix(root_path)
        .unwrap_or(directory)
        .display()
        .to_string()
}
