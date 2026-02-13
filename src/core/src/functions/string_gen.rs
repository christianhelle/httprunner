use super::substitution::FunctionSubstitutor;

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

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_generate_string() {
        let string = StringSubstitutor {}.generate();
        let alphanum_pattern = Regex::new(r"^[A-Za-z0-9]+$").unwrap();
        assert!(
            alphanum_pattern.is_match(&string),
            "Generated string '{}' is not purely alphanumeric",
            string
        );
        assert_eq!(
            string.len(),
            20,
            "Generated string '{}' should be exactly 20 alphanumeric characters, got {}",
            string,
            string.len()
        );
    }

    #[test]
    fn test_string_substitutor_regex() {
        let sub = StringSubstitutor {};
        let regex = regex::Regex::new(sub.get_regex()).unwrap();

        assert!(regex.is_match("string()"));
        assert!(regex.is_match("test string()"));
        assert!(!regex.is_match("nostring()"));
        assert!(!regex.is_match("mystring()"));
    }

    #[test]
    fn test_string_substitutor_generates_20_chars() {
        let sub = StringSubstitutor {};
        let s = sub.generate();

        assert_eq!(s.len(), 20);
    }

    #[test]
    fn test_string_substitutor_generates_alphanumeric() {
        let sub = StringSubstitutor {};
        let s = sub.generate();

        assert!(s.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_string_substitutor_generates_different_values() {
        let sub = StringSubstitutor {};
        let s1 = sub.generate();
        let s2 = sub.generate();

        assert_ne!(s1, s2);
    }

    #[test]
    fn test_string_not_matches_with_prefix() {
        let sub = StringSubstitutor {};
        let regex = regex::Regex::new(sub.get_regex()).unwrap();

        assert!(!regex.is_match("mystring()"));
        assert!(!regex.is_match("_string()"));
        assert!(!regex.is_match("xstring()"));
    }

    #[test]
    fn test_string_word_boundary_strict() {
        let sub = StringSubstitutor {};
        let regex = regex::Regex::new(sub.get_regex()).unwrap();

        assert!(!regex.is_match("stringextra()"));
        assert!(!regex.is_match("prefixstring()"));
        assert!(!regex.is_match("_string()"));
    }

    #[test]
    fn test_string_character_distribution() {
        use std::collections::HashSet;

        let sub = StringSubstitutor {};
        let mut chars_found = HashSet::new();

        for _ in 0..1000 {
            let s = sub.generate();
            for c in s.chars() {
                chars_found.insert(c);
            }
        }

        assert!(
            chars_found.iter().any(|c| c.is_ascii_digit()),
            "Should include digits"
        );
        assert!(
            chars_found.iter().any(|c| c.is_ascii_uppercase()),
            "Should include uppercase"
        );
        assert!(
            chars_found.iter().any(|c| c.is_ascii_lowercase()),
            "Should include lowercase"
        );
    }

    #[test]
    fn test_string_generation_variety() {
        use std::collections::HashSet;

        let sub = StringSubstitutor {};
        let mut values = HashSet::new();

        for _ in 0..200 {
            values.insert(sub.generate());
        }

        assert!(
            values.len() > 150,
            "Should generate mostly unique strings in 200 iterations"
        );
    }
}
