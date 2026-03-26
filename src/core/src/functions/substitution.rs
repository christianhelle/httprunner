use crate::functions::address::AddressSubstitutor;
use crate::functions::base64_encode::Base64EncodeSubstitutor;
use crate::functions::date::GetDateSubstitutor;
use crate::functions::datetime::GetDateTimeSubstitutor;
use crate::functions::email::EmailSubstitutor;
use crate::functions::first_name::FirstNameSubstitutor;
use crate::functions::guid::GuidSubstitutor;
use crate::functions::job_title::JobTitleSubstitutor;
use crate::functions::last_name::LastNameSubstitutor;
use crate::functions::lorem_ipsum::LoremIpsumSubstitutor;
use crate::functions::lower::LowerSubstitutor;
use crate::functions::name::NameSubstitutor;
use crate::functions::number::NumberSubstitutor;
use crate::functions::string_gen::StringSubstitutor;
use crate::functions::time::GetTimeSubstitutor;
use crate::functions::upper::UpperSubstitutor;
use crate::functions::utc_datetime::GetUtcDateTimeSubstitutor;
use anyhow::Result;
use regex::{Regex, RegexBuilder};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static REGEX_CACHE: OnceLock<Mutex<HashMap<String, Regex>>> = OnceLock::new();

pub(crate) fn get_case_insensitive_regex(
    pattern: &str,
) -> std::result::Result<Regex, regex::Error> {
    let cache = REGEX_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache.lock().expect("regex cache mutex poisoned");

    if let Some(regex) = cache.get(pattern) {
        return Ok(regex.clone());
    }

    let regex = RegexBuilder::new(pattern).case_insensitive(true).build()?;
    cache.insert(pattern.to_string(), regex.clone());

    Ok(regex)
}

pub trait FunctionSubstitutor: Sync {
    fn get_regex(&self) -> &str;
    fn generate(&self) -> String;
    fn replace(&self, input: &str) -> std::result::Result<String, regex::Error> {
        let re = get_case_insensitive_regex(self.get_regex())?;
        Ok(re
            .replace_all(input, |_: &regex::Captures| self.generate())
            .to_string())
    }
}

pub fn substitute_functions(input: &str) -> Result<String> {
    static SUBSTITUTORS: &[&dyn FunctionSubstitutor] = &[
        &GuidSubstitutor {},
        &StringSubstitutor {},
        &NumberSubstitutor {},
        &Base64EncodeSubstitutor {},
        &UpperSubstitutor {},
        &LowerSubstitutor {},
        &NameSubstitutor {},
        &FirstNameSubstitutor {},
        &LastNameSubstitutor {},
        &AddressSubstitutor {},
        &JobTitleSubstitutor {},
        &EmailSubstitutor {},
        &GetDateSubstitutor {},
        &GetTimeSubstitutor {},
        &GetDateTimeSubstitutor {},
        &GetUtcDateTimeSubstitutor {},
        &LoremIpsumSubstitutor {},
    ];

    let mut result = input.to_string();
    for substitutor in SUBSTITUTORS {
        result = substitutor.replace(&result)?;
    }

    Ok(result)
}

#[cfg(test)]
fn regex_cache_size() -> usize {
    REGEX_CACHE
        .get()
        .map(|cache| cache.lock().expect("regex cache mutex poisoned").len())
        .unwrap_or(0)
}

#[cfg(test)]
fn regex_cache_contains(pattern: &str) -> bool {
    REGEX_CACHE
        .get()
        .map(|cache| {
            cache
                .lock()
                .expect("regex cache mutex poisoned")
                .contains_key(pattern)
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSubstitutor;

    impl FunctionSubstitutor for TestSubstitutor {
        fn get_regex(&self) -> &str {
            r"\btest_function\(\)"
        }

        fn generate(&self) -> String {
            "generated".to_string()
        }
    }

    #[test]
    fn replace_reuses_cached_regex_for_same_pattern() {
        let substitutor = TestSubstitutor;
        let initial_size = regex_cache_size();
        assert_eq!(substitutor.replace("test_function()").unwrap(), "generated");
        assert_eq!(substitutor.replace("test_function()").unwrap(), "generated");

        assert!(regex_cache_contains(substitutor.get_regex()));
        assert!(regex_cache_size() >= initial_size);
    }
}
