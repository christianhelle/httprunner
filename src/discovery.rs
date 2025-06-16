use walkdir::WalkDir;
use std::path::Path;
use anyhow::Result;
use colored::*;

pub fn discover_http_files<P: AsRef<Path>>(dir_path: P) -> Result<Vec<String>> {
    let mut files = Vec::new();
    
    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(extension) = entry.path().extension() {
                if extension == "http" {
                    if let Some(path_str) = entry.path().to_str() {
                        files.push(path_str.to_string());
                    }
                }
            }
        }
    }
    
    Ok(files)
}

pub fn run_discovery_mode() -> Result<Vec<String>> {
    println!("{} Discovering .http files recursively...{}", "🔍".blue(), "".normal());
    
    let files = discover_http_files(".")?;
    
    if files.is_empty() {
        println!("{} No .http files found in current directory and subdirectories{}", 
                 "⚠️".yellow(), "".normal());
        return Ok(files);
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
    fn test_discover_http_files() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();
        
        // Create some test files
        fs::write(temp_path.join("test1.http"), "GET https://example.com").unwrap();
        fs::write(temp_path.join("test2.txt"), "not an http file").unwrap();
        
        // Create subdirectory with http file
        let sub_dir = temp_path.join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        fs::write(sub_dir.join("test3.http"), "POST https://example.com").unwrap();
        
        let files = discover_http_files(temp_path).unwrap();
        
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.ends_with("test1.http")));
        assert!(files.iter().any(|f| f.ends_with("test3.http")));
        assert!(!files.iter().any(|f| f.ends_with("test2.txt")));
    }
}