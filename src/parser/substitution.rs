use crate::error::Result;
use crate::functions::substitute_functions;
use crate::types::{HttpRequest, Variable};

pub fn substitute_variables(input: &str, variables: &[Variable]) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                chars.next();

                let mut var_name = String::new();
                let mut found_closing = false;

                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        if chars.peek() == Some(&'}') {
                            chars.next();
                            found_closing = true;
                            break;
                        } else {
                            var_name.push(ch);
                        }
                    } else {
                        var_name.push(ch);
                    }
                }

                if found_closing {
                    if let Some(var) = variables.iter().find(|v| v.name == var_name) {
                        result.push_str(&var.value);
                    } else {
                        result.push_str("{{");
                        result.push_str(&var_name);
                        result.push_str("}}");
                    }
                } else {
                    result.push_str("{{");
                    result.push_str(&var_name);
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

pub fn substitute_functions_in_request(request: &mut HttpRequest) -> Result<()> {
    request.url = substitute_functions(&request.url)?;

    for header in &mut request.headers {
        header.name = substitute_functions(&header.name)?;
        header.value = substitute_functions(&header.value)?;
    }

    if let Some(ref body) = request.body {
        request.body = Some(substitute_functions(body)?);
    }

    for assertion in &mut request.assertions {
        assertion.expected_value = substitute_functions(&assertion.expected_value)?;
    }

    Ok(())
}
