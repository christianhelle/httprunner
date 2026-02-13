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
use regex::RegexBuilder;

pub trait FunctionSubstitutor: Sync {
    fn get_regex(&self) -> &str;
    fn generate(&self) -> String;
    fn replace(&self, input: &str) -> std::result::Result<String, regex::Error> {
        let re = RegexBuilder::new(self.get_regex())
            .case_insensitive(true)
            .build()?;
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
