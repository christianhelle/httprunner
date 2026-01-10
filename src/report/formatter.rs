use super::time_utils::{is_leap_year, day_of_year_to_month_day};

pub fn escape_markdown(s: &str) -> String {
    s.replace('|', "\\|")
}

/// Formats the current local datetime as a string in the format: YYYY-MM-DD HH:MM:SS
/// Uses system time to format timestamps.
pub fn format_local_datetime() -> String {
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
    
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds)
}
