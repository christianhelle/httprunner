use super::*;
use crate::error::Result;
use regex::Regex;

pub fn substitute_functions(input: &str) -> Result<String> {
    let guid_regex = Regex::new(r"\bguid\(\)").unwrap();
    let string_regex = Regex::new(r"\bstring\(\)").unwrap();

    let result = guid_regex.replace_all(&input, &generate_guid()).to_string();
    let result = string_regex
        .replace_all(&result, &generate_string(10))
        .to_string();

    Ok(result)
}
