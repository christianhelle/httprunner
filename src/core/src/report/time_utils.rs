pub fn is_leap_year(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

pub fn day_of_year_to_month_day(day_of_year: u32, is_leap: bool) -> (u32, u32) {
    let days_in_months = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut remaining_days = day_of_year + 1; // Convert 0-indexed to 1-indexed
    for (i, &days) in days_in_months.iter().enumerate() {
        if remaining_days <= days {
            return ((i + 1) as u32, remaining_days);
        }
        remaining_days -= days;
    }
    (12, 31) // Fallback
}

pub struct DateTimeComponents {
    pub year: u64,
    pub month: u32,
    pub day: u32,
    pub hours: u64,
    pub minutes: u64,
    pub seconds: u64,
}

impl DateTimeComponents {
    pub fn now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now();
        let duration = now
            .duration_since(UNIX_EPOCH)
            .expect("System clock is set before Unix epoch (1970-01-01)");
        let secs = duration.as_secs();

        const SECS_PER_DAY: u64 = 86400;
        const SECS_PER_HOUR: u64 = 3600;
        const SECS_PER_MIN: u64 = 60;

        let days = secs / SECS_PER_DAY;
        let remaining = secs % SECS_PER_DAY;

        let hours = remaining / SECS_PER_HOUR;
        let minutes = (remaining % SECS_PER_HOUR) / SECS_PER_MIN;
        let seconds = remaining % SECS_PER_MIN;

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

        Self {
            year,
            month,
            day,
            hours,
            minutes,
            seconds,
        }
    }
}
