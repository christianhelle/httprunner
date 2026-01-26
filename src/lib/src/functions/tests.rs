use crate::functions::generators::{
    Base64EncodeSubstitutor, GuidSubstitutor, NumberSubstitutor, StringSubstitutor,
};
use crate::functions::substitution::FunctionSubstitutor;
use regex::Regex;

#[test]
fn test_generate_guid() {
    let guid = GuidSubstitutor {}.generate();
    let hex_pattern = Regex::new(r"^[0-9a-fA-F]{32}$").unwrap();
    assert!(
        hex_pattern.is_match(&guid),
        "GUID '{}' does not match pattern /^[0-9a-fA-F]{{32}}$/",
        guid
    );
}

#[test]
fn test_generate_string() {
    let string = StringSubstitutor {}.generate();
    let alphanum_pattern = Regex::new(r"^[A-Za-z0-9]+$").unwrap();
    assert!(
        alphanum_pattern.is_match(&string),
        "Generated string '{}' is not purely alphanumeric",
        string
    );
    assert_eq!(
        string.len(),
        20,
        "Generated string '{}' should be exactly 20 alphanumeric characters, got {}",
        string,
        string.len()
    );
}

#[test]
fn test_generate_number() {
    let number_str = NumberSubstitutor {}.generate();
    let number: i32 = number_str
        .parse()
        .expect("Generated number string could not be parsed as i32");
    assert!(
        (0..=100).contains(&number),
        "Generated number {} is not within range 0..=100",
        number
    );
    assert_ne!(number, -1, "Generated number should not be -1");
}

#[test]
fn test_base64_encode() {
    let result =
        Base64EncodeSubstitutor {}.replace(&String::from("base64_encode('Hello, World!')"));
    assert!(result.is_ok(), "Expected Ok result, got {:?}", result);
    assert_eq!(result.unwrap(), "SGVsbG8sIFdvcmxkIQ==");
}

#[test]
fn test_substitute_functions_case_insensitivity() {
    use crate::functions::substitute_functions;

    let input = r#"{"guid": "guid()", "GUID": "GUID()", "Guid": "Guid()"}"#;
    let result = substitute_functions(input).unwrap();

    // All three should be substituted with different GUIDs
    assert!(!result.contains("guid()"));
    assert!(!result.contains("GUID()"));
    assert!(!result.contains("Guid()"));

    // Verify the result contains valid GUIDs (32 hex characters)
    let guid_pattern = Regex::new(r"[0-9a-fA-F]{32}").unwrap();
    let matches: Vec<_> = guid_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3, "Should have 3 GUID values");
}

#[test]
fn test_substitute_functions_string_case_insensitivity() {
    use crate::functions::substitute_functions;

    let input = r#"{"s1": "string()", "s2": "STRING()", "s3": "String()"}"#;
    let result = substitute_functions(input).unwrap();

    // All three should be substituted
    assert!(!result.contains("string()"));
    assert!(!result.contains("STRING()"));
    assert!(!result.contains("String()"));

    // Verify the result contains alphanumeric strings
    let alphanum_pattern = Regex::new(r#""s\d": "([A-Za-z0-9]{20})""#).unwrap();
    let matches: Vec<_> = alphanum_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3, "Should have 3 string values");
}

#[test]
fn test_substitute_functions_number_case_insensitivity() {
    use crate::functions::substitute_functions;

    let input = r#"{"n1": number(), "n2": NUMBER(), "n3": Number()}"#;
    let result = substitute_functions(input).unwrap();

    // All three should be substituted
    assert!(!result.contains("number()"));
    assert!(!result.contains("NUMBER()"));
    assert!(!result.contains("Number()"));

    // Verify the result contains numbers
    let number_pattern = Regex::new(r#""n\d": (\d+)"#).unwrap();
    let matches: Vec<_> = number_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3, "Should have 3 number values");
}

#[test]
fn test_substitute_functions_base64_case_insensitivity() {
    use crate::functions::substitute_functions;

    let input = r#"{"b1": "base64_encode('test')", "b2": "BASE64_ENCODE('test')", "b3": "Base64_Encode('test')"}"#;
    let result = substitute_functions(input).unwrap();

    // All three should be substituted with the same base64 value
    assert!(!result.contains("base64_encode"));
    assert!(!result.contains("BASE64_ENCODE"));
    assert!(!result.contains("Base64_Encode"));

    // All should produce the same encoded value
    assert!(
        result.contains("dGVzdA=="),
        "Should contain base64 encoded 'test'"
    );

    // Verify all three were replaced
    let b64_pattern = Regex::new(r#""b\d": "dGVzdA==""#).unwrap();
    let matches: Vec<_> = b64_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3, "Should have 3 base64 values");
}

#[test]
fn test_substitute_functions_multiple_in_same_string() {
    use crate::functions::substitute_functions;

    let input = r#"{"id": "guid()", "name": "User string()", "score": number()}"#;
    let result = substitute_functions(input).unwrap();

    // Verify all functions were substituted
    assert!(!result.contains("guid()"));
    assert!(!result.contains("string()"));
    assert!(!result.contains("number()"));

    // Verify structure is maintained
    assert!(result.contains(r#""id":"#));
    assert!(result.contains(r#""name":"#));
    assert!(result.contains(r#""score":"#));
}

#[test]
fn test_substitute_functions_in_url() {
    use crate::functions::substitute_functions;

    let input = "https://api.example.com/users/guid()/posts/number()";
    let result = substitute_functions(input).unwrap();

    // Verify functions were substituted
    assert!(!result.contains("guid()"));
    assert!(!result.contains("number()"));

    // Verify URL structure is maintained
    assert!(result.starts_with("https://api.example.com/users/"));
    assert!(result.contains("/posts/"));
}

#[test]
fn test_substitute_functions_in_headers() {
    use crate::functions::substitute_functions;

    let input = "Authorization: Bearer base64_encode('user:pass')\nX-Request-ID: guid()";
    let result = substitute_functions(input).unwrap();

    // Verify functions were substituted
    assert!(!result.contains("base64_encode"));
    assert!(!result.contains("guid()"));

    // Verify headers structure
    assert!(result.contains("Authorization: Bearer"));
    assert!(result.contains("X-Request-ID:"));
}
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
        assert!((0..=100).contains(&num));
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
