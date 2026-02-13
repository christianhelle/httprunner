use super::substitution::FunctionSubstitutor;

pub struct GetUtcDateTimeSubstitutor {}
impl FunctionSubstitutor for GetUtcDateTimeSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bgetutcdatetime\(\)"
    }

    fn generate(&self) -> String {
        use chrono::prelude::*;
        let utc: DateTime<Utc> = Utc::now();
        utc.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_getutcdatetime() {
        let sub = GetUtcDateTimeSubstitutor {};
        let utc_datetime = sub.generate();

        let datetime_pattern = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").unwrap();
        assert!(
            datetime_pattern.is_match(&utc_datetime),
            "UTC DateTime '{}' does not match pattern YYYY-MM-DD HH:MM:SS",
            utc_datetime
        );

        use chrono::NaiveDateTime;
        assert!(
            NaiveDateTime::parse_from_str(&utc_datetime, "%Y-%m-%d %H:%M:%S").is_ok(),
            "UTC DateTime '{}' could not be parsed",
            utc_datetime
        );
    }
}
