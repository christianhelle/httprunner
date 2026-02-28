use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::ProcessorResults;

pub fn export_json(results: &ProcessorResults) -> Result<String, std::io::Error> {
    export_json_to_dir(results, None)
}

pub fn export_json_to_dir(
    results: &ProcessorResults,
    output_dir: Option<&Path>,
) -> Result<String, std::io::Error> {
    let json = serde_json::to_string_pretty(results)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is before UNIX epoch")
        .as_secs();
    let filename = format!("httprunner_results_{}.json", timestamp);

    let filepath = match output_dir {
        Some(dir) => dir.join(&filename),
        None => Path::new(&filename).to_path_buf(),
    };

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&filepath)?;

    file.write_all(json.as_bytes())?;

    Ok(filename)
}
