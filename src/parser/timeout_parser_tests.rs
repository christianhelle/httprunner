use super::timeout_parser::*;

#[test]
fn test_parse_timeout_milliseconds() {
    assert_eq!(parse_timeout_value("1000ms"), Some(1000));
    assert_eq!(parse_timeout_value("500ms"), Some(500));
    assert_eq!(parse_timeout_value("1ms"), Some(1));
}

#[test]
fn test_parse_timeout_seconds() {
    assert_eq!(parse_timeout_value("5s"), Some(5000));
    assert_eq!(parse_timeout_value("1s"), Some(1000));
    assert_eq!(parse_timeout_value("30s"), Some(30000));
}

#[test]
fn test_parse_timeout_minutes() {
    assert_eq!(parse_timeout_value("1m"), Some(60000));
    assert_eq!(parse_timeout_value("2m"), Some(120000));
    assert_eq!(parse_timeout_value("5m"), Some(300000));
}

#[test]
fn test_parse_timeout_plain_number_defaults_to_seconds() {
    assert_eq!(parse_timeout_value("10"), Some(10000));
    assert_eq!(parse_timeout_value("1"), Some(1000));
}

#[test]
fn test_parse_timeout_with_whitespace() {
    assert_eq!(parse_timeout_value("  5000ms  "), Some(5000));
    assert_eq!(parse_timeout_value(" 10s "), Some(10000));
    assert_eq!(parse_timeout_value(" 2m "), Some(120000));
}

#[test]
fn test_parse_timeout_invalid_values() {
    assert_eq!(parse_timeout_value("abc"), None);
    assert_eq!(parse_timeout_value("ms"), None);
    assert_eq!(parse_timeout_value("s"), None);
    assert_eq!(parse_timeout_value(""), None);
}

#[test]
fn test_parse_timeout_negative_values() {
    assert_eq!(parse_timeout_value("-5s"), None);
    assert_eq!(parse_timeout_value("-1000ms"), None);
}

#[test]
fn test_parse_timeout_overflow() {
    // Test values that would overflow when multiplied
    assert_eq!(parse_timeout_value("18446744073709551615m"), None);
    assert_eq!(parse_timeout_value("18446744073709551615s"), None);
}

#[test]
fn test_parse_timeout_zero() {
    assert_eq!(parse_timeout_value("0ms"), Some(0));
    assert_eq!(parse_timeout_value("0s"), Some(0));
    assert_eq!(parse_timeout_value("0m"), Some(0));
}

#[test]
fn test_parse_timeout_case_sensitivity() {
    // Should not parse uppercase units
    assert_eq!(parse_timeout_value("5S"), None);
    assert_eq!(parse_timeout_value("5M"), None);
    assert_eq!(parse_timeout_value("5MS"), None);
}
