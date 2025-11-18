use anyhow::Result;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Log {
    log_file: Option<File>,
}

impl Log {
    pub fn new(base_filename: Option<&str>) -> Result<Self> {
        let log_file = if let Some(filename) = base_filename {
            Some(create_log_file(filename)?)
        } else {
            None
        };

        Ok(Log { log_file })
    }

    pub fn writeln(&mut self, message: &str) {
        println!("{}", message);
        if let Some(ref mut file) = self.log_file {
            let _ = writeln!(file, "{}", message);
        }
    }
}

fn create_log_file(base_filename: &str) -> Result<File> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let log_filename = format!("{}_{}.log", base_filename, timestamp);

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_filename)?;

    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
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
