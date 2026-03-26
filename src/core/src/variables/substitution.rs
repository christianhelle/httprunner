use super::extractor::extract_request_variable_value;
use super::parser::parse_request_variable;
use crate::types::RequestContext;
use anyhow::{Result, anyhow};

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

                if looks_like_request_variable(&var_ref) {
                    let request_var = parse_request_variable(&var_ref)
                        .map_err(|e| anyhow!("Invalid request variable '{var_ref}': {e}"))?;
                    match extract_request_variable_value(&request_var, context)? {
                        Some(value) => result.push_str(&value),
                        None => {
                            return Err(anyhow!(
                                "Unable to resolve request variable: {}",
                                request_var.reference
                            ));
                        }
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

fn looks_like_request_variable(reference: &str) -> bool {
    let cleaned = reference
        .strip_prefix("{{")
        .and_then(|value| value.strip_suffix("}}"))
        .unwrap_or(reference);

    let mut parts = cleaned.split('.');
    parts.next();
    matches!(parts.next(), Some("request" | "response"))
}
