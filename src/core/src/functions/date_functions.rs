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
