use super::helpers::*;

#[test]
fn test_red_wraps_text_with_ansi_codes() {
    let result = red("error");
    assert_eq!(result, "\x1b[31merror\x1b[0m");
}

#[test]
fn test_red_empty_string() {
    let result = red("");
    assert_eq!(result, "\x1b[31m\x1b[0m");
}

#[test]
fn test_red_with_special_chars() {
    let result = red("Error: Failed!");
    assert_eq!(result, "\x1b[31mError: Failed!\x1b[0m");
}

#[test]
fn test_green_wraps_text_with_ansi_codes() {
    let result = green("success");
    assert_eq!(result, "\x1b[32msuccess\x1b[0m");
}

#[test]
fn test_green_empty_string() {
    let result = green("");
    assert_eq!(result, "\x1b[32m\x1b[0m");
}

#[test]
fn test_green_with_special_chars() {
    let result = green("✓ Success!");
    assert_eq!(result, "\x1b[32m✓ Success!\x1b[0m");
}

#[test]
fn test_yellow_wraps_text_with_ansi_codes() {
    let result = yellow("warning");
    assert_eq!(result, "\x1b[33mwarning\x1b[0m");
}

#[test]
fn test_yellow_empty_string() {
    let result = yellow("");
    assert_eq!(result, "\x1b[33m\x1b[0m");
}

#[test]
fn test_yellow_with_numbers() {
    let result = yellow("Code: 404");
    assert_eq!(result, "\x1b[33mCode: 404\x1b[0m");
}

#[test]
fn test_blue_wraps_text_with_ansi_codes() {
    let result = blue("info");
    assert_eq!(result, "\x1b[34minfo\x1b[0m");
}

#[test]
fn test_blue_empty_string() {
    let result = blue("");
    assert_eq!(result, "\x1b[34m\x1b[0m");
}

#[test]
fn test_blue_with_url() {
    let result = blue("https://example.com");
    assert_eq!(result, "\x1b[34mhttps://example.com\x1b[0m");
}

#[test]
fn test_color_functions_preserve_content() {
    let original = "test content";

    assert!(red(original).contains(original));
    assert!(green(original).contains(original));
    assert!(yellow(original).contains(original));
    assert!(blue(original).contains(original));
}

#[test]
fn test_nested_colors_not_affected() {
    // Each function should just wrap the text, even if it already has ANSI codes
    let already_colored = "\x1b[31mred\x1b[0m";
    let result = green(already_colored);
    assert_eq!(result, "\x1b[32m\x1b[31mred\x1b[0m\x1b[0m");
}

#[test]
fn test_multiline_text() {
    let multiline = "line1\nline2\nline3";
    let result = red(multiline);
    assert_eq!(result, "\x1b[31mline1\nline2\nline3\x1b[0m");
}

#[test]
fn test_unicode_characters() {
    let unicode = "Hello 世界 🌍";
    let result = green(unicode);
    assert_eq!(result, "\x1b[32mHello 世界 🌍\x1b[0m");
}
