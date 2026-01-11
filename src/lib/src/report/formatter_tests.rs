use super::formatter::*;

#[test]
fn test_escape_markdown_pipe() {
    assert_eq!(escape_markdown("test|value"), "test\\|value");
}

#[test]
fn test_escape_markdown_multiple_pipes() {
    assert_eq!(escape_markdown("a|b|c|d"), "a\\|b\\|c\\|d");
}

#[test]
fn test_escape_markdown_no_pipes() {
    assert_eq!(escape_markdown("no pipes here"), "no pipes here");
}

#[test]
fn test_escape_markdown_empty_string() {
    assert_eq!(escape_markdown(""), "");
}

#[test]
fn test_escape_markdown_only_pipe() {
    assert_eq!(escape_markdown("|"), "\\|");
}

#[test]
fn test_escape_markdown_pipes_at_boundaries() {
    assert_eq!(
        escape_markdown("|start|middle|end|"),
        "\\|start\\|middle\\|end\\|"
    );
}

#[test]
fn test_escape_markdown_with_special_chars() {
    assert_eq!(escape_markdown("test|value<>!@#"), "test\\|value<>!@#");
}

#[test]
fn test_escape_markdown_json_with_pipe() {
    assert_eq!(
        escape_markdown(r#"{"status":"ok|fail"}"#),
        r#"{"status":"ok\|fail"}"#
    );
}

#[test]
fn test_escape_markdown_url_with_pipe() {
    assert_eq!(
        escape_markdown("https://api.example.com?filter=a|b"),
        "https://api.example.com?filter=a\\|b"
    );
}

#[test]
fn test_escape_markdown_preserves_newlines() {
    assert_eq!(escape_markdown("line1|pipe\nline2"), "line1\\|pipe\nline2");
}
