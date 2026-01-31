pub fn sanitize_error_message(message: &str) -> String {
    let mut sanitized = message.to_string();

    if let Ok(url_re) = regex::Regex::new(r"https?://[^\s]+") {
        sanitized = url_re.replace_all(&sanitized, "[URL]").to_string();
    }

    let path_patterns = [
        r#"[A-Za-z]:\\[^\s:*?"<>|]+"#,
        r"~[/\\][^\s]*",
        r"(/[a-zA-Z0-9_\-./]+)+",
    ];

    for pattern in path_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            sanitized = re.replace_all(&sanitized, "[PATH]").to_string();
        }
    }

    if let Ok(ip_re) = regex::Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b") {
        sanitized = ip_re.replace_all(&sanitized, "[IP]").to_string();
    }

    if let Ok(email_re) = regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")
    {
        sanitized = email_re.replace_all(&sanitized, "[EMAIL]").to_string();
    }

    if sanitized.len() > 500 {
        sanitized.truncate(500);
        sanitized.push_str("...[TRUNCATED]");
    }

    sanitized
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_error_type_name(error: &dyn std::error::Error) -> String {
    let full_type = std::any::type_name_of_val(error);
    full_type
        .rsplit("::")
        .next()
        .unwrap_or("Unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_error_message_removes_unix_paths() {
        let msg = "Failed to open /home/user/secret/file.txt";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("/home/user"));
        assert!(sanitized.contains("[PATH]"));
    }

    #[test]
    fn test_sanitize_error_message_removes_windows_paths() {
        let msg = "Failed to open C:\\Users\\Admin\\Documents\\secret.txt";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("C:\\Users"));
        assert!(sanitized.contains("[PATH]"));
    }

    #[test]
    fn test_sanitize_error_message_removes_urls() {
        let msg = "Failed to connect to https://api.secret-server.com/v1/data";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("https://"));
        assert!(sanitized.contains("[URL]"));
    }

    #[test]
    fn test_sanitize_error_message_removes_ip_addresses() {
        let msg = "Connection refused to 192.168.1.100:8080";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("192.168.1.100"));
        assert!(sanitized.contains("[IP]"));
    }

    #[test]
    fn test_sanitize_error_message_removes_emails() {
        let msg = "Invalid auth for user@example.com";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("user@example.com"));
        assert!(sanitized.contains("[EMAIL]"));
    }

    #[test]
    fn test_sanitize_error_message_truncates_long_messages() {
        let msg = "x".repeat(1000);
        let sanitized = sanitize_error_message(&msg);
        assert!(sanitized.len() < 600);
        assert!(sanitized.ends_with("[TRUNCATED]"));
    }
}
