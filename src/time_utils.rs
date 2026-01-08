//! Utilities for working with time without external dependencies.
//!
//! This module provides functions for converting UNIX timestamps to human-readable
//! date/time strings without relying on external crates like chrono.

use std::time::SystemTime;

/// Converts days since UNIX epoch to (year, month, day).
pub fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    let mut year = 1970;
    let mut remaining_days = days;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let days_in_months = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for &days_in_month in &days_in_months {
        if remaining_days < days_in_month as u64 {
            break;
        }
        remaining_days -= days_in_month as u64;
        month += 1;
    }

    (year, month, remaining_days + 1)
}

/// Checks if a year is a leap year.
pub fn is_leap_year(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

/// Formats the current UTC time as "YYYY-MM-DD HH:MM:SS UTC".
pub fn format_utc_datetime() -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time before UNIX EPOCH");

    let secs = now.as_secs();
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    let (year, month, day) = days_to_ymd(days);

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
        year, month, day, hours, minutes, seconds
    )
}

/// Formats the current UTC time as a compact timestamp "YYYYMMDD-HHMMSS".
/// Useful for filenames.
pub fn format_compact_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time before UNIX EPOCH");

    let secs = now.as_secs();
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    let (year, month, day) = days_to_ymd(days);

    format!(
        "{:04}{:02}{:02}-{:02}{:02}{:02}",
        year, month, day, hours, minutes, seconds
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2000)); // Divisible by 400
        assert!(is_leap_year(2004)); // Divisible by 4 but not 100
        assert!(!is_leap_year(1900)); // Divisible by 100 but not 400
        assert!(!is_leap_year(2001)); // Not divisible by 4
    }

    #[test]
    fn test_days_to_ymd_epoch() {
        // Day 0 is 1970-01-01
        assert_eq!(days_to_ymd(0), (1970, 1, 1));
    }

    #[test]
    fn test_days_to_ymd_year_boundary() {
        // Day 365 is 1971-01-01 (1970 was not a leap year)
        assert_eq!(days_to_ymd(365), (1971, 1, 1));
    }

    #[test]
    fn test_days_to_ymd_leap_year() {
        // Feb 29, 2000
        let days_since_epoch = (2000 - 1970) * 365 + 7 + 31 + 29 - 1; // Account for leap years
        let (year, month, day) = days_to_ymd(days_since_epoch);
        assert_eq!(year, 2000);
        assert_eq!(month, 2);
        assert_eq!(day, 29);
    }

    #[test]
    fn test_format_utc_datetime_format() {
        let result = format_utc_datetime();
        // Check format: YYYY-MM-DD HH:MM:SS UTC
        assert!(result.ends_with(" UTC"));
        assert_eq!(result.len(), 23); // "2024-01-01 00:00:00 UTC"
    }

    #[test]
    fn test_format_compact_timestamp_format() {
        let result = format_compact_timestamp();
        // Check format: YYYYMMDD-HHMMSS
        assert_eq!(result.len(), 15); // "20240101-000000"
        assert!(result.contains('-'));
    }
}
