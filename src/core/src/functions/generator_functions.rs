use crate::functions::values::LOREM_IPSUM_WORDS;
use crate::functions::{substitution::FunctionSubstitutor, values};

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

pub struct JobTitleSubstitutor {}
impl FunctionSubstitutor for JobTitleSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bjob_title\(\)"
    }

    fn generate(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..values::JOB_TITLES.len());
        values::JOB_TITLES[index].to_string()
    }
}

pub struct EmailSubstitutor {}
impl FunctionSubstitutor for EmailSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bemail\(\)"
    }

    fn generate(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..values::EMAIL_DOMAINS.len());
        let domain = values::EMAIL_DOMAINS[index].to_string();
        let first_name = FirstNameSubstitutor {}.generate();
        let last_name = LastNameSubstitutor {}.generate();

        format!(
            "{}.{}@{}",
            normalize_name_for_email(first_name),
            normalize_name_for_email(last_name),
            domain
        )
        .to_string()
    }
}

fn normalize_name_for_email(name: String) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

pub struct LoremIpsumSubstitutor {}
impl FunctionSubstitutor for LoremIpsumSubstitutor {
    fn get_regex(&self) -> &str {
        r"(?!)"
    }

    fn generate(&self) -> String {
        String::new()
    }

    fn replace(&self, input: &str) -> Result<String, regex::Error> {
        use regex::RegexBuilder;

        let pattern = r"\blorem_ipsum\(\s*(\d+)\s*\)";
        let regex = RegexBuilder::new(pattern).case_insensitive(true).build()?;
        Ok(regex
            .replace_all(input, |caps: &regex::Captures| {
                let val = &caps[1]
                    .parse::<usize>()
                    .expect("Could not parse number of words");
                if val < &LOREM_IPSUM_WORDS.len() {
                    LOREM_IPSUM_WORDS
                        .iter()
                        .take(*val)
                        .map(|w| w.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                } else {
                    let mut words = Vec::new();
                    for i in 0..*val {
                        words.push(LOREM_IPSUM_WORDS[i % LOREM_IPSUM_WORDS.len()].to_string());
                    }
                    words.join(" ")
                }
            })
            .to_string())
    }
}
