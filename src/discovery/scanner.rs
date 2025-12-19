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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn discover_http_files_finds_nested_files() {
        let temp = tempdir().unwrap();
        let nested = temp.path().join("nested");
        fs::create_dir(&nested).unwrap();

        let first = temp.path().join("first.http");
        let second = nested.join("second.http");
        fs::write(&first, "GET http://example.com").unwrap();
        fs::write(&second, "POST http://example.com").unwrap();
        fs::write(temp.path().join("ignore.txt"), "noop").unwrap();

        let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(files.len(), 2);
        let first_str = first.to_string_lossy().to_string();
        let second_str = second.to_string_lossy().to_string();
        assert!(files.contains(&first_str));
        assert!(files.contains(&second_str));
    }

    #[test]
    fn discover_http_files_returns_empty_when_none_found() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("file.txt"), "noop").unwrap();
        let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
        assert!(files.is_empty());
    }
}
