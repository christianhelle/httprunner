use std::fs::OpenOptions;
use std::io::{self, Write};
use chrono::Utc;
use anyhow::Result;

pub struct Log {
    file: Option<std::fs::File>,
}

impl Log {
    pub fn new(base_filename: Option<&str>) -> Result<Self> {
        let file = if let Some(filename) = base_filename {
            Some(create_log_file(filename)?)
        } else {
            None
        };
        
        Ok(Self { file })
    }

    pub fn write(&mut self, message: &str) {
        // Always print to stdout
        print!("{}", message);
        io::stdout().flush().unwrap_or(());
        
        // Also write to log file if available
        if let Some(ref mut file) = self.file {
            let _ = file.write_all(message.as_bytes());
            let _ = file.flush();
        }
    }

    pub fn writeln(&mut self, message: &str) {
        self.write(message);
        self.write("\n");
    }
}

fn create_log_file(base_filename: &str) -> Result<std::fs::File> {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let log_filename = if base_filename.is_empty() {
        format!("log_{}.log", timestamp)
    } else {
        format!("{}_{}.log", base_filename, timestamp)
    };
    
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_filename)?;
    
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_log_creation() {
        let temp_dir = tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let mut log = Log::new(Some("test")).unwrap();
        log.writeln("Test message");
        
        // Check that log file was created
        let entries = fs::read_dir(".").unwrap();
        let log_files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_str().unwrap_or("").starts_with("test_"))
            .collect();
        
        assert_eq!(log_files.len(), 1);
    }

    #[test]
    fn test_log_without_file() {
        let mut log = Log::new(None).unwrap();
        log.writeln("Test message"); // Should not panic
    }
}