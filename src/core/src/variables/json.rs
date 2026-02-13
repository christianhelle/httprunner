use anyhow::{anyhow, Result};

pub fn extract_json_property(json_body: &str, property: &str) -> Result<Option<String>> {
    // Handle nested properties like "json.token" and array indexing like "data[0].version"
    let parts: Vec<&str> = property.split('.').collect();
    let mut current_json = json_body.to_string();

    for part in parts {
        if let Some(bracket_pos) = part.find('[') {
            let property_name = &part[..bracket_pos];
            let index_part = &part[bracket_pos..];

            match extract_simple_json_property(&current_json, property_name) {
                Ok(Some(value)) => {
                    current_json = value;

                    if let Some(index_value) = parse_array_index(index_part) {
                        match extract_array_element(&current_json, index_value) {
                            Ok(Some(element)) => current_json = element,
                            Ok(None) => return Ok(None),
                            Err(e) => return Err(e),
                        }
                    } else {
                        return Err(anyhow!("Invalid array index: {}", index_part));
                    }
                }
                Ok(None) => return Ok(None),
                Err(e) => return Err(e),
            }
        } else {
            match extract_simple_json_property(&current_json, part) {
                Ok(Some(value)) => current_json = value,
                Ok(None) => return Ok(None),
                Err(e) => return Err(e),
            }
        }
    }

    Ok(Some(current_json))
}

fn parse_array_index(index_str: &str) -> Option<usize> {
    // Parse "[0]" or "[123]" format
    if index_str.starts_with('[') && index_str.ends_with(']') {
        let index_num = &index_str[1..index_str.len() - 1];
        index_num.parse::<usize>().ok()
    } else {
        None
    }
}

fn extract_array_element(json_body: &str, index: usize) -> Result<Option<String>> {
    let trimmed = json_body.trim();

    if !trimmed.starts_with('[') {
        return Err(anyhow!("Expected array but got: {}", trimmed));
    }

    let chars: Vec<char> = trimmed.chars().collect();
    let mut pos = 1; // Skip opening [
    let mut current_index = 0;

    while pos < chars.len() {
        // Skip whitespace
        while pos < chars.len() && matches!(chars[pos], ' ' | '\t' | '\n' | '\r') {
            pos += 1;
        }

        if pos >= chars.len() || chars[pos] == ']' {
            return Ok(None); // Array ended, index not found
        }

        let element_start = pos;

        // Find the end of this array element
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;

        while pos < chars.len() {
            if escape_next {
                escape_next = false;
                pos += 1;
                continue;
            }

            match chars[pos] {
                '\\' => escape_next = true,
                '"' if !escape_next => in_string = !in_string,
                '{' | '[' if !in_string => depth += 1,
                '}' | ']' if !in_string => depth -= 1,
                ',' if !in_string && depth == 0 => break,
                _ => {}
            }

            if depth < 0 {
                break; // Closing array bracket
            }

            pos += 1;
        }

        if current_index == index {
            let element: String = chars[element_start..pos].iter().collect();
            return Ok(Some(element.trim().to_string()));
        }

        current_index += 1;

        // Skip comma
        if pos < chars.len() && chars[pos] == ',' {
            pos += 1;
        }
    }

    Ok(None)
}

fn extract_simple_json_property(json_body: &str, property: &str) -> Result<Option<String>> {
    let search_pattern = format!("\"{}\":", property);

    if let Some(start_pos) = json_body.find(&search_pattern) {
        let mut pos = start_pos + search_pattern.len();
        let chars: Vec<char> = json_body.chars().collect();

        // Skip whitespace
        while pos < chars.len() && matches!(chars[pos], ' ' | '\t' | '\n' | '\r') {
            pos += 1;
        }

        if pos >= chars.len() {
            return Ok(None);
        }

        if chars[pos] == '"' {
            pos += 1;
            let value_start = pos;

            let mut escape_next = false;
            while pos < chars.len() {
                if escape_next {
                    escape_next = false;
                } else if chars[pos] == '\\' {
                    escape_next = true;
                } else if chars[pos] == '"' {
                    break;
                }
                pos += 1;
            }

            if pos < chars.len() {
                let value: String = chars[value_start..pos].iter().collect();
                return Ok(Some(value));
            }
        } else if chars[pos] == '{' {
            // Handle object value
            let mut brace_count = 1;
            pos += 1;
            let value_start = pos - 1;

            while pos < chars.len() && brace_count > 0 {
                if chars[pos] == '{' {
                    brace_count += 1;
                } else if chars[pos] == '}' {
                    brace_count -= 1;
                }
                pos += 1;
            }

            if brace_count == 0 {
                let value: String = chars[value_start..pos].iter().collect();
                return Ok(Some(value));
            }
        } else if chars[pos] == '[' {
            // Handle array value
            let mut bracket_count = 1;
            pos += 1;
            let value_start = pos - 1;
            let mut in_string = false;
            let mut escape_next = false;

            while pos < chars.len() && bracket_count > 0 {
                if escape_next {
                    escape_next = false;
                } else if chars[pos] == '\\' {
                    escape_next = true;
                } else if chars[pos] == '"' {
                    in_string = !in_string;
                } else if !in_string {
                    if chars[pos] == '[' {
                        bracket_count += 1;
                    } else if chars[pos] == ']' {
                        bracket_count -= 1;
                    }
                }
                pos += 1;
            }

            if bracket_count == 0 {
                let value: String = chars[value_start..pos].iter().collect();
                return Ok(Some(value));
            }
        } else {
            // Non-string value (number, boolean, null)
            let value_start = pos;

            while pos < chars.len()
                && !matches!(chars[pos], ',' | '}' | ']' | ' ' | '\t' | '\n' | '\r')
            {
                pos += 1;
            }

            let value: String = chars[value_start..pos].iter().collect();
            return Ok(Some(value));
        }
    }

    Ok(None)
}
