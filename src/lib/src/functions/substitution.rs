use crate::functions::generators::{
    AddressSubstitutor, Base64EncodeSubstitutor, FirstNameSubstitutor, GuidSubstitutor,
    JobTitleSubstitutor, LastNameSubstitutor, NameSubstitutor, NumberSubstitutor,
    StringSubstitutor,
};
use anyhow::Result;
use regex::RegexBuilder;

pub trait FunctionSubstitutor {
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
    let substitutors: &[&dyn FunctionSubstitutor] = &[
        &GuidSubstitutor {},
        &StringSubstitutor {},
        &NumberSubstitutor {},
        &Base64EncodeSubstitutor {},
        &NameSubstitutor {},
        &FirstNameSubstitutor {},
        &LastNameSubstitutor {},
        &AddressSubstitutor {},
        &JobTitleSubstitutor {},
    ];

    let mut result = input.to_string();
    for substitutor in substitutors {
        result = substitutor.replace(&result)?;
    }

    Ok(result)
}
