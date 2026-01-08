use crate::functions::generator::{
    Base64EncodeSubstitutor, GuidSubstitutor, NumberSubstitutor, StringSubstitutor,
};
use crate::functions::substitution::FunctionSubstitutor;

#[test]
fn test_generate_guid() {
    assert_eq!(GuidSubstitutor {}.generate().len(), 32);
}

#[test]
fn test_generate_string() {
    assert_eq!(StringSubstitutor {}.generate().len(), 20);
}

#[test]
fn test_generate_number() {
    assert_ne!(NumberSubstitutor {}.generate(), "-1");
}

#[test]
fn test_base64_encode() {
    assert_eq!(
        Base64EncodeSubstitutor {}.replace(&String::from("base64_encode('Hello, World!')")),
        "SGVsbG8sIFdvcmxkIQ=="
    );
}
