/// Utility functions for date/time calculations without external dependencies

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
