use super::time_utils::*;

#[test]
fn test_is_leap_year_divisible_by_4_not_100() {
    assert!(is_leap_year(2024));
    assert!(is_leap_year(2028));
}

#[test]
fn test_is_leap_year_divisible_by_400() {
    assert!(is_leap_year(2000));
    assert!(is_leap_year(2400));
}

#[test]
fn test_is_leap_year_divisible_by_100_not_400() {
    assert!(!is_leap_year(1900));
    assert!(!is_leap_year(2100));
}

#[test]
fn test_is_leap_year_not_divisible_by_4() {
    assert!(!is_leap_year(2021));
    assert!(!is_leap_year(2022));
    assert!(!is_leap_year(2023));
}

#[test]
fn test_day_of_year_to_month_day_leap_year() {
    // Jan 1
    assert_eq!(day_of_year_to_month_day(0, true), (1, 1));
    // Jan 31
    assert_eq!(day_of_year_to_month_day(30, true), (1, 31));
    // Feb 1
    assert_eq!(day_of_year_to_month_day(31, true), (2, 1));
    // Feb 29 (leap day)
    assert_eq!(day_of_year_to_month_day(59, true), (2, 29));
    // Mar 1
    assert_eq!(day_of_year_to_month_day(60, true), (3, 1));
}

#[test]
fn test_day_of_year_to_month_day_non_leap_year() {
    // Jan 1
    assert_eq!(day_of_year_to_month_day(0, false), (1, 1));
    // Jan 31
    assert_eq!(day_of_year_to_month_day(30, false), (1, 31));
    // Feb 1
    assert_eq!(day_of_year_to_month_day(31, false), (2, 1));
    // Feb 28
    assert_eq!(day_of_year_to_month_day(58, false), (2, 28));
    // Mar 1
    assert_eq!(day_of_year_to_month_day(59, false), (3, 1));
}

#[test]
fn test_day_of_year_to_month_day_year_end() {
    // Dec 31 in leap year (day 365, 0-indexed)
    assert_eq!(day_of_year_to_month_day(365, true), (12, 31));
    // Dec 31 in non-leap year (day 364, 0-indexed)
    assert_eq!(day_of_year_to_month_day(364, false), (12, 31));
}
