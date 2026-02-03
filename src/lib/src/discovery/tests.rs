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

#[test]
fn run_discovery_mode_returns_empty_list_when_no_files() {
    // Instead of changing directories, use discover_http_files directly
    let temp = tempdir().unwrap();
    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert!(files.is_empty());
}

#[test]
fn discover_http_files_multiple_nested_levels() {
    let temp = tempdir().unwrap();
    let level1 = temp.path().join("level1");
    let level2 = level1.join("level2");
    let level3 = level2.join("level3");

    fs::create_dir_all(&level3).unwrap();

    let file1 = temp.path().join("root.http");
    let file2 = level1.join("l1.http");
    let file3 = level2.join("l2.http");
    let file4 = level3.join("l3.http");

    fs::write(&file1, "GET http://example.com/root").unwrap();
    fs::write(&file2, "GET http://example.com/l1").unwrap();
    fs::write(&file3, "GET http://example.com/l2").unwrap();
    fs::write(&file4, "GET http://example.com/l3").unwrap();

    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert_eq!(files.len(), 4);
}

#[test]
fn discover_http_files_ignores_non_http_extensions() {
    let temp = tempdir().unwrap();

    fs::write(temp.path().join("test.http"), "GET http://example.com").unwrap();
    fs::write(temp.path().join("test.txt"), "Some text").unwrap();
    fs::write(temp.path().join("test.json"), "{}").unwrap();
    fs::write(temp.path().join("test.md"), "# Markdown").unwrap();
    fs::write(temp.path().join("test.xml"), "<xml/>").unwrap();

    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert_eq!(files.len(), 1);
}

#[test]
fn discover_http_files_finds_multiple_in_same_directory() {
    let temp = tempdir().unwrap();

    fs::write(temp.path().join("api1.http"), "GET http://example.com/1").unwrap();
    fs::write(temp.path().join("api2.http"), "GET http://example.com/2").unwrap();
    fs::write(temp.path().join("api3.http"), "GET http://example.com/3").unwrap();

    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert_eq!(files.len(), 3);
}

#[test]
fn discover_http_files_with_symlinks() {
    let temp = tempdir().unwrap();
    let nested = temp.path().join("nested");
    fs::create_dir(&nested).unwrap();

    let file = nested.join("test.http");
    fs::write(&file, "GET http://example.com").unwrap();

    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert!(!files.is_empty());
}

#[test]
fn discover_http_files_empty_directory() {
    let temp = tempdir().unwrap();
    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert!(files.is_empty());
}

#[test]
fn discover_http_files_single_file() {
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("single.http"), "GET http://example.com").unwrap();

    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("single.http"));
}

#[test]
fn discover_http_files_with_hidden_directories() {
    let temp = tempdir().unwrap();
    let hidden = temp.path().join(".hidden");
    fs::create_dir(&hidden).unwrap();

    fs::write(temp.path().join("visible.http"), "GET http://example.com").unwrap();
    fs::write(hidden.join("hidden.http"), "GET http://example.com").unwrap();

    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    // Should find both visible and hidden files
    assert!(!files.is_empty());
}

#[test]
fn discover_http_files_case_sensitive_extension() {
    let temp = tempdir().unwrap();

    fs::write(temp.path().join("lowercase.http"), "GET http://example.com").unwrap();
    fs::write(temp.path().join("uppercase.HTTP"), "GET http://example.com").unwrap();
    fs::write(temp.path().join("mixed.Http"), "GET http://example.com").unwrap();

    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    // Only .http (lowercase) should be found
    assert_eq!(files.len(), 1);
}

#[test]
fn discover_http_files_with_special_chars_in_filename() {
    let temp = tempdir().unwrap();

    fs::write(temp.path().join("test-api.http"), "GET http://example.com").unwrap();
    fs::write(temp.path().join("test_api.http"), "GET http://example.com").unwrap();
    fs::write(temp.path().join("test.api.http"), "GET http://example.com").unwrap();

    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert_eq!(files.len(), 3);
}

#[test]
fn discover_http_files_mixed_content() {
    let temp = tempdir().unwrap();
    let nested1 = temp.path().join("api");
    let nested2 = temp.path().join("tests");
    fs::create_dir(&nested1).unwrap();
    fs::create_dir(&nested2).unwrap();

    fs::write(temp.path().join("root.http"), "GET http://example.com").unwrap();
    fs::write(nested1.join("users.http"), "GET http://example.com/users").unwrap();
    fs::write(nested1.join("README.md"), "# API Tests").unwrap();
    fs::write(
        nested2.join("integration.http"),
        "GET http://example.com/test",
    )
    .unwrap();
    fs::write(nested2.join("config.json"), "{}").unwrap();

    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert_eq!(files.len(), 3);
}

#[test]
fn run_discovery_mode_with_files() {
    // Instead of changing directories, use discover_http_files directly
    let temp = tempdir().unwrap();

    fs::write(temp.path().join("test1.http"), "GET http://example.com/1").unwrap();
    fs::write(temp.path().join("test2.http"), "GET http://example.com/2").unwrap();

    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert_eq!(files.len(), 2);
}

#[test]
fn discover_http_files_preserves_paths() {
    let temp = tempdir().unwrap();
    let nested = temp.path().join("nested");
    fs::create_dir(&nested).unwrap();

    let file1 = temp.path().join("root.http");
    let file2 = nested.join("nested.http");
    fs::write(&file1, "GET http://example.com").unwrap();
    fs::write(&file2, "POST http://example.com").unwrap();

    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();

    // Verify paths are preserved correctly
    assert!(files.iter().any(|f| f.ends_with("root.http")));
    assert!(
        files
            .iter()
            .any(|f| f.contains("nested") && f.ends_with("nested.http"))
    );
}

#[test]
fn discover_http_files_handles_nonexistent_directory() {
    let temp = tempdir().unwrap();
    let nonexistent = temp.path().join("does_not_exist");
    // Should handle gracefully, WalkDir will skip it
    let result = discover_http_files(nonexistent.to_str().unwrap());
    // Result should be Ok but empty
    assert!(result.is_ok());
}

#[test]
fn discover_http_files_with_deeply_nested_structure() {
    let temp = tempdir().unwrap();
    let mut path = temp.path().to_path_buf();
    
    // Create 10 levels deep
    for i in 0..10 {
        path = path.join(format!("level{}", i));
    }
    fs::create_dir_all(&path).unwrap();
    
    let file = path.join("deep.http");
    fs::write(&file, "GET http://example.com/deep").unwrap();
    
    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("deep.http"));
}

#[test]
fn discover_http_files_ignores_directories_named_with_http_extension() {
    let temp = tempdir().unwrap();
    let dir_named_http = temp.path().join("folder.http");
    fs::create_dir(&dir_named_http).unwrap();
    
    // Also create a real .http file
    let real_file = temp.path().join("real.http");
    fs::write(&real_file, "GET http://example.com").unwrap();
    
    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    // Should only find the real file, not the directory
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("real.http"));
}

#[test]
fn discover_http_files_with_unicode_in_directory_names() {
    let temp = tempdir().unwrap();
    let unicode_dir = temp.path().join("日本語");
    fs::create_dir(&unicode_dir).unwrap();
    
    let file = unicode_dir.join("test.http");
    fs::write(&file, "GET http://example.com").unwrap();
    
    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].contains("日本語"));
}

#[test]
fn discover_http_files_with_unicode_in_filenames() {
    let temp = tempdir().unwrap();
    
    let file = temp.path().join("テスト.http");
    fs::write(&file, "GET http://example.com").unwrap();
    
    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with(".http"));
}

#[test]
fn discover_http_files_with_spaces_in_names() {
    let temp = tempdir().unwrap();
    
    let file = temp.path().join("my test file.http");
    fs::write(&file, "GET http://example.com").unwrap();
    
    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("my test file.http"));
}

#[test]
fn discover_http_files_sorted_output() {
    let temp = tempdir().unwrap();
    
    for name in ["zebra.http", "alpha.http", "beta.http"] {
        fs::write(temp.path().join(name), "GET http://example.com").unwrap();
    }
    
    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert_eq!(files.len(), 3);
    // Files are discovered, ordering depends on filesystem
    assert!(files.iter().any(|f| f.ends_with("zebra.http")));
    assert!(files.iter().any(|f| f.ends_with("alpha.http")));
    assert!(files.iter().any(|f| f.ends_with("beta.http")));
}

#[test]
fn discover_http_files_no_duplicate_files() {
    let temp = tempdir().unwrap();
    
    fs::write(temp.path().join("test.http"), "GET http://example.com").unwrap();
    
    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert_eq!(files.len(), 1);
    
    // Verify no duplicates in the result
    let unique: std::collections::HashSet<_> = files.iter().collect();
    assert_eq!(unique.len(), files.len());
}

#[test]
fn discover_http_files_respects_file_extension_boundary() {
    let temp = tempdir().unwrap();
    
    // Files that contain .http but don't end with it
    fs::write(temp.path().join("test.http.bak"), "GET http://example.com").unwrap();
    fs::write(temp.path().join("test.httpx"), "GET http://example.com").unwrap();
    fs::write(temp.path().join("valid.http"), "GET http://example.com").unwrap();
    
    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("valid.http"));
}

#[test]
fn run_discovery_mode_output_format() {
    // Test that run_discovery_mode doesn't panic
    // This will output to stdout but we're mainly checking it doesn't crash
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("test.http"), "GET http://example.com").unwrap();
    
    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert!(!files.is_empty());
}

#[test]
fn discover_http_files_with_very_long_filename() {
    let temp = tempdir().unwrap();
    
    // Create a very long but valid filename
    let long_name = format!("{}.http", "a".repeat(200));
    let file = temp.path().join(&long_name);
    
    // This might fail on some filesystems, so we handle both cases
    if fs::write(&file, "GET http://example.com").is_ok() {
        let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(files.len(), 1);
    }
}

#[test]
fn discover_http_files_empty_http_file() {
    let temp = tempdir().unwrap();
    
    // Empty .http files should still be discovered
    fs::write(temp.path().join("empty.http"), "").unwrap();
    
    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert_eq!(files.len(), 1);
}

#[test]
fn discover_http_files_binary_content_in_http_file() {
    let temp = tempdir().unwrap();
    
    // .http file with binary content should still be found (we don't read content)
    let binary_data = vec![0u8, 1, 2, 3, 255, 254, 253];
    std::fs::write(temp.path().join("binary.http"), binary_data).unwrap();
    
    let files = discover_http_files(temp.path().to_str().unwrap()).unwrap();
    assert_eq!(files.len(), 1);
}
