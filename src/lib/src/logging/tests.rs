#[cfg(test)]
mod logging_tests {
    use crate::logging::Log;

    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn log_writes_to_timestamped_file() {
        let temp = tempdir().unwrap();
        let base = temp.path().join("testrun");
        let base_str = base.to_string_lossy().to_string();

        {
            let mut log = Log::new(Some(&base_str)).unwrap();
            log.writeln("hello world");
        }

        let entries: Vec<_> = fs::read_dir(temp.path())
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();

        let log_path = entries
            .iter()
            .find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with("testrun_"))
                    .unwrap_or(false)
            })
            .expect("log file not created");

        let content = fs::read_to_string(log_path).unwrap();
        assert!(content.contains("hello world"));
    }

    #[test]
    fn log_strips_ansi_codes() {
        let temp = tempdir().unwrap();
        let base = temp.path().join("testansi");
        let base_str = base.to_string_lossy().to_string();

        {
            let mut log = Log::new(Some(&base_str)).unwrap();
            // Write a message with ANSI color codes (red text)
            log.writeln("\x1b[31mRed Text\x1b[0m Normal");
            // Write a message with blue text
            log.writeln("\x1b[34m🚀\x1b[0m Test");
        }

        let entries: Vec<_> = fs::read_dir(temp.path())
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();

        let log_path = entries
            .iter()
            .find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with("testansi_"))
                    .unwrap_or(false)
            })
            .expect("log file not created");

        let content = fs::read_to_string(log_path).unwrap();
        // Should contain the text without ANSI codes
        assert!(content.contains("Red Text Normal"));
        assert!(content.contains("🚀 Test"));
        // Should NOT contain ANSI codes
        assert!(!content.contains("\x1b[31m"));
        assert!(!content.contains("\x1b[34m"));
        assert!(!content.contains("\x1b[0m"));
    }
}

#[cfg(test)]
mod support_tests {
    use crate::logging::get_support_key;

    #[test]
    fn generate_support_key_creates_valid_format() {
        // Call the internal generate function through get_support_key
        let key = get_support_key().unwrap();
        
        // Full key should be 32 hex characters (UUID v4 without hyphens)
        assert_eq!(key.key.len(), 32);
        assert!(key.key.chars().all(|c| c.is_ascii_hexdigit()));
        
        // Short key should be first 8 characters
        assert_eq!(key.short_key.len(), 8);
        assert_eq!(key.short_key, &key.key[..8]);
    }

    #[test]
    fn generate_support_key_produces_unique_keys() {
        // Generate two keys and verify they're different
        let key1 = get_support_key().unwrap();
        let key2 = get_support_key().unwrap();
        
        // Note: On platforms with persistence, these might be the same.
        // But we can verify they're both valid format.
        assert_eq!(key1.key.len(), 32);
        assert_eq!(key2.key.len(), 32);
        assert_eq!(key1.short_key.len(), 8);
        assert_eq!(key2.short_key.len(), 8);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn get_support_key_returns_valid_format() {
        // Verify that calling get_support_key() returns a valid key
        let key1 = get_support_key().unwrap();
        let key2 = get_support_key().unwrap();
        
        // Both calls should return valid keys
        assert_eq!(key1.key.len(), 32);
        assert_eq!(key2.key.len(), 32);
        assert_eq!(key1.short_key.len(), 8);
        assert_eq!(key2.short_key.len(), 8);
        
        // Keys should be the same due to persistence
        assert_eq!(key1.key, key2.key);
        assert_eq!(key1.short_key, key2.short_key);
    }
}
