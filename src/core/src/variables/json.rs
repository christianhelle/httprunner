use anyhow::{Result, anyhow};
use serde_json::Value;

enum JsonPathSegment {
    Key(String),
    Index(usize),
}

pub fn extract_json_property(json_body: &str, property: &str) -> Result<Option<String>> {
    if property.trim().is_empty() {
        return Ok(None);
    }

    let root: Value =
        serde_json::from_str(json_body).map_err(|e| anyhow!("Invalid JSON body: {e}"))?;
    let path = parse_json_path(property)?;
    let Some(value) = find_json_value(&root, &path)? else {
        return Ok(None);
    };

    Ok(Some(format_json_value(value)))
}

fn parse_json_path(property: &str) -> Result<Vec<JsonPathSegment>> {
    let mut segments = Vec::new();

    for raw_part in property.split('.') {
        if raw_part.is_empty() {
            return Err(anyhow!("Invalid JSON property path: {property}"));
        }

        let mut part = raw_part;
        if !part.starts_with('[') {
            let key_end = part.find('[').unwrap_or(part.len());
            let key = &part[..key_end];
            if key.is_empty() {
                return Err(anyhow!("Invalid JSON property path: {property}"));
            }
            segments.push(JsonPathSegment::Key(key.to_string()));
            part = &part[key_end..];
        }

        while !part.is_empty() {
            if !part.starts_with('[') {
                return Err(anyhow!("Invalid JSON property path: {property}"));
            }

            let Some(end_bracket) = part.find(']') else {
                return Err(anyhow!("Invalid array index: {part}"));
            };

            let index_str = &part[1..end_bracket];
            let index = index_str
                .parse::<usize>()
                .map_err(|_| anyhow!("Invalid array index: [{index_str}]"))?;
            segments.push(JsonPathSegment::Index(index));
            part = &part[end_bracket + 1..];
        }
    }

    Ok(segments)
}

fn find_json_value<'a>(root: &'a Value, path: &[JsonPathSegment]) -> Result<Option<&'a Value>> {
    let mut current = root;

    for segment in path {
        match segment {
            JsonPathSegment::Key(key) => match current {
                Value::Object(map) => {
                    let Some(next) = map.get(key) else {
                        return Ok(None);
                    };
                    current = next;
                }
                _ => return Ok(None),
            },
            JsonPathSegment::Index(index) => match current {
                Value::Array(items) => {
                    let Some(next) = items.get(*index) else {
                        return Ok(None);
                    };
                    current = next;
                }
                _ => {
                    return Err(anyhow!(
                        "Expected array but got: {}",
                        format_json_value(current)
                    ));
                }
            },
        }
    }

    Ok(Some(current))
}

fn format_json_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::String(text) => text.clone(),
        _ => value.to_string(),
    }
}
