use crate::functions::{substitution::FunctionSubstitutor, values};
use std::result::Result;

pub struct GuidSubstitutor {}
impl FunctionSubstitutor for GuidSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bguid\(\)"
    }

    fn generate(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 16];
        rng.fill(&mut bytes);
        format!(
            "{:08x}{:04x}{:04x}{:04x}{:012x}",
            u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            u16::from_be_bytes([bytes[4], bytes[5]]),
            (u16::from_be_bytes([bytes[6], bytes[7]]) & 0x0fff) | 0x4000,
            (u16::from_be_bytes([bytes[8], bytes[9]]) & 0x3fff) | 0x8000,
            u64::from_be_bytes([
                0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
            ]) & 0xffffffffffff
        )
    }
}

pub struct StringSubstitutor {}
impl FunctionSubstitutor for StringSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bstring\(\)"
    }

    fn generate(&self) -> String {
        use rand::Rng;
        use rand::distributions::Alphanumeric;

        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(20)
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

        rand::thread_rng().gen_range(0..=100).to_string()
    }
}

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

pub struct NameSubstitutor {}
impl FunctionSubstitutor for NameSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bname\(\)"
    }

    fn generate(&self) -> String {
        let first_name = FirstNameSubstitutor {}.generate();
        let last_name = LastNameSubstitutor {}.generate();
        format!("{} {}", first_name, last_name).to_string()
    }
}

pub struct FirstNameSubstitutor {}
impl FunctionSubstitutor for FirstNameSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bfirst_name\(\)"
    }

    fn generate(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..values::FIRST_NAMES.len());
        values::FIRST_NAMES[index].to_string()
    }
}

pub struct LastNameSubstitutor {}
impl FunctionSubstitutor for LastNameSubstitutor {
    fn get_regex(&self) -> &str {
        r"\blast_name\(\)"
    }

    fn generate(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..values::LAST_NAMES.len());
        values::LAST_NAMES[index].to_string()
    }
}

pub struct AddressSubstitutor {}
impl FunctionSubstitutor for AddressSubstitutor {
    fn get_regex(&self) -> &str {
        r"\baddress\(\)"
    }

    fn generate(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..values::ADDRESSES.len());
        values::ADDRESSES[index].to_string()
    }
}
