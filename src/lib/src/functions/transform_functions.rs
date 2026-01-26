use crate::functions::substitution::FunctionSubstitutor;

pub struct Base64EncodeSubstitutor {}
impl FunctionSubstitutor for Base64EncodeSubstitutor {
    fn get_regex(&self) -> &str {
        r"(?!)"
    }

    fn generate(&self) -> String {
        String::new()
    }

    fn replace(&self, input: &str) -> Result<String, regex::Error> {
        use base64::Engine;
        use base64::engine::general_purpose;
        use regex::RegexBuilder;

        let re = RegexBuilder::new(r"\bbase64_encode\(\s*'((?:[^'\\]|\\.)*)'\s*\)")
            .case_insensitive(true)
            .build()?;
        Ok(re
            .replace_all(input, |caps: &regex::Captures| {
                let to_encode = &caps[1];
                general_purpose::STANDARD.encode(to_encode)
            })
            .to_string())
    }
}

pub struct UpperSubstitutor {}
impl FunctionSubstitutor for UpperSubstitutor {
    fn get_regex(&self) -> &str {
        r"(?!)"
    }

    fn generate(&self) -> String {
        String::new()
    }

    fn replace(&self, input: &str) -> Result<String, regex::Error> {
        use regex::RegexBuilder;

        let re = RegexBuilder::new(r"\bupper\(\s*'((?:[^'\\]|\\.)*)'\s*\)")
            .case_insensitive(true)
            .build()?;
        Ok(re
            .replace_all(input, |caps: &regex::Captures| (caps[1]).to_uppercase())
            .to_string())
    }
}

pub struct LowerSubstitutor {}
impl FunctionSubstitutor for LowerSubstitutor {
    fn get_regex(&self) -> &str {
        r"(?!)"
    }

    fn generate(&self) -> String {
        String::new()
    }

    fn replace(&self, input: &str) -> Result<String, regex::Error> {
        use regex::RegexBuilder;

        let re = RegexBuilder::new(r"\blower\(\s*'((?:[^'\\]|\\.)*)'\s*\)")
            .case_insensitive(true)
            .build()?;
        Ok(re
            .replace_all(input, |caps: &regex::Captures| caps[1].to_lowercase())
            .to_string())
    }
}
