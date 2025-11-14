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
