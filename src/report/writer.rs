use std::fs;
use std::io::Write;

#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(test)]
static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn write_report(content: String) -> Result<String, std::io::Error> {
    write_report_with_extension(content, "md")
}

pub fn write_report_with_extension(
    content: String,
    extension: &str,
) -> Result<String, std::io::Error> {
    let timestamp = format_local_timestamp();
    #[cfg(test)]
    let filename = {
        let count = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        format!(
            "httprunner-report-{}-{}-{}.{}",
            timestamp,
            std::process::id(),
            count,
            extension
        )
    };
    #[cfg(not(test))]
    let filename = format!("httprunner-report-{}.{}", timestamp, extension);

    let mut file = fs::File::create(&filename)?;
    file.write_all(content.as_bytes())?;

    Ok(filename)
}

fn format_local_timestamp() -> String {
    use chrono::Local;
    let now = Local::now();
    now.format("%Y%m%d-%H%M%S").to_string()
}
