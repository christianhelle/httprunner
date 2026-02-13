use super::substitution::FunctionSubstitutor;

pub struct GetDateSubstitutor {}
impl FunctionSubstitutor for GetDateSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bgetdate\(\)"
    }

    fn generate(&self) -> String {
        use chrono::prelude::*;
        let local: DateTime<Local> = Local::now();
        local.format("%Y-%m-%d").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_getdate() {
        let sub = GetDateSubstitutor {};
        let date = sub.generate();

        let date_pattern = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
        assert!(
            date_pattern.is_match(&date),
            "Date '{}' does not match pattern YYYY-MM-DD",
            date
        );

        use chrono::NaiveDate;
        assert!(
            NaiveDate::parse_from_str(&date, "%Y-%m-%d").is_ok(),
            "Date '{}' could not be parsed",
            date
        );
    }
}
