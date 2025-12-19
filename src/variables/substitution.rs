use crate::types::RequestContext;
use anyhow::Result;
use super::extractor::extract_request_variable_value;
use super::parser::parse_request_variable;

pub fn substitute_request_variables(input: &str, context: &[RequestContext]) -> Result<String> {
    let mut result = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '{' {
            let mut j = i + 2;
            while j + 1 < chars.len() && !(chars[j] == '}' && chars[j + 1] == '}') {
                j += 1;
            }

            if j + 1 < chars.len() {
                let var_ref: String = chars[i..=j + 1].iter().collect();

                if var_ref.matches('.').count() >= 3 {
                    match parse_request_variable(&var_ref) {
                        Ok(request_var) => {
                            match extract_request_variable_value(&request_var, context) {
                                Ok(Some(value)) => result.push_str(&value),
                                _ => result.push_str(&var_ref),
                            }
                        }
                        Err(_) => result.push_str(&var_ref),
                    }
                } else {
                    result.push_str(&var_ref);
                }

                i = j + 2;
            } else {
                result.push(chars[i]);
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    Ok(result)
}
