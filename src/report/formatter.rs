pub fn escape_markdown(s: &str) -> String {
    s.replace('|', "\\|")
}

/// Formats the current local datetime as a string in the format: YYYY-MM-DD HH:MM:SS
/// Uses Local time to match the filename timestamp format.
pub fn format_local_datetime() -> String {
    use chrono::Local;
    let now = Local::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}
