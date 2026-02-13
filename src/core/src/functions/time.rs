use super::substitution::FunctionSubstitutor;

pub struct GetTimeSubstitutor {}
impl FunctionSubstitutor for GetTimeSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bgettime\(\)"
    }

    fn generate(&self) -> String {
        use chrono::prelude::*;
        let local: DateTime<Local> = Local::now();
        local.format("%H:%M:%S").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_gettime() {
        let sub = GetTimeSubstitutor {};
        let time = sub.generate();

        let time_pattern = Regex::new(r"^\d{2}:\d{2}:\d{2}$").unwrap();
        assert!(
            time_pattern.is_match(&time),
            "Time '{}' does not match pattern HH:MM:SS",
            time
        );

        use chrono::NaiveTime;
        assert!(
            NaiveTime::parse_from_str(&time, "%H:%M:%S").is_ok(),
            "Time '{}' could not be parsed",
            time
        );
    }
}
