use crate::functions::date_functions::{
    GetDateSubstitutor, GetDateTimeSubstitutor, GetTimeSubstitutor, GetUtcDateTimeSubstitutor,
};
use crate::functions::generator_functions::{
    AddressSubstitutor, EmailSubstitutor, FirstNameSubstitutor, GuidSubstitutor,
    JobTitleSubstitutor, LastNameSubstitutor, NameSubstitutor, NumberSubstitutor,
    StringSubstitutor, LoremIpsumSubstitutor
};
use crate::functions::transform_functions::{
    Base64EncodeSubstitutor, LowerSubstitutor, UpperSubstitutor,
};
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
