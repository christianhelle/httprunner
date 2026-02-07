use super::time_utils::DateTimeComponents;
use crate::types::ProcessorResults;

pub fn escape_markdown(s: &str) -> String {
    s.replace('|', "\\|")
}

pub struct ReportSummary {
    pub total_success: u32,
    pub total_failed: u32,
    pub total_skipped: u32,
    pub total_requests: u32,
    pub success_rate: f64,
}

impl ReportSummary {
    pub fn from_results(results: &ProcessorResults) -> Self {
        let total_success: u32 = results.files.iter().map(|f| f.success_count).sum();
        let total_failed: u32 = results.files.iter().map(|f| f.failed_count).sum();
        let total_skipped: u32 = results.files.iter().map(|f| f.skipped_count).sum();
        let total_requests = total_success + total_failed + total_skipped;
        let success_rate = if total_requests > 0 {
            (total_success as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        Self {
            total_success,
            total_failed,
            total_skipped,
            total_requests,
            success_rate,
        }
    }
}

/// Formats the current local datetime as a string in the format: YYYY-MM-DD HH:MM:SS
/// Uses system time to format timestamps.
pub fn format_local_datetime() -> String {
    let dt = DateTimeComponents::now();
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        dt.year, dt.month, dt.day, dt.hours, dt.minutes, dt.seconds
    )
}
