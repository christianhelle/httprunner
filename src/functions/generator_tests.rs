use super::generator::*;
use super::substitution::FunctionSubstitutor;

#[test]
fn test_guid_substitutor_regex() {
    let sub = GuidSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    assert!(regex.is_match("guid()"));
    assert!(regex.is_match("Bearer guid()"));
    assert!(!regex.is_match("noguid()"));
    assert!(!regex.is_match("myguid()"));
}

#[test]
fn test_guid_substitutor_generates_valid_uuid() {
    let sub = GuidSubstitutor {};
    let guid = sub.generate();

    // UUID v4 in simple format should be 32 hex characters
    assert_eq!(guid.len(), 32);
    assert!(guid.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_guid_substitutor_generates_unique_values() {
    let sub = GuidSubstitutor {};
    let guid1 = sub.generate();
    let guid2 = sub.generate();

    assert_ne!(guid1, guid2);
}

#[test]
fn test_string_substitutor_regex() {
    let sub = StringSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    assert!(regex.is_match("string()"));
    assert!(regex.is_match("test string()"));
    assert!(!regex.is_match("nostring()"));
    assert!(!regex.is_match("mystring()"));
}

#[test]
fn test_string_substitutor_generates_20_chars() {
    let sub = StringSubstitutor {};
    let s = sub.generate();

    assert_eq!(s.len(), 20);
}

#[test]
fn test_string_substitutor_generates_alphanumeric() {
    let sub = StringSubstitutor {};
    let s = sub.generate();

    assert!(s.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[test]
fn test_string_substitutor_generates_different_values() {
    let sub = StringSubstitutor {};
    let s1 = sub.generate();
    let s2 = sub.generate();

    // Should be highly unlikely to generate the same 20-char string
    assert_ne!(s1, s2);
}

#[test]
fn test_number_substitutor_regex() {
    let sub = NumberSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    assert!(regex.is_match("number()"));
    assert!(regex.is_match("value: number()"));
    assert!(!regex.is_match("nonumber()"));
    assert!(!regex.is_match("mynumber()"));
}

#[test]
fn test_number_substitutor_generates_in_range() {
    let sub = NumberSubstitutor {};

    for _ in 0..100 {
        let num_str = sub.generate();
        let num: i32 = num_str.parse().unwrap();
        assert!(num >= 0 && num <= 100);
    }
}

#[test]
fn test_number_substitutor_generates_numeric_string() {
    let sub = NumberSubstitutor {};
    let num_str = sub.generate();

    assert!(num_str.parse::<i32>().is_ok());
}

#[test]
fn test_base64_encode_substitutor_single_quote() {
    let sub = Base64EncodeSubstitutor {};
    let input = "Bearer base64_encode('username:password')";
    let result = sub.replace(input).unwrap();

    assert_eq!(result, "Bearer dXNlcm5hbWU6cGFzc3dvcmQ=");
}

#[test]
fn test_base64_encode_substitutor_with_spaces() {
    let sub = Base64EncodeSubstitutor {};
    let input = "base64_encode( 'test' )";
    let result = sub.replace(input).unwrap();

    assert_eq!(result, "dGVzdA==");
}

#[test]
fn test_base64_encode_substitutor_case_insensitive() {
    let sub = Base64EncodeSubstitutor {};

    let input1 = "BASE64_ENCODE('hello')";
    let result1 = sub.replace(input1).unwrap();
    assert_eq!(result1, "aGVsbG8=");

    let input2 = "Base64_Encode('hello')";
    let result2 = sub.replace(input2).unwrap();
    assert_eq!(result2, "aGVsbG8=");
}

#[test]
fn test_base64_encode_substitutor_empty_string() {
    let sub = Base64EncodeSubstitutor {};
    let input = "base64_encode('')";
    let result = sub.replace(input).unwrap();

    assert_eq!(result, "");
}

#[test]
fn test_base64_encode_substitutor_special_chars() {
    let sub = Base64EncodeSubstitutor {};
    let input = "base64_encode('Hello, World!')";
    let result = sub.replace(input).unwrap();

    assert_eq!(result, "SGVsbG8sIFdvcmxkIQ==");
}

#[test]
fn test_base64_encode_substitutor_multiple() {
    let sub = Base64EncodeSubstitutor {};
    let input = "base64_encode('foo') and base64_encode('bar')";
    let result = sub.replace(input).unwrap();

    assert_eq!(result, "Zm9v and YmFy");
}

#[test]
fn test_base64_encode_substitutor_no_match() {
    let sub = Base64EncodeSubstitutor {};
    let input = "no encoding here";
    let result = sub.replace(input).unwrap();

    assert_eq!(result, "no encoding here");
}

#[test]
fn test_base64_encode_substitutor_json_body() {
    let sub = Base64EncodeSubstitutor {};
    let input = r#"{"auth": "base64_encode('user:pass')"}"#;
    let result = sub.replace(input).unwrap();

    assert_eq!(result, r#"{"auth": "dXNlcjpwYXNz"}"#);
}
