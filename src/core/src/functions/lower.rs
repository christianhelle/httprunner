use super::substitution::FunctionSubstitutor;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_basic() {
        let result = LowerSubstitutor {}.replace(&String::from("lower('HELLO, WORLD')"));
        assert!(result.is_ok(), "Expected Ok result, got {:?}", result);
        assert_eq!(result.unwrap(), "hello, world");
    }

    #[test]
    fn test_lower_substitutor_single_quote() {
        let sub = LowerSubstitutor {};
        let input = "Code: lower('HELLO')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "Code: hello");
    }

    #[test]
    fn test_lower_substitutor_with_spaces() {
        let sub = LowerSubstitutor {};
        let input = "lower( 'TEST TEXT' )";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "test text");
    }

    #[test]
    fn test_lower_substitutor_case_insensitive() {
        let sub = LowerSubstitutor {};

        let input1 = "LOWER('HELLO')";
        let result1 = sub.replace(input1).unwrap();
        assert_eq!(result1, "hello");

        let input2 = "Lower('HELLO')";
        let result2 = sub.replace(input2).unwrap();
        assert_eq!(result2, "hello");

        let input3 = "lOwEr('HELLO')";
        let result3 = sub.replace(input3).unwrap();
        assert_eq!(result3, "hello");
    }

    #[test]
    fn test_lower_substitutor_empty_string() {
        let sub = LowerSubstitutor {};
        let input = "lower('')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "");
    }

    #[test]
    fn test_lower_substitutor_special_chars() {
        let sub = LowerSubstitutor {};
        let input = "lower('HELLO, WORLD!')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "hello, world!");
    }

    #[test]
    fn test_lower_substitutor_multiple() {
        let sub = LowerSubstitutor {};
        let input = "lower('FOO') and lower('BAR')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "foo and bar");
    }

    #[test]
    fn test_lower_substitutor_no_match() {
        let sub = LowerSubstitutor {};
        let input = "no transformation here";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "no transformation here");
    }

    #[test]
    fn test_lower_substitutor_json_body() {
        let sub = LowerSubstitutor {};
        let input = r#"{"code": "lower('ABC123')"}"#;
        let result = sub.replace(input).unwrap();

        assert_eq!(result, r#"{"code": "abc123"}"#);
    }

    #[test]
    fn test_lower_with_numbers() {
        let sub = LowerSubstitutor {};
        let input = "lower('TEST123ABC')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "test123abc");
    }

    #[test]
    fn test_lower_already_lowercase() {
        let sub = LowerSubstitutor {};
        let input = "lower('already lowercase')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "already lowercase");
    }

    #[test]
    fn test_lower_mixed_case() {
        let sub = LowerSubstitutor {};
        let input = "lower('MiXeD CaSe TeXt')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "mixed case text");
    }

    #[test]
    fn test_lower_with_unicode() {
        let sub = LowerSubstitutor {};
        let input = "lower('HELLO CAFÉ')";
        let result = sub.replace(input).unwrap();

        assert!(!result.contains("lower"));
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_lower_with_single_quote_in_value() {
        let sub = LowerSubstitutor {};
        let input = "lower('HE\\'S HERE')";
        let result = sub.replace(input).unwrap();

        assert!(!result.contains("lower"));
    }

    #[test]
    fn test_lower_regex_word_boundaries() {
        let sub = LowerSubstitutor {};
        let input1 = "lower('TEST')";
        let result1 = sub.replace(input1).unwrap();
        assert!(!result1.contains("lower"));

        let input2 = "use lower('TEST') here";
        let result2 = sub.replace(input2).unwrap();
        assert!(!result2.contains("lower"));
    }

    #[test]
    fn test_lower_consistency() {
        let sub = LowerSubstitutor {};
        let input = "lower('CONSISTENT')";

        let result1 = sub.replace(input).unwrap();
        let result2 = sub.replace(input).unwrap();

        assert_eq!(
            result1, result2,
            "Lower transformation should be consistent"
        );
        assert_eq!(result1, "consistent");
    }

    #[test]
    fn test_lower_very_long_string() {
        let sub = LowerSubstitutor {};
        let long_upper = "A".repeat(1000);
        let long_lower = "a".repeat(1000);

        let input = format!("lower('{}')", long_upper);
        let result = sub.replace(&input).unwrap();
        assert!(!result.contains("lower"));
        assert_eq!(result, long_lower);
    }
}
