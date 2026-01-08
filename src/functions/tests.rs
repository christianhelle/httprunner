use crate::functions::generator::{
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
    assert!(
        string.len() >= 10,
        "Generated string '{}' should be at least 10 alphanumeric characters, got {}",
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
        number >= 0 && number <= 100,
        "Generated number {} is not within range 0..=100",
        number
    );
    assert_ne!(
        number, -1,
        "Generated number should not be -1"
    );
}

#[test]
fn test_base64_encode() {
    assert_eq!(
        Base64EncodeSubstitutor {}.replace(&String::from("base64_encode('Hello, World!')")),
        "SGVsbG8sIFdvcmxkIQ=="
    );
}
