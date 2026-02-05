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
