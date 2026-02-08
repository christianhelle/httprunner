use super::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn load_environment_file_returns_empty_without_environment() {
    let temp = tempdir().unwrap();
    let http_file = temp.path().join("request.http");
    let vars = load_environment_file(http_file.to_str().unwrap(), None).unwrap();
    assert!(vars.is_empty());
}

#[test]
fn load_environment_file_reads_nearest_env_file() {
    let temp = tempdir().unwrap();
    let nested = temp.path().join("nested");
    fs::create_dir(&nested).unwrap();

    let http_file = nested.join("request.http");
    fs::write(&http_file, "GET http://example.com").unwrap();
    fs::write(
        temp.path().join("http-client.env.json"),
        r#"{"dev":{"TOKEN":"abc","COUNT":1,"FLAG":true}}"#,
    )
    .unwrap();

    let vars = load_environment_file(http_file.to_str().unwrap(), Some("dev")).unwrap();
    let map: std::collections::HashMap<_, _> =
        vars.into_iter().map(|v| (v.name, v.value)).collect();
    assert_eq!(map.get("TOKEN"), Some(&"abc".to_string()));
    assert_eq!(map.get("COUNT"), Some(&"1".to_string()));
    assert_eq!(map.get("FLAG"), Some(&"true".to_string()));
}

#[test]
fn find_environment_file_returns_none_when_absent() {
    let temp = tempdir().unwrap();
    let http_file = temp.path().join("request.http");
    fs::write(&http_file, "GET http://example.com").unwrap();
    let found = find_environment_file(http_file.to_str().unwrap()).unwrap();
    assert!(found.is_none());
}

#[test]
fn parse_environment_file_handles_non_string_values() {
    let temp = tempdir().unwrap();
    let env_file = temp.path().join("http-client.env.json");
    fs::write(
        &env_file,
        r#"{"dev":{"TEXT":"value","NUMBER":123,"OBJECT":{"foo":"bar"}}}"#,
    )
    .unwrap();

    let parsed = parse_environment_file(&env_file).unwrap();
    let dev = parsed.get("dev").unwrap();
    assert_eq!(dev.get("TEXT").unwrap(), "value");
    assert_eq!(dev.get("NUMBER").unwrap(), "123");
    assert_eq!(dev.get("OBJECT").unwrap(), r#"{"foo":"bar"}"#);
}

#[test]
fn find_environment_file_finds_file_in_parent_directory() {
    let temp = tempdir().unwrap();
    let nested = temp.path().join("nested");
    fs::create_dir(&nested).unwrap();

    let env_file = temp.path().join("http-client.env.json");
    fs::write(&env_file, r#"{"dev":{"KEY":"value"}}"#).unwrap();

    let http_file = nested.join("request.http");
    fs::write(&http_file, "GET http://example.com").unwrap();

    let found = find_environment_file(http_file.to_str().unwrap()).unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap(), env_file.to_string_lossy().to_string());
}

#[test]
fn load_environment_file_handles_missing_environment_name() {
    let temp = tempdir().unwrap();
    let env_file = temp.path().join("http-client.env.json");
    fs::write(&env_file, r#"{"prod":{"KEY":"value"}}"#).unwrap();

    let http_file = temp.path().join("request.http");
    fs::write(&http_file, "GET http://example.com").unwrap();

    let vars = load_environment_file(http_file.to_str().unwrap(), Some("dev")).unwrap();
    assert!(vars.is_empty());
}

#[test]
fn save_environment_file_writes_valid_json() {
    let temp = tempdir().unwrap();
    let env_file = temp.path().join("http-client.env.json");

    let mut config = std::collections::HashMap::new();
    let mut dev_vars = std::collections::HashMap::new();
    dev_vars.insert("API_URL".to_string(), "http://localhost".to_string());
    dev_vars.insert("TOKEN".to_string(), "abc123".to_string());
    config.insert("dev".to_string(), dev_vars);

    save_environment_file(&env_file, &config).unwrap();

    // Read back and verify
    let parsed = parse_environment_file(&env_file).unwrap();
    let dev = parsed.get("dev").unwrap();
    assert_eq!(dev.get("API_URL").unwrap(), "http://localhost");
    assert_eq!(dev.get("TOKEN").unwrap(), "abc123");
}

#[test]
fn save_environment_file_preserves_multiple_environments() {
    let temp = tempdir().unwrap();
    let env_file = temp.path().join("http-client.env.json");

    let mut config = std::collections::HashMap::new();

    let mut dev_vars = std::collections::HashMap::new();
    dev_vars.insert("URL".to_string(), "http://dev.example.com".to_string());
    config.insert("dev".to_string(), dev_vars);

    let mut prod_vars = std::collections::HashMap::new();
    prod_vars.insert("URL".to_string(), "https://prod.example.com".to_string());
    config.insert("prod".to_string(), prod_vars);

    save_environment_file(&env_file, &config).unwrap();

    let parsed = parse_environment_file(&env_file).unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(
        parsed.get("dev").unwrap().get("URL").unwrap(),
        "http://dev.example.com"
    );
    assert_eq!(
        parsed.get("prod").unwrap().get("URL").unwrap(),
        "https://prod.example.com"
    );
}
