pub fn format_json_if_valid(text: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(json) => match serde_json::to_string_pretty(&json) {
            Ok(pretty) => pretty,
            Err(_) => text.to_string(),
        },
        Err(_) => text.to_string(),
    }
}

pub fn format_request_name(name: &Option<String>) -> String {
    name.as_ref()
        .map(|n| format!("{}: ", n))
        .unwrap_or_default()
}
