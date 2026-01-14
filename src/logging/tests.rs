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
}
