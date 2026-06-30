use crate::colors;
use anyhow::Result;
use walkdir::WalkDir;

#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

/// Recursively discover `.http` files under `root`, returned in sorted order.
///
/// `on_progress` is called with the running count as each file is found, letting
/// UIs report discovery progress. Shared by the GUI and TUI file trees so the
/// walk/filter/sort logic lives in one place.
#[cfg(not(target_arch = "wasm32"))]
pub fn discover_http_file_paths<F>(root: &Path, mut on_progress: F) -> Vec<PathBuf>
where
    F: FnMut(usize),
{
    let mut files = Vec::new();

    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file()
            && entry.path().extension().is_some_and(|ext| ext == "http")
        {
            files.push(entry.path().to_path_buf());
            on_progress(files.len());
        }
    }

    files.sort();
    files
}

pub(crate) fn discover_http_files(dir_path: &str) -> Result<Vec<String>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file()
            && let Some(path_str) = entry.path().to_str()
            && path_str.ends_with(".http")
        {
            files.push(path_str.to_string());
        }
    }

    Ok(files)
}

pub fn run_discovery_mode() -> Result<Vec<String>> {
    println!(
        "{} Discovering .http files recursively...",
        colors::blue("🔍")
    );

    let files = discover_http_files(".")?;

    if files.is_empty() {
        println!(
            "{} No .http files found in current directory and subdirectories",
            colors::yellow("⚠️")
        );
        return Ok(Vec::new());
    }

    println!("Found {} .http file(s):", files.len());
    for file_path in &files {
        println!("  📄 {}", file_path);
    }
    println!();

    Ok(files)
}
