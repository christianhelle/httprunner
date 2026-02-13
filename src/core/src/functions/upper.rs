use super::substitution::FunctionSubstitutor;

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
            .replace_all(input, |caps: &regex::Captures| caps[1].to_uppercase())
            .to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upper_basic() {
        let result = UpperSubstitutor {}.replace(&String::from("upper('hello, world')"));
        assert!(result.is_ok(), "Expected Ok result, got {:?}", result);
        assert_eq!(result.unwrap(), "HELLO, WORLD");
    }

    #[test]
    fn test_upper_substitutor_single_quote() {
        let sub = UpperSubstitutor {};
        let input = "Code: upper('hello')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "Code: HELLO");
    }

    #[test]
    fn test_upper_substitutor_with_spaces() {
        let sub = UpperSubstitutor {};
        let input = "upper( 'test text' )";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "TEST TEXT");
    }

    #[test]
    fn test_upper_substitutor_case_insensitive() {
        let sub = UpperSubstitutor {};

        let input1 = "UPPER('hello')";
        let result1 = sub.replace(input1).unwrap();
        assert_eq!(result1, "HELLO");

        let input2 = "Upper('hello')";
        let result2 = sub.replace(input2).unwrap();
        assert_eq!(result2, "HELLO");

        let input3 = "uPpEr('hello')";
        let result3 = sub.replace(input3).unwrap();
        assert_eq!(result3, "HELLO");
    }

    #[test]
    fn test_upper_substitutor_empty_string() {
        let sub = UpperSubstitutor {};
        let input = "upper('')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "");
    }

    #[test]
    fn test_upper_substitutor_special_chars() {
        let sub = UpperSubstitutor {};
        let input = "upper('hello, world!')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "HELLO, WORLD!");
    }

    #[test]
    fn test_upper_substitutor_multiple() {
        let sub = UpperSubstitutor {};
        let input = "upper('foo') and upper('bar')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "FOO and BAR");
    }

    #[test]
    fn test_upper_substitutor_no_match() {
        let sub = UpperSubstitutor {};
        let input = "no transformation here";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "no transformation here");
    }

    #[test]
    fn test_upper_substitutor_json_body() {
        let sub = UpperSubstitutor {};
        let input = r#"{"code": "upper('abc123')"}"#;
        let result = sub.replace(input).unwrap();

        assert_eq!(result, r#"{"code": "ABC123"}"#);
    }

    #[test]
    fn test_upper_with_numbers() {
        let sub = UpperSubstitutor {};
        let input = "upper('test123abc')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "TEST123ABC");
    }

    #[test]
    fn test_upper_already_uppercase() {
        let sub = UpperSubstitutor {};
        let input = "upper('ALREADY UPPERCASE')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "ALREADY UPPERCASE");
    }

    #[test]
    fn test_upper_mixed_case() {
        let sub = UpperSubstitutor {};
        let input = "upper('MiXeD CaSe TeXt')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "MIXED CASE TEXT");
    }

    #[test]
    fn test_upper_with_unicode() {
        let sub = UpperSubstitutor {};
        let input = "upper('hello café')";
        let result = sub.replace(input).unwrap();

        assert!(!result.contains("upper"));
        assert!(result.contains("HELLO"));
    }

    #[test]
    fn test_upper_with_single_quote_in_value() {
        let sub = UpperSubstitutor {};
        let input = "upper('he\\'s here')";
        let result = sub.replace(input).unwrap();

        assert!(!result.contains("upper"));
    }

    #[test]
    fn test_upper_regex_word_boundaries() {
        let sub = UpperSubstitutor {};
        let input1 = "upper('test')";
        let result1 = sub.replace(input1).unwrap();
        assert!(!result1.contains("upper"));

        let input2 = "use upper('test') here";
        let result2 = sub.replace(input2).unwrap();
        assert!(!result2.contains("upper"));
    }

    #[test]
    fn test_upper_consistency() {
        let sub = UpperSubstitutor {};
        let input = "upper('consistent')";

        let result1 = sub.replace(input).unwrap();
        let result2 = sub.replace(input).unwrap();

        assert_eq!(
            result1, result2,
            "Upper transformation should be consistent"
        );
        assert_eq!(result1, "CONSISTENT");
    }
}
