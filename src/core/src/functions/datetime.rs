use super::substitution::FunctionSubstitutor;

pub struct GetDateTimeSubstitutor {}
impl FunctionSubstitutor for GetDateTimeSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bgetdatetime\(\)"
    }

    fn generate(&self) -> String {
        use chrono::prelude::*;
        let local: DateTime<Local> = Local::now();
        local.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_getdatetime() {
        let sub = GetDateTimeSubstitutor {};
        let datetime = sub.generate();

        let datetime_pattern = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").unwrap();
        assert!(
            datetime_pattern.is_match(&datetime),
            "DateTime '{}' does not match pattern YYYY-MM-DD HH:MM:SS",
            datetime
        );

        use chrono::NaiveDateTime;
        assert!(
            NaiveDateTime::parse_from_str(&datetime, "%Y-%m-%d %H:%M:%S").is_ok(),
            "DateTime '{}' could not be parsed",
            datetime
        );
    }
}
