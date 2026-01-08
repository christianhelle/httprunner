use crate::types::Variable;

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
