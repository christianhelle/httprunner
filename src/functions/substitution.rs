use crate::error::Result;
use crate::functions::generator::{GuidSubstitutor, StringSubstitutor};
use regex::Regex;

pub trait FunctionSubstitutor {
    fn get_regex(&self) -> String;
    fn generate(&self) -> String;
    fn replace(&self, input: &String) -> String {
        Regex::new(&self.get_regex())
            .unwrap()
            .replace_all(&input, &self.generate())
            .to_string()
    }
}

pub fn substitute_functions(input: &str) -> Result<String> {
    let vec: Vec<Box<dyn FunctionSubstitutor>> = vec![
        Box::new(GuidSubstitutor {}),
        Box::new(StringSubstitutor {}),
    ];

    let mut result = input.to_string();
    for substitutor in vec {
        result = substitutor.replace(&result);
    }

    Ok(result)
}
