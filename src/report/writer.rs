use std::fs;
use std::io::Write;

use super::time_utils::{is_leap_year, day_of_year_to_month_day};

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
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH)
        .expect("System clock is set before Unix epoch (1970-01-01)");
    let secs = duration.as_secs();
    
    // Calculate time components
    const SECS_PER_DAY: u64 = 86400;
    const SECS_PER_HOUR: u64 = 3600;
    const SECS_PER_MIN: u64 = 60;
    
    // Days since Unix epoch
    let days = secs / SECS_PER_DAY;
    let remaining = secs % SECS_PER_DAY;
    
    // Time of day
    let hours = remaining / SECS_PER_HOUR;
    let minutes = (remaining % SECS_PER_HOUR) / SECS_PER_MIN;
    let seconds = remaining % SECS_PER_MIN;
    
    // Calculate year, month, day from days since epoch (1970-01-01)
    let mut year = 1970;
    let mut day_of_year = days;
    
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if day_of_year >= days_in_year {
            day_of_year -= days_in_year;
            year += 1;
        } else {
            break;
        }
    }
    
    let (month, day) = day_of_year_to_month_day(day_of_year as u32, is_leap_year(year));
    
    format!("{:04}{:02}{:02}-{:02}{:02}{:02}",
        year, month, day, hours, minutes, seconds)
}
