use super::substitution::FunctionSubstitutor;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encode() {
        let result =
            Base64EncodeSubstitutor {}.replace(&String::from("base64_encode('Hello, World!')"));
        assert!(result.is_ok(), "Expected Ok result, got {:?}", result);
        assert_eq!(result.unwrap(), "SGVsbG8sIFdvcmxkIQ==");
    }

    #[test]
    fn test_base64_encode_substitutor_single_quote() {
        let sub = Base64EncodeSubstitutor {};
        let input = "Bearer base64_encode('username:password')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "Bearer dXNlcm5hbWU6cGFzc3dvcmQ=");
    }

    #[test]
    fn test_base64_encode_substitutor_with_spaces() {
        let sub = Base64EncodeSubstitutor {};
        let input = "base64_encode( 'test' )";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "dGVzdA==");
    }

    #[test]
    fn test_base64_encode_substitutor_case_insensitive() {
        let sub = Base64EncodeSubstitutor {};

        let input1 = "BASE64_ENCODE('hello')";
        let result1 = sub.replace(input1).unwrap();
        assert_eq!(result1, "aGVsbG8=");

        let input2 = "Base64_Encode('hello')";
        let result2 = sub.replace(input2).unwrap();
        assert_eq!(result2, "aGVsbG8=");
    }

    #[test]
    fn test_base64_encode_substitutor_empty_string() {
        let sub = Base64EncodeSubstitutor {};
        let input = "base64_encode('')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "");
    }

    #[test]
    fn test_base64_encode_substitutor_special_chars() {
        let sub = Base64EncodeSubstitutor {};
        let input = "base64_encode('Hello, World!')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "SGVsbG8sIFdvcmxkIQ==");
    }

    #[test]
    fn test_base64_encode_substitutor_multiple() {
        let sub = Base64EncodeSubstitutor {};
        let input = "base64_encode('foo') and base64_encode('bar')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "Zm9v and YmFy");
    }

    #[test]
    fn test_base64_encode_substitutor_no_match() {
        let sub = Base64EncodeSubstitutor {};
        let input = "no encoding here";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "no encoding here");
    }

    #[test]
    fn test_base64_encode_substitutor_json_body() {
        let sub = Base64EncodeSubstitutor {};
        let input = r#"{"auth": "base64_encode('user:pass')"}"#;
        let result = sub.replace(input).unwrap();

        assert_eq!(result, r#"{"auth": "dXNlcjpwYXNz"}"#);
    }

    #[test]
    fn test_base64_encode_with_unicode() {
        let sub = Base64EncodeSubstitutor {};
        let input = "base64_encode('Hello 世界')";
        let result = sub.replace(input).unwrap();

        assert!(!result.contains("base64_encode"));
        assert!(!result.contains("世"));
        assert!(
            result
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
        );
    }

    #[test]
    fn test_base64_encode_with_escape_sequences() {
        let sub = Base64EncodeSubstitutor {};
        let input = "base64_encode('Line1\\nLine2\\tTab')";
        let result = sub.replace(input).unwrap();

        assert!(!result.contains("base64_encode"));
    }

    #[test]
    fn test_base64_encode_with_very_long_string() {
        let sub = Base64EncodeSubstitutor {};
        let long_string = "x".repeat(1000);
        let input = format!("base64_encode('{}')", long_string);
        let result = sub.replace(&input).unwrap();

        assert!(!result.contains("base64_encode"));
        assert!(!result.contains(&long_string));
    }

    #[test]
    fn test_multiple_spaces_in_base64() {
        let sub = Base64EncodeSubstitutor {};
        let input = "base64_encode(  'test'  )";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "dGVzdA==");
    }

    #[test]
    fn test_base64_with_quotes_in_string() {
        let sub = Base64EncodeSubstitutor {};
        let input = "base64_encode('Say \\'Hello\\'')";
        let result = sub.replace(input).unwrap();

        assert!(!result.contains("base64_encode"));
    }

    #[test]
    fn test_base64_special_characters() {
        let sub = Base64EncodeSubstitutor {};

        let test_cases = vec![
            ("base64_encode('!@#$%^&*()')", "!@#$%^&*()"),
            ("base64_encode('[]{}()<>?')", "[]{}()<>?"),
            ("base64_encode('.,;:')", ".,;:"),
        ];

        for (input, expected_part) in test_cases {
            let result = sub.replace(input).unwrap();
            assert!(!result.contains("base64_encode"));
            assert!(!result.contains(expected_part));
        }
    }

    #[test]
    fn test_base64_numeric_string() {
        let sub = Base64EncodeSubstitutor {};
        let input = "base64_encode('1234567890')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "MTIzNDU2Nzg5MA==");
    }

    #[test]
    fn test_base64_mixed_case() {
        let sub = Base64EncodeSubstitutor {};
        let input = "base64_encode('AbCdEfGhIj')";
        let result = sub.replace(input).unwrap();

        assert!(!result.contains("base64_encode"));
        assert!(!result.contains("AbCdEfGhIj"));
    }

    #[test]
    fn test_base64_encode_with_single_quote_in_value() {
        let sub = Base64EncodeSubstitutor {};
        let input = "base64_encode('He\\'s here')";
        let result = sub.replace(input).unwrap();

        assert!(!result.contains("base64_encode"));
    }

    #[test]
    fn test_base64_encode_with_backslash() {
        let sub = Base64EncodeSubstitutor {};
        let input = r#"base64_encode('path\\to\\file')"#;
        let result = sub.replace(input).unwrap();

        assert!(!result.contains("base64_encode"));
    }

    #[test]
    fn test_base64_encode_all_printable_ascii() {
        let sub = Base64EncodeSubstitutor {};
        let printable_ascii = "!\"#$%&()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~";
        let input = format!("base64_encode('{}')", printable_ascii);
        let result = sub.replace(&input).unwrap();

        assert!(!result.contains("base64_encode"));
        assert!(!result.contains(printable_ascii));
        assert!(
            result
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
        );
    }

    #[test]
    fn test_base64_encode_with_single_character() {
        let sub = Base64EncodeSubstitutor {};
        let input = "base64_encode('a')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "YQ==");
        assert!(!result.contains("base64_encode"));
    }

    #[test]
    fn test_base64_encode_with_two_characters() {
        let sub = Base64EncodeSubstitutor {};
        let input = "base64_encode('ab')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "YWI=");
    }

    #[test]
    fn test_base64_encode_with_three_characters() {
        let sub = Base64EncodeSubstitutor {};
        let input = "base64_encode('abc')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "YWJj");
    }

    #[test]
    fn test_base64_encode_padding_variants() {
        let sub = Base64EncodeSubstitutor {};

        let r1 = sub.replace("base64_encode('abc')").unwrap();
        assert_eq!(r1, "YWJj");

        let r2 = sub.replace("base64_encode('ab')").unwrap();
        assert_eq!(r2, "YWI=");

        let r3 = sub.replace("base64_encode('a')").unwrap();
        assert_eq!(r3, "YQ==");
    }

    #[test]
    fn test_base64_encode_numbers_only() {
        let sub = Base64EncodeSubstitutor {};
        let input = "base64_encode('0123456789')";
        let result = sub.replace(input).unwrap();

        assert_eq!(result, "MDEyMzQ1Njc4OQ==");
    }

    #[test]
    fn test_base64_encode_regex_word_boundaries() {
        let sub = Base64EncodeSubstitutor {};
        let input1 = "base64_encode('test')";
        let result1 = sub.replace(input1).unwrap();
        assert!(!result1.contains("base64_encode"));

        let input2 = "use base64_encode('test') here";
        let result2 = sub.replace(input2).unwrap();
        assert!(!result2.contains("base64_encode"));
    }

    #[test]
    fn test_base64_encode_consistency() {
        let sub = Base64EncodeSubstitutor {};
        let input = "base64_encode('consistent')";

        let result1 = sub.replace(input).unwrap();
        let result2 = sub.replace(input).unwrap();

        assert_eq!(result1, result2, "Base64 encoding should be consistent");
    }

    #[test]
    fn test_base64_with_all_base64_chars() {
        let sub = Base64EncodeSubstitutor {};
        let input =
            "base64_encode('ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/')";
        let result = sub.replace(input).unwrap();

        assert!(!result.contains("base64_encode"));
    }

    #[test]
    fn test_base64_encode_very_long_input() {
        let sub = Base64EncodeSubstitutor {};
        let long_text = "a".repeat(5000);
        let input = format!("base64_encode('{}')", long_text);
        let result = sub.replace(&input).unwrap();

        assert!(!result.contains("base64_encode"));
        assert!(!result.contains(&long_text));
    }
}
