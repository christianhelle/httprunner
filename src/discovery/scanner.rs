use crate::colors;
use anyhow::Result;
use walkdir::WalkDir;

/// Discover all .http files recursively in a directory
///
/// Walks the directory tree starting from the specified path and finds
/// all files with the .http extension.
///
/// # Arguments
///
/// * `dir_path` - Starting directory path to search from
///
/// # Returns
///
/// Returns a vector of file paths to all discovered .http files
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

/// Run discovery mode to find and list all .http files in current directory
///
/// Recursively searches the current directory for .http files and prints
/// the discovered files to stdout.
///
/// # Returns
///
/// Returns a vector of discovered file paths
///
/// # Errors
///
/// Returns an error if the directory cannot be traversed
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
