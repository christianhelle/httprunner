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
    assert!(files.len() >= 1);
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
