use crate::error::Result;
use crate::functions::generator::{Base64EncodeSubstitutor, GuidSubstitutor, NumberSubstitutor, StringSubstitutor};
use regex::RegexBuilder;

pub trait FunctionSubstitutor {
    fn get_regex(&self) -> &str;
    fn generate(&self) -> String;
    fn replace(&self, input: &String) -> String {
        RegexBuilder::new(&self.get_regex())
            .case_insensitive(true)
            .build()
            .unwrap()
            .replace_all(&input, &self.generate())
            .to_string()
    }
}

pub fn substitute_functions(input: &str) -> Result<String> {
    let substitutors: &[&dyn FunctionSubstitutor] = &[
        &GuidSubstitutor {},
        &StringSubstitutor {},
        &NumberSubstitutor {},
        &Base64EncodeSubstitutor {}
    ];

    let result = substitutors
        .iter()
        .fold(input.to_string(), |acc, substitutor| {
            substitutor.replace(&acc)
        });

    Ok(result)
}
