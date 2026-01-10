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
    let temp = tempdir().unwrap();
    let orig_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp).unwrap();

    let files = run_discovery_mode().unwrap();
    assert!(files.is_empty());

    std::env::set_current_dir(orig_dir).unwrap();
}

