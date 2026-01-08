use crate::functions::generator::{GuidSubstitutor, NumberSubstitutor, StringSubstitutor};
use crate::functions::substitution::FunctionSubstitutor;

#[test]
fn test_generate_guid() {
    assert_eq!(GuidSubstitutor {}.generate().len(), 32);
}

#[test]
fn test_generate_string() {
    assert_eq!(StringSubstitutor {}.generate().len(), 10);
}

#[test]
fn test_generate_number() {
    assert_ne!(NumberSubstitutor {}.generate(), "-1");
}
