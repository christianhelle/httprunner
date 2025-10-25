use crate::colors;
use anyhow::Result;
use walkdir::WalkDir;

pub fn discover_http_files(dir_path: &str) -> Result<Vec<String>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(path_str) = entry.path().to_str() {
                if path_str.ends_with(".http") {
                    files.push(path_str.to_string());
                }
            }
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
