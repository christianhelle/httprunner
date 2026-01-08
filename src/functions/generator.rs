use crate::functions::substitution::FunctionSubstitutor;
use regex::RegexBuilder;

pub struct GuidSubstitutor {}
impl FunctionSubstitutor for GuidSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bguid\(\)"
    }

    fn generate(&self) -> String {
        use uuid::Uuid;
        Uuid::new_v4().as_simple().to_string()
    }
}

pub struct StringSubstitutor {}
impl FunctionSubstitutor for StringSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bstring\(\)"
    }

    fn generate(&self) -> String {
        use rand::Rng;
        use rand::distr::Alphanumeric;

        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect()
    }
}

pub struct NumberSubstitutor {}
impl FunctionSubstitutor for NumberSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bnumber\(\)"
    }

    fn generate(&self) -> String {
        use rand::Rng;
        use rand::distr::Uniform;

        rand::rng()
            .sample(Uniform::new_inclusive(0, 100).unwrap())
            .to_string()
    }
}

pub struct Base64EncodeSubstitutor {}
impl FunctionSubstitutor for Base64EncodeSubstitutor {
    fn get_regex(&self) -> &str {
        panic!("Base64Substitutor.get_regex should not be called directly");
    }

    fn generate(&self) -> String {
        panic!("Base64Substitutor.generate should not be called directly");
    }

    fn replace(&self, input: &String) -> String {
        use base64::Engine;
        use base64::engine::general_purpose;

        let re = RegexBuilder::new(r"\bbase64_encode\(\s*'([^']*)'\s*\)")
            .case_insensitive(true)
            .build()
            .unwrap();
        re.replace_all(input, |caps: &regex::Captures| {
            let to_encode = &caps[1];
            general_purpose::STANDARD.encode(to_encode)
        })
        .to_string()
    }
}
