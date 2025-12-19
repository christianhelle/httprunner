use crate::request_variables;
use anyhow::Result;

pub fn extract_json_value(json_body: &str, json_path: &str) -> Result<Option<String>> {
    if let Some(property) = json_path.strip_prefix("$.") {
        return request_variables::extract_json_property(json_body, property);
    }

    Ok(None)
}
