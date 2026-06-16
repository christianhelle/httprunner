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

pub trait RegexCache: Send + Sync {
    fn get_or_insert_with(
        &self,
        pattern: &str,
        f: Box<dyn FnOnce() -> std::result::Result<Regex, regex::Error> + Send>,
    ) -> std::result::Result<Regex, regex::Error>;
}

pub struct HashMapRegexCache {
    inner: Mutex<HashMap<String, Regex>>,
}

impl Default for HashMapRegexCache {
    fn default() -> Self {
        Self::new()
    }
}

impl HashMapRegexCache {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.inner.lock().expect("regex cache mutex poisoned").len()
    }

    #[cfg(test)]
    pub(crate) fn contains(&self, pattern: &str) -> bool {
        self.inner
            .lock()
            .expect("regex cache mutex poisoned")
            .contains_key(pattern)
    }
}

impl RegexCache for HashMapRegexCache {
    fn get_or_insert_with(
        &self,
        pattern: &str,
        f: Box<dyn FnOnce() -> std::result::Result<Regex, regex::Error> + Send>,
    ) -> std::result::Result<Regex, regex::Error> {
        let mut cache = self.inner.lock().expect("regex cache mutex poisoned");
        if let Some(regex) = cache.get(pattern) {
            return Ok(regex.clone());
        }
        let regex = f()?;
        cache.insert(pattern.to_string(), regex.clone());
        Ok(regex)
    }
}

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

pub(crate) fn get_case_insensitive_regex_with_cache(
    pattern: &str,
    cache: &dyn RegexCache,
) -> std::result::Result<Regex, regex::Error> {
    let owned = pattern.to_string();
    cache.get_or_insert_with(pattern, Box::new(move || {
        RegexBuilder::new(&owned).case_insensitive(true).build()
    }))
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
    fn replace_with_cache(
        &self,
        input: &str,
        cache: &dyn RegexCache,
    ) -> std::result::Result<String, regex::Error> {
        let re = get_case_insensitive_regex_with_cache(self.get_regex(), cache)?;
        Ok(re
            .replace_all(input, |_: &regex::Captures| self.generate())
            .to_string())
    }
}

pub fn substitute_functions(input: &str) -> Result<String> {
    const SUBSTITUTORS: &[&dyn FunctionSubstitutor] = &[
        &GuidSubstitutor {} as &dyn FunctionSubstitutor,
        &StringSubstitutor {} as &dyn FunctionSubstitutor,
        &NumberSubstitutor {} as &dyn FunctionSubstitutor,
        &Base64EncodeSubstitutor {} as &dyn FunctionSubstitutor,
        &UpperSubstitutor {} as &dyn FunctionSubstitutor,
        &LowerSubstitutor {} as &dyn FunctionSubstitutor,
        &NameSubstitutor {} as &dyn FunctionSubstitutor,
        &FirstNameSubstitutor {} as &dyn FunctionSubstitutor,
        &LastNameSubstitutor {} as &dyn FunctionSubstitutor,
        &AddressSubstitutor {} as &dyn FunctionSubstitutor,
        &JobTitleSubstitutor {} as &dyn FunctionSubstitutor,
        &EmailSubstitutor {} as &dyn FunctionSubstitutor,
        &GetDateSubstitutor {} as &dyn FunctionSubstitutor,
        &GetTimeSubstitutor {} as &dyn FunctionSubstitutor,
        &GetDateTimeSubstitutor {} as &dyn FunctionSubstitutor,
        &GetUtcDateTimeSubstitutor {} as &dyn FunctionSubstitutor,
        &LoremIpsumSubstitutor {} as &dyn FunctionSubstitutor,
    ];

    let mut result = input.to_string();
    for substitutor in SUBSTITUTORS {
        result = substitutor.replace(&result)?;
    }

    Ok(result)
}

pub fn substitute_functions_with_cache(
    input: &str,
    cache: &dyn RegexCache,
) -> Result<String> {
    const SUBSTITUTORS: &[&dyn FunctionSubstitutor] = &[
        &GuidSubstitutor {} as &dyn FunctionSubstitutor,
        &StringSubstitutor {} as &dyn FunctionSubstitutor,
        &NumberSubstitutor {} as &dyn FunctionSubstitutor,
        &Base64EncodeSubstitutor {} as &dyn FunctionSubstitutor,
        &UpperSubstitutor {} as &dyn FunctionSubstitutor,
        &LowerSubstitutor {} as &dyn FunctionSubstitutor,
        &NameSubstitutor {} as &dyn FunctionSubstitutor,
        &FirstNameSubstitutor {} as &dyn FunctionSubstitutor,
        &LastNameSubstitutor {} as &dyn FunctionSubstitutor,
        &AddressSubstitutor {} as &dyn FunctionSubstitutor,
        &JobTitleSubstitutor {} as &dyn FunctionSubstitutor,
        &EmailSubstitutor {} as &dyn FunctionSubstitutor,
        &GetDateSubstitutor {} as &dyn FunctionSubstitutor,
        &GetTimeSubstitutor {} as &dyn FunctionSubstitutor,
        &GetDateTimeSubstitutor {} as &dyn FunctionSubstitutor,
        &GetUtcDateTimeSubstitutor {} as &dyn FunctionSubstitutor,
        &LoremIpsumSubstitutor {} as &dyn FunctionSubstitutor,
    ];

    let mut result = input.to_string();
    for substitutor in SUBSTITUTORS {
        result = substitutor.replace_with_cache(&result, cache)?;
    }

    Ok(result)
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
        let cache = HashMapRegexCache::new();
        let substitutor = TestSubstitutor;
        assert_eq!(
            substitutor
                .replace_with_cache("test_function()", &cache)
                .unwrap(),
            "generated"
        );
        assert_eq!(
            substitutor
                .replace_with_cache("test_function()", &cache)
                .unwrap(),
            "generated"
        );

        assert!(cache.contains(substitutor.get_regex()));
    }

    #[test]
    fn independent_caches_do_not_interfere() {
        let cache_a = HashMapRegexCache::new();
        let cache_b = HashMapRegexCache::new();
        let substitutor = TestSubstitutor;

        substitutor
            .replace_with_cache("test_function()", &cache_a)
            .unwrap();

        assert_eq!(cache_a.len(), 1);
        assert_eq!(cache_b.len(), 0);
    }
}
