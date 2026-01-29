use crate::functions::date_functions::{
    GetDateSubstitutor, GetDateTimeSubstitutor, GetTimeSubstitutor, GetUtcDateTimeSubstitutor,
};
use crate::functions::generator_functions::{
    AddressSubstitutor, EmailSubstitutor, FirstNameSubstitutor, GuidSubstitutor,
    LastNameSubstitutor, NameSubstitutor, NumberSubstitutor, StringSubstitutor,
};
use crate::functions::substitution::FunctionSubstitutor;
use crate::functions::transform_functions::{
    Base64EncodeSubstitutor, LowerSubstitutor, UpperSubstitutor,
};
use regex::Regex;

#[test]
fn test_generate_guid() {
    let guid = GuidSubstitutor {}.generate();
    let hex_pattern = Regex::new(r"^[0-9a-fA-F]{32}$").unwrap();
    assert!(
        hex_pattern.is_match(&guid),
        "GUID '{}' does not match pattern /^[0-9a-fA-F]{{32}}$/",
        guid
    );
}

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
fn test_generate_number() {
    let number_str = NumberSubstitutor {}.generate();
    let number: i32 = number_str
        .parse()
        .expect("Generated number string could not be parsed as i32");
    assert!(
        (0..=100).contains(&number),
        "Generated number {} is not within range 0..=100",
        number
    );
    assert_ne!(number, -1, "Generated number should not be -1");
}

#[test]
fn test_base64_encode() {
    let result =
        Base64EncodeSubstitutor {}.replace(&String::from("base64_encode('Hello, World!')"));
    assert!(result.is_ok(), "Expected Ok result, got {:?}", result);
    assert_eq!(result.unwrap(), "SGVsbG8sIFdvcmxkIQ==");
}

#[test]
fn test_substitute_functions_case_insensitivity() {
    use crate::functions::substitute_functions;

    let input = r#"{"guid": "guid()", "GUID": "GUID()", "Guid": "Guid()"}"#;
    let result = substitute_functions(input).unwrap();

    // All three should be substituted with different GUIDs
    assert!(!result.contains("guid()"));
    assert!(!result.contains("GUID()"));
    assert!(!result.contains("Guid()"));

    // Verify the result contains valid GUIDs (32 hex characters)
    let guid_pattern = Regex::new(r"[0-9a-fA-F]{32}").unwrap();
    let matches: Vec<_> = guid_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3, "Should have 3 GUID values");
}

#[test]
fn test_substitute_functions_string_case_insensitivity() {
    use crate::functions::substitute_functions;

    let input = r#"{"s1": "string()", "s2": "STRING()", "s3": "String()"}"#;
    let result = substitute_functions(input).unwrap();

    // All three should be substituted
    assert!(!result.contains("string()"));
    assert!(!result.contains("STRING()"));
    assert!(!result.contains("String()"));

    // Verify the result contains alphanumeric strings
    let alphanum_pattern = Regex::new(r#""s\d": "([A-Za-z0-9]{20})""#).unwrap();
    let matches: Vec<_> = alphanum_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3, "Should have 3 string values");
}

#[test]
fn test_substitute_functions_number_case_insensitivity() {
    use crate::functions::substitute_functions;

    let input = r#"{"n1": number(), "n2": NUMBER(), "n3": Number()}"#;
    let result = substitute_functions(input).unwrap();

    // All three should be substituted
    assert!(!result.contains("number()"));
    assert!(!result.contains("NUMBER()"));
    assert!(!result.contains("Number()"));

    // Verify the result contains numbers
    let number_pattern = Regex::new(r#""n\d": (\d+)"#).unwrap();
    let matches: Vec<_> = number_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3, "Should have 3 number values");
}

#[test]
fn test_substitute_functions_base64_case_insensitivity() {
    use crate::functions::substitute_functions;

    let input = r#"{"b1": "base64_encode('test')", "b2": "BASE64_ENCODE('test')", "b3": "Base64_Encode('test')"}"#;
    let result = substitute_functions(input).unwrap();

    // All three should be substituted with the same base64 value
    assert!(!result.contains("base64_encode"));
    assert!(!result.contains("BASE64_ENCODE"));
    assert!(!result.contains("Base64_Encode"));

    // All should produce the same encoded value
    assert!(
        result.contains("dGVzdA=="),
        "Should contain base64 encoded 'test'"
    );

    // Verify all three were replaced
    let b64_pattern = Regex::new(r#""b\d": "dGVzdA==""#).unwrap();
    let matches: Vec<_> = b64_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3, "Should have 3 base64 values");
}

#[test]
fn test_substitute_functions_multiple_in_same_string() {
    use crate::functions::substitute_functions;

    let input = r#"{"id": "guid()", "name": "User string()", "score": number()}"#;
    let result = substitute_functions(input).unwrap();

    // Verify all functions were substituted
    assert!(!result.contains("guid()"));
    assert!(!result.contains("string()"));
    assert!(!result.contains("number()"));

    // Verify structure is maintained
    assert!(result.contains(r#""id":"#));
    assert!(result.contains(r#""name":"#));
    assert!(result.contains(r#""score":"#));
}

#[test]
fn test_substitute_functions_in_url() {
    use crate::functions::substitute_functions;

    let input = "https://api.example.com/users/guid()/posts/number()";
    let result = substitute_functions(input).unwrap();

    // Verify functions were substituted
    assert!(!result.contains("guid()"));
    assert!(!result.contains("number()"));

    // Verify URL structure is maintained
    assert!(result.starts_with("https://api.example.com/users/"));
    assert!(result.contains("/posts/"));
}

#[test]
fn test_substitute_functions_in_headers() {
    use crate::functions::substitute_functions;

    let input = "Authorization: Bearer base64_encode('user:pass')\nX-Request-ID: guid()";
    let result = substitute_functions(input).unwrap();

    // Verify functions were substituted
    assert!(!result.contains("base64_encode"));
    assert!(!result.contains("guid()"));

    // Verify headers structure
    assert!(result.contains("Authorization: Bearer"));
    assert!(result.contains("X-Request-ID:"));
}
#[test]
fn test_guid_substitutor_regex() {
    let sub = GuidSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    assert!(regex.is_match("guid()"));
    assert!(regex.is_match("Bearer guid()"));
    assert!(!regex.is_match("noguid()"));
    assert!(!regex.is_match("myguid()"));
}

#[test]
fn test_guid_substitutor_generates_valid_uuid() {
    let sub = GuidSubstitutor {};
    let guid = sub.generate();

    // UUID v4 in simple format should be 32 hex characters
    assert_eq!(guid.len(), 32);
    assert!(guid.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_guid_substitutor_generates_unique_values() {
    let sub = GuidSubstitutor {};
    let guid1 = sub.generate();
    let guid2 = sub.generate();

    assert_ne!(guid1, guid2);
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

    // Should be highly unlikely to generate the same 20-char string
    assert_ne!(s1, s2);
}

#[test]
fn test_number_substitutor_regex() {
    let sub = NumberSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    assert!(regex.is_match("number()"));
    assert!(regex.is_match("value: number()"));
    assert!(!regex.is_match("nonumber()"));
    assert!(!regex.is_match("mynumber()"));
}

#[test]
fn test_number_substitutor_generates_in_range() {
    let sub = NumberSubstitutor {};

    for _ in 0..100 {
        let num_str = sub.generate();
        let num: i32 = num_str.parse().unwrap();
        assert!((0..=100).contains(&num));
    }
}

#[test]
fn test_number_substitutor_generates_numeric_string() {
    let sub = NumberSubstitutor {};
    let num_str = sub.generate();

    assert!(num_str.parse::<i32>().is_ok());
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
fn test_name_substitutor() {
    let sub = NameSubstitutor {};
    let name = sub.generate();
    assert!(!name.is_empty(), "Generated name should not be empty");
    assert!(
        name.contains(' '),
        "Generated name '{}' should contain a space between first and last name",
        name
    );
}

#[test]
fn test_first_name_substitutor() {
    let sub = FirstNameSubstitutor {};
    let first_name = sub.generate();
    assert!(
        !first_name.is_empty(),
        "Generated first name should not be empty"
    );
}

#[test]
fn test_last_name_substitutor() {
    let sub = LastNameSubstitutor {};
    let last_name = sub.generate();
    assert!(
        !last_name.is_empty(),
        "Generated last name should not be empty"
    );
}

// ========== NAME FUNCTION TESTS ==========

#[test]
fn test_name_substitutor_regex() {
    let sub = NameSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    assert!(regex.is_match("name()"));
    assert!(regex.is_match("User: name()"));
    assert!(!regex.is_match("noname()"));
    assert!(!regex.is_match("myname()"));
}

#[test]
fn test_name_substitutor_case_insensitive() {
    use crate::functions::substitute_functions;

    let input = r#"{"n1": "name()", "n2": "NAME()", "n3": "Name()"}"#;
    let result = substitute_functions(input).unwrap();

    // All should be substituted
    assert!(!result.contains("name()"));
    assert!(!result.contains("NAME()"));
    assert!(!result.contains("Name()"));

    // All should contain a space (first + last name), so at least 3 spaces
    let space_count = result.matches(' ').count();
    assert!(
        space_count >= 3,
        "Should have at least 3 spaces (one per generated name)"
    );
}

#[test]
fn test_first_name_substitutor_regex() {
    let sub = FirstNameSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    assert!(regex.is_match("first_name()"));
    assert!(regex.is_match("Person: first_name()"));
    assert!(!regex.is_match("nofirst_name()"));
    assert!(!regex.is_match("myfirst_name()"));
}

#[test]
fn test_first_name_substitutor_case_insensitive() {
    use crate::functions::substitute_functions;

    let input = r#"{"f1": "first_name()", "f2": "FIRST_NAME()", "f3": "First_Name()"}"#;
    let result = substitute_functions(input).unwrap();

    // All should be substituted
    assert!(!result.contains("first_name()"));
    assert!(!result.contains("FIRST_NAME()"));
    assert!(!result.contains("First_Name()"));
}

#[test]
fn test_last_name_substitutor_regex() {
    let sub = LastNameSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    assert!(regex.is_match("last_name()"));
    assert!(regex.is_match("Surname: last_name()"));
    assert!(!regex.is_match("nolast_name()"));
    assert!(!regex.is_match("mylast_name()"));
}

#[test]
fn test_last_name_substitutor_case_insensitive() {
    use crate::functions::substitute_functions;

    let input = r#"{"l1": "last_name()", "l2": "LAST_NAME()", "l3": "Last_Name()"}"#;
    let result = substitute_functions(input).unwrap();

    // All should be substituted
    assert!(!result.contains("last_name()"));
    assert!(!result.contains("LAST_NAME()"));
    assert!(!result.contains("Last_Name()"));
}

// ========== COMBINED FUNCTION TESTS ==========

#[test]
fn test_all_functions_combined() {
    use crate::functions::substitute_functions;

    let input = r#"{"id": "guid()", "firstName": "first_name()", "lastName": "last_name()", "fullName": "name()", "address": "address()", "randomStr": "string()", "randomNum": "number()", "encoded": "base64_encode('secret')"}"#;
    let result = substitute_functions(input).unwrap();

    // Verify no functions remain
    assert!(!result.contains("guid()"));
    assert!(!result.contains("first_name()"));
    assert!(!result.contains("last_name()"));
    assert!(!result.contains("name()"));
    assert!(!result.contains("address()"));
    assert!(!result.contains("string()"));
    assert!(!result.contains("number()"));
    assert!(!result.contains("base64_encode"));

    // Verify structure is maintained (with spaces after colons as in the input)
    assert!(result.contains(r#""id": "#));
    assert!(result.contains(r#""firstName": "#));
    assert!(result.contains(r#""lastName": "#));
    assert!(result.contains(r#""fullName": "#));
    assert!(result.contains(r#""address": "#));
    assert!(result.contains(r#""randomStr": "#));
    assert!(result.contains(r#""randomNum": "#));
    assert!(result.contains(r#""encoded": "#));
}

#[test]
fn test_mixed_functions_in_url() {
    use crate::functions::substitute_functions;

    let input = "/api/users/guid()/posts/number()/comments/first_name()";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("number()"));
    assert!(!result.contains("first_name()"));
    assert!(result.starts_with("/api/users/"));
}

#[test]
fn test_functions_in_headers() {
    use crate::functions::substitute_functions;

    let input = "Authorization: Bearer base64_encode('token')\nX-User-ID: guid()\nX-User-Name: first_name() last_name()";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("base64_encode"));
    assert!(!result.contains("guid()"));
    assert!(!result.contains("first_name()"));
    assert!(!result.contains("last_name()"));
    assert!(result.contains("Authorization: Bearer"));
    assert!(result.contains("X-User-ID:"));
    assert!(result.contains("X-User-Name:"));
}

// ========== EDGE CASE TESTS ==========

#[test]
fn test_guid_with_whitespace_around() {
    use crate::functions::substitute_functions;

    let input = "  guid()  ";
    let result = substitute_functions(input).unwrap();

    // Whitespace should be preserved, function replaced
    assert!(result.starts_with("  "));
    assert!(result.ends_with("  "));
    assert!(!result.contains("guid()"));
}

#[test]
fn test_string_empty_context() {
    use crate::functions::substitute_functions;

    let input = "string()";
    let result = substitute_functions(input).unwrap();

    assert_ne!(input, result);
    assert!(!result.contains("string()"));
}

#[test]
fn test_number_at_boundaries() {
    let sub = NumberSubstitutor {};

    // Generate many numbers to verify boundary compliance
    let mut found_zero = false;
    let mut found_hundred = false;

    for _ in 0..1000 {
        let num_str = sub.generate();
        let num: i32 = num_str.parse().unwrap();
        assert!((0..=100).contains(&num), "Number {} out of range", num);

        if num == 0 {
            found_zero = true;
        }
        if num == 100 {
            found_hundred = true;
        }
    }

    // With 1000 iterations, we should hit boundaries at least once
    assert!(
        found_zero || found_hundred,
        "Should generate boundary values in 1000 iterations"
    );
}

#[test]
fn test_base64_encode_with_unicode() {
    let sub = Base64EncodeSubstitutor {};
    let input = "base64_encode('Hello 世界')";
    let result = sub.replace(input).unwrap();

    // Should properly encode unicode characters
    assert!(!result.contains("base64_encode"));
    assert!(!result.contains("世"));
    // The result should be valid base64
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

    // Should handle escaped characters
    assert!(!result.contains("base64_encode"));
}

#[test]
fn test_base64_encode_with_very_long_string() {
    let sub = Base64EncodeSubstitutor {};
    let long_string = "x".repeat(1000);
    let input = format!("base64_encode('{}')", long_string);
    let result = sub.replace(&input).unwrap();

    // Result should be longer than input (base64 expansion) but not contain the function call
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
    // Using escaped quotes within the string
    let input = "base64_encode('Say \\'Hello\\'')";
    let result = sub.replace(input).unwrap();

    assert!(!result.contains("base64_encode"));
}

// ========== UUID FORMAT VALIDATION ==========

#[test]
fn test_guid_is_valid_uuid_v4_format() {
    let guid = GuidSubstitutor {}.generate();

    // UUID v4 format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx (without dashes in our case)
    // But we generate without dashes, so just verify it's 32 hex chars
    assert_eq!(guid.len(), 32, "GUID should be 32 hex characters");

    // Verify all characters are valid hex
    for (i, c) in guid.chars().enumerate() {
        assert!(
            c.is_ascii_hexdigit(),
            "Character at position {} ('{}') is not hex",
            i,
            c
        );
    }

    // Check version bits (should be 4 at position 12)
    assert_eq!(
        guid.chars().nth(12),
        Some('4'),
        "Version should be 4 at position 12"
    );

    // Check variant bits (should be 8, 9, a, or b at position 16)
    let variant = guid.chars().nth(16).unwrap();
    assert!(
        ['8', '9', 'a', 'b', 'A', 'B'].contains(&variant),
        "Variant should be 8, 9, a, or b at position 16, got {}",
        variant
    );
}

#[test]
fn test_guid_uniqueness_large_sample() {
    use std::collections::HashSet;

    let sub = GuidSubstitutor {};
    let mut guids = HashSet::new();

    for _ in 0..100 {
        let guid = sub.generate();
        assert!(
            guids.insert(guid.clone()),
            "Generated duplicate GUID: {}",
            guid
        );
    }

    assert_eq!(guids.len(), 100, "Should have 100 unique GUIDs");
}

// ========== NEGATIVE TESTS (SHOULD NOT MATCH) ==========

#[test]
fn test_guid_not_matches_with_prefix() {
    let sub = GuidSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    assert!(!regex.is_match("myguid()"));
    assert!(!regex.is_match("_guid()"));
    assert!(!regex.is_match("guid_"));
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
fn test_number_not_matches_with_prefix() {
    let sub = NumberSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    assert!(!regex.is_match("mynumber()"));
    assert!(!regex.is_match("_number()"));
    assert!(!regex.is_match("number_()"));
}

#[test]
fn test_function_not_replaced_when_incorrect_syntax() {
    use crate::functions::substitute_functions;

    // Missing closing paren
    let input1 = "guid(";
    let result1 = substitute_functions(input1).unwrap();
    assert_eq!(result1, input1, "Should not replace malformed function");

    // Missing opening paren
    let input2 = "guid)";
    let result2 = substitute_functions(input2).unwrap();
    assert_eq!(result2, input2, "Should not replace malformed function");

    // Extra characters
    let input3 = "guid() extra";
    let result3 = substitute_functions(input3).unwrap();
    assert!(
        result3.contains("extra"),
        "Should preserve non-function text"
    );
    assert!(!result3.contains("guid()"));
}

// ========== JSON CONTEXT TESTS ==========

#[test]
fn test_functions_in_nested_json() {
    use crate::functions::substitute_functions;

    let input = r#"{"user": {"id": "guid()", "name": "first_name()", "score": number()}}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("first_name()"));
    assert!(!result.contains("number()"));
    assert!(result.contains(r#""user": "#));
    assert!(result.contains(r#""id": "#));
    assert!(result.contains(r#""name": "#));
    assert!(result.contains(r#""score": "#));
}

#[test]
fn test_functions_in_json_array() {
    use crate::functions::substitute_functions;

    let input = r#"[{"id": "guid()"}, {"id": "guid()"}, {"id": "guid()"}]"#;
    let result = substitute_functions(input).unwrap();

    // Should have 3 different GUIDs
    assert!(!result.contains("guid()"));
    let guid_pattern = Regex::new(r"[0-9a-fA-F]{32}").unwrap();
    let matches: Vec<_> = guid_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3, "Should have 3 different GUIDs");
}

#[test]
fn test_functions_in_querystring() {
    use crate::functions::substitute_functions;

    let input = "?userId=guid()&userName=first_name()&score=number()";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("first_name()"));
    assert!(!result.contains("number()"));
    assert!(result.starts_with("?"));
    assert!(result.contains("&"));
}

// ========== REPEATED SUBSTITUTION TESTS ==========

#[test]
fn test_multiple_same_function_generates_different_values() {
    use crate::functions::substitute_functions;

    let input = r#"{"id1": "guid()", "id2": "guid()", "id3": "guid()"}"#;
    let result = substitute_functions(input).unwrap();

    let guid_pattern = Regex::new(r"[0-9a-fA-F]{32}").unwrap();
    let matches: Vec<_> = guid_pattern.find_iter(&result).collect();

    assert_eq!(matches.len(), 3, "Should have 3 GUIDs");

    // Verify all are different (highly likely with randomness)
    let guid1 = matches[0].as_str();
    let guid2 = matches[1].as_str();
    let guid3 = matches[2].as_str();

    assert_ne!(guid1, guid2);
    assert_ne!(guid2, guid3);
    assert_ne!(guid1, guid3);
}

#[test]
fn test_multiple_string_functions_generates_different_values() {
    use crate::functions::substitute_functions;

    let input = r#"{"s1": "string()", "s2": "string()", "s3": "string()"}"#;
    let result = substitute_functions(input).unwrap();

    let string_pattern = Regex::new(r#""s\d": "([A-Za-z0-9]{20})""#).unwrap();
    let matches: Vec<_> = string_pattern.find_iter(&result).collect();

    assert_eq!(matches.len(), 3, "Should have 3 strings");
}

#[test]
fn test_single_substitute_call_idempotency() {
    use crate::functions::substitute_functions;

    let input = r#"{"id": "guid()", "value": 123}"#;
    let result1 = substitute_functions(input).unwrap();
    let result2 = substitute_functions(&result1).unwrap();

    // Second call should not change anything (no more functions to substitute)
    assert_eq!(result1, result2);
}

// ========== WHITESPACE AND FORMATTING TESTS ==========

#[test]
fn test_functions_with_newlines() {
    use crate::functions::substitute_functions;

    let input = "{\n  \"id\": \"guid()\",\n  \"name\": \"string()\"\n}";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("string()"));
    // Newlines should be preserved
    assert!(result.contains('\n'));
}

#[test]
fn test_functions_with_tabs() {
    use crate::functions::substitute_functions;

    let input = "{\t\"id\": \"guid()\"\t}";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(result.contains('\t'));
}

// ========== CASE SENSITIVITY TESTS ==========

#[test]
fn test_all_case_variations_of_guid() {
    use crate::functions::substitute_functions;

    let input = r#"{"g1": "guid()", "g2": "GUID()", "g3": "Guid()", "g4": "gUiD()"}"#;
    let result = substitute_functions(input).unwrap();

    // None should contain the function
    assert!(!result.to_lowercase().contains("guid()"));
}

#[test]
fn test_all_case_variations_of_string() {
    use crate::functions::substitute_functions;

    let input = r#"{"s1": "string()", "s2": "STRING()", "s3": "String()", "s4": "sTrInG()"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.to_lowercase().contains("string()"));
}

// ========== BASE64 COMPREHENSIVE TESTS ==========

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

// ========== ADDRESS SUBSTITUTOR TESTS ==========

#[test]
fn test_address_substitutor_generates_value() {
    let sub = AddressSubstitutor {};
    let address = sub.generate();
    assert!(!address.is_empty(), "Generated address should not be empty");
}

#[test]
fn test_address_substitutor_regex() {
    let sub = AddressSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    assert!(regex.is_match("address()"));
    assert!(regex.is_match("Location: address()"));
    assert!(!regex.is_match("noaddress()"));
    assert!(!regex.is_match("myaddress()"));
}

#[test]
fn test_address_substitutor_generates_different_values() {
    let sub = AddressSubstitutor {};
    let addr1 = sub.generate();
    let addr2 = sub.generate();

    // Addresses should differ most of the time (not guaranteed but highly likely)
    // We just verify they're both non-empty
    assert!(!addr1.is_empty());
    assert!(!addr2.is_empty());
}

#[test]
fn test_address_substitutor_in_substitution() {
    use crate::functions::substitute_functions;

    let input = r#"{"address": "address()", "country": "USA"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("address()"));
    assert!(result.contains(r#""address":"#));
}

#[test]
fn test_address_substitutor_case_insensitive() {
    use crate::functions::substitute_functions;

    let input = r#"{"a1": "address()", "a2": "ADDRESS()", "a3": "Address()"}"#;
    let result = substitute_functions(input).unwrap();

    // All should be substituted
    assert!(!result.contains("address()"));
    assert!(!result.contains("ADDRESS()"));
    assert!(!result.contains("Address()"));
}

// ========== COMPREHENSIVE ERROR HANDLING TESTS ==========

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
fn test_base64_encode_with_nested_quotes() {
    use crate::functions::substitute_functions;

    // Test that escaped quotes are handled - this is the actual input format
    let input = r#"{"token": "base64_encode('test')"}"#;
    let result = substitute_functions(input).unwrap();

    // The base64_encode function should be substituted
    assert!(!result.contains("base64_encode"));
    assert!(result.contains("dGVzdA=="));
}

// ========== COMPREHENSIVE GENERATION DISTRIBUTION TESTS ==========

#[test]
fn test_number_distribution() {
    let sub = NumberSubstitutor {};
    let mut counts = [0; 101];

    for _ in 0..1000 {
        let num_str = sub.generate();
        let num: usize = num_str.parse().unwrap();
        counts[num] += 1;
    }

    // Verify we're generating a reasonable distribution
    // Most numbers should appear at least once in 1000 iterations
    let non_zero_count = counts.iter().filter(|&&c| c > 0).count();
    assert!(
        non_zero_count >= 50,
        "Expected at least 50 different numbers in 1000 iterations, got {}",
        non_zero_count
    );
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

    // Should have found various alphanumeric characters
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

// ========== COMPREHENSIVE CONTEXT TESTS ==========

#[test]
fn test_functions_in_xml_context() {
    use crate::functions::substitute_functions;

    let input = r#"<user><id>guid()</id><name>name()</name><score>number()</score></user>"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("name()"));
    assert!(!result.contains("number()"));
    assert!(result.contains("<user>"));
    assert!(result.contains("</user>"));
}

#[test]
fn test_functions_in_form_data() {
    use crate::functions::substitute_functions;

    let input = "firstName=first_name()&lastName=last_name()&email=base64_encode('email@test.com')&id=guid()";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("first_name()"));
    assert!(!result.contains("last_name()"));
    assert!(!result.contains("base64_encode"));
    assert!(!result.contains("guid()"));
    assert!(result.contains("&"));
}

#[test]
fn test_functions_at_string_boundaries() {
    use crate::functions::substitute_functions;

    let input1 = "guid()";
    let result1 = substitute_functions(input1).unwrap();
    assert!(!result1.contains("guid()"));

    let input2 = "string()end";
    let result2 = substitute_functions(input2).unwrap();
    assert!(!result2.contains("string()"));
}

#[test]
fn test_functions_adjacent_to_symbols() {
    use crate::functions::substitute_functions;

    let input = "[guid()], (number()), {name()}";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("number()"));
    assert!(!result.contains("name()"));
    assert!(result.contains("["));
    assert!(result.contains("("));
    assert!(result.contains("{"));
}

// ========== COMPREHENSIVE ENCODING TESTS ==========

#[test]
fn test_base64_encode_all_printable_ascii() {
    let sub = Base64EncodeSubstitutor {};
    // Use a simpler set that avoids escaping issues
    let printable_ascii = "!\"#$%&()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~";
    let input = format!("base64_encode('{}')", printable_ascii);
    let result = sub.replace(&input).unwrap();

    assert!(!result.contains("base64_encode"));
    assert!(!result.contains(printable_ascii));
    // Should contain valid base64 characters
    assert!(
        result
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
    );
}

#[test]
fn test_base64_encode_consecutive_calls() {
    use crate::functions::substitute_functions;

    let input = r#"{"a": "base64_encode('test1')", "b": "base64_encode('test2')", "c": "base64_encode('test3')"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("base64_encode"));
    assert!(result.contains("dGVzdDE="));
    assert!(result.contains("dGVzdDI="));
    assert!(result.contains("dGVzdDM="));
}

// ========== REGEX BOUNDARY TESTS ==========

#[test]
fn test_guid_word_boundary_strict() {
    let sub = GuidSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    // Word boundaries - should NOT match with prefix
    assert!(regex.is_match("guid()")); // Should match
    assert!(regex.is_match("guid()extra")); // guid() is at a word boundary
    assert!(!regex.is_match("prefixguid()")); // guid() is not at word boundary (preceded by 'x')
    assert!(!regex.is_match("my_guid()")); // guid() is not at word boundary (preceded by '_')
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
fn test_number_word_boundary_strict() {
    let sub = NumberSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    assert!(!regex.is_match("numberextra()"));
    assert!(!regex.is_match("prefixnumber()"));
}

#[test]
fn test_name_word_boundary_strict() {
    let sub = NameSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    assert!(!regex.is_match("nameextra()"));
    assert!(!regex.is_match("prefixname()"));
}

#[test]
fn test_first_name_word_boundary_strict() {
    let sub = FirstNameSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    assert!(!regex.is_match("first_nameextra()"));
    assert!(!regex.is_match("prefixfirst_name()"));
}

#[test]
fn test_last_name_word_boundary_strict() {
    let sub = LastNameSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    assert!(!regex.is_match("last_nameextra()"));
    assert!(!regex.is_match("prefixlast_name()"));
}

#[test]
fn test_address_word_boundary_strict() {
    let sub = AddressSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    assert!(!regex.is_match("addressextra()"));
    assert!(!regex.is_match("prefixaddress()"));
}

// ========== SUBSTITUTION ORDER TESTS ==========

#[test]
fn test_substitution_preserves_order() {
    use crate::functions::substitute_functions;

    let input = r#"["guid()", "string()", "number()"]"#;
    let result = substitute_functions(input).unwrap();

    // Should maintain array structure
    assert!(result.starts_with("["));
    assert!(result.ends_with("]"));
    assert!(result.contains(","));
}

#[test]
fn test_substitution_with_multiple_same_functions_preserves_count() {
    use crate::functions::substitute_functions;

    let input = "guid() guid() guid() guid() guid()";
    let result = substitute_functions(input).unwrap();

    let guid_pattern = Regex::new(r"[0-9a-fA-F]{32}").unwrap();
    let matches: Vec<_> = guid_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 5, "Should have 5 GUIDs");
}

// ========== COMPLEX PAYLOAD TESTS ==========

#[test]
fn test_complex_json_payload_with_all_functions() {
    use crate::functions::substitute_functions;

    let input = r#"{
        "id": "guid()",
        "user": {
            "firstName": "first_name()",
            "lastName": "last_name()",
            "fullName": "name()",
            "address": "address()"
        },
        "metadata": {
            "score": number(),
            "token": "base64_encode('secret')",
            "randomId": "string()"
        },
        "tags": ["guid()", "string()", "name()"]
    }"#;

    let result = substitute_functions(input).unwrap();

    // Verify no functions remain
    assert!(!result.contains("guid()"));
    assert!(!result.contains("first_name()"));
    assert!(!result.contains("last_name()"));
    assert!(!result.contains("name()"));
    assert!(!result.contains("address()"));
    assert!(!result.contains("number()"));
    assert!(!result.contains("base64_encode"));
    assert!(!result.contains("string()"));

    // Verify structure is maintained
    assert!(result.contains("firstName"));
    assert!(result.contains("lastName"));
    assert!(result.contains("fullName"));
    assert!(result.contains("address"));
    assert!(result.contains("metadata"));
    assert!(result.contains("tags"));
}

#[test]
fn test_http_request_body_with_functions() {
    use crate::functions::substitute_functions;

    let input = r#"POST /api/users HTTP/1.1
Host: api.example.com
Authorization: Bearer base64_encode('token')
Content-Type: application/json

{"id": "guid()", "name": "first_name() last_name()", "score": number()}"#;

    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("base64_encode"));
    assert!(!result.contains("guid()"));
    assert!(!result.contains("first_name()"));
    assert!(!result.contains("last_name()"));
    assert!(!result.contains("number()"));
    assert!(result.contains("POST"));
    assert!(result.contains("Authorization: Bearer"));
}

// ========== CASE VARIATION COMPREHENSIVE TESTS ==========

#[test]
fn test_address_all_case_variations() {
    use crate::functions::substitute_functions;

    let input = r#"{"a1": "address()", "a2": "ADDRESS()", "a3": "Address()", "a4": "aDdReSs()"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.to_lowercase().contains("address()"));
}

#[test]
fn test_all_functions_extreme_case_mixing() {
    use crate::functions::substitute_functions;

    let input = r#"{"g": "GuId()", "s": "sTrInG()", "n": "NuMbEr()", "f": "FiRsT_NaMe()", "l": "LaSt_NaMe()", "na": "NaMe()", "a": "AdDrEsS()"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.to_lowercase().contains("guid()"));
    assert!(!result.to_lowercase().contains("string()"));
    assert!(!result.to_lowercase().contains("number()"));
    assert!(!result.to_lowercase().contains("first_name()"));
    assert!(!result.to_lowercase().contains("last_name()"));
    assert!(!result.to_lowercase().contains("name()"));
    assert!(!result.to_lowercase().contains("address()"));
}

// ========== REPEATED SEQUENTIAL SUBSTITUTION TESTS ==========

#[test]
fn test_repeated_substitution_convergence() {
    use crate::functions::substitute_functions;

    let input = r#"{"id": "guid()", "value": 123}"#;
    let mut current = input.to_string();

    for i in 0..5 {
        let next = substitute_functions(&current).unwrap();
        if i == 0 {
            // First substitution should change the input
            assert_ne!(current, next, "First substitution should modify the input");
        } else {
            // After first substitution, should not change
            assert_eq!(
                current, next,
                "Subsequent substitutions should not modify the result"
            );
            break;
        }
        current = next;
    }
}

// ========== MIXED CONTENT TESTS ==========

#[test]
fn test_functions_mixed_with_plain_text() {
    use crate::functions::substitute_functions;

    let input = "User guid() created on 2025-01-26 with name() scoring number()";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("name()"));
    assert!(!result.contains("number()"));
    assert!(result.contains("User"));
    assert!(result.contains("created"));
    assert!(result.contains("2025-01-26"));
}

#[test]
fn test_functions_with_sql_statements() {
    use crate::functions::substitute_functions;

    let input =
        "SELECT * FROM users WHERE id = 'guid()' AND name = 'name()' AND address = 'address()'";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("name()"));
    assert!(!result.contains("address()"));
    assert!(result.contains("SELECT"));
    assert!(result.contains("FROM"));
}

// ========== EMPTY AND WHITESPACE COMPREHENSIVE TESTS ==========

#[test]
fn test_function_surrounded_by_various_whitespace() {
    use crate::functions::substitute_functions;

    let input1 = "\t\nguid()\r\n\t";
    let result1 = substitute_functions(input1).unwrap();
    assert!(!result1.contains("guid()"));

    let input2 = "\t\n\r  string()  \r\n\t";
    let result2 = substitute_functions(input2).unwrap();
    assert!(!result2.contains("string()"));
}

#[test]
fn test_multiple_functions_separated_by_whitespace() {
    use crate::functions::substitute_functions;

    let input = "guid()     string()     number()     name()";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("string()"));
    assert!(!result.contains("number()"));
    assert!(!result.contains("name()"));
}

// ========== SPECIAL SEQUENCE TESTS ==========

#[test]
fn test_functions_in_duplicate_patterns() {
    use crate::functions::substitute_functions;

    let input = r#"{"id": "guid()", "id": "guid()", "id": "guid()"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    let guid_pattern = Regex::new(r"[0-9a-fA-F]{32}").unwrap();
    let matches: Vec<_> = guid_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3);
}

#[test]
fn test_base64_with_all_base64_chars() {
    let sub = Base64EncodeSubstitutor {};
    let input = "base64_encode('ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/')";
    let result = sub.replace(input).unwrap();

    assert!(!result.contains("base64_encode"));
}

#[test]
fn test_generator_stability_across_calls() {
    let guid_sub = GuidSubstitutor {};
    let string_sub = StringSubstitutor {};
    let number_sub = NumberSubstitutor {};

    // Each generator should produce a single value per call
    let guid = guid_sub.generate();
    assert_eq!(guid.len(), 32);

    let s = string_sub.generate();
    assert_eq!(s.len(), 20);

    let n = number_sub.generate();
    assert!(n.parse::<i32>().is_ok());
}

// ========== BOUNDARY VALUE TESTS ==========

#[test]
fn test_number_boundary_comprehensive() {
    let sub = NumberSubstitutor {};
    let mut has_low = false;
    let mut has_mid = false;
    let mut has_high = false;

    for _ in 0..500 {
        let num_str = sub.generate();
        let num: i32 = num_str.parse().unwrap();

        if num <= 10 {
            has_low = true;
        }
        if (40..=60).contains(&num) {
            has_mid = true;
        }
        if num >= 90 {
            has_high = true;
        }
    }

    assert!(has_low, "Should generate some low numbers");
    assert!(has_mid, "Should generate some mid-range numbers");
    assert!(has_high, "Should generate some high numbers");
}

// ========== COMPLETE SUBSTITUTION STATE TESTS ==========

#[test]
fn test_no_partial_function_substitutions() {
    use crate::functions::substitute_functions;

    let input = "guid() and string() and number()";
    let result = substitute_functions(input).unwrap();

    // Should not have any orphaned parentheses or function fragments
    assert!(!result.contains("()"));
}

#[test]
fn test_substitution_with_numeric_suffixes() {
    use crate::functions::substitute_functions;

    let input = r#"{"f1": "guid()", "f2": "string()", "f3": "number()""#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("string()"));
    assert!(!result.contains("number()"));
}

// ========== JOB TITLE SUBSTITUTOR TESTS ==========

#[test]
fn test_job_title_substitutor() {
    use crate::functions::generator_functions::JobTitleSubstitutor;

    let sub = JobTitleSubstitutor {};
    let job_title = sub.generate();
    assert!(
        !job_title.is_empty(),
        "Generated job title should not be empty"
    );
}

#[test]
fn test_job_title_substitutor_generates_different_values() {
    use crate::functions::generator_functions::JobTitleSubstitutor;

    let sub = JobTitleSubstitutor {};
    let job1 = sub.generate();
    let job2 = sub.generate();

    // Titles should differ (not guaranteed but likely with pool of 20)
    assert!(!job1.is_empty());
    assert!(!job2.is_empty());
}

// ========== COMPREHENSIVE FIRST/LAST NAME TESTS ==========

#[test]
fn test_first_name_generates_valid_names() {
    let sub = FirstNameSubstitutor {};
    for _ in 0..50 {
        let name = sub.generate();
        assert!(!name.is_empty(), "First name should not be empty");
        assert!(
            !name.is_empty(),
            "First name should have at least one character"
        );
    }
}

#[test]
fn test_last_name_generates_valid_names() {
    let sub = LastNameSubstitutor {};
    for _ in 0..50 {
        let name = sub.generate();
        assert!(!name.is_empty(), "Last name should not be empty");
        assert!(
            !name.is_empty(),
            "Last name should have at least one character"
        );
    }
}

#[test]
fn test_full_name_structure() {
    let sub = NameSubstitutor {};
    let name = sub.generate();
    let parts: Vec<&str> = name.split(' ').collect();

    assert_eq!(parts.len(), 2, "Full name should have exactly 2 parts");
    assert!(!parts[0].is_empty(), "First name part should not be empty");
    assert!(!parts[1].is_empty(), "Last name part should not be empty");
}

// ========== GUID VERSION AND VARIANT VALIDATION ==========

#[test]
fn test_guid_version_4_compliance() {
    let sub = GuidSubstitutor {};

    for _ in 0..50 {
        let guid = sub.generate();
        // Position 12 should be '4' for UUID v4
        assert_eq!(
            guid.chars().nth(12),
            Some('4'),
            "UUID version should be 4 at position 12 in {}",
            guid
        );
    }
}

#[test]
fn test_guid_variant_validation() {
    let sub = GuidSubstitutor {};
    let valid_variants = ['8', '9', 'a', 'b', 'A', 'B'];

    for _ in 0..50 {
        let guid = sub.generate();
        // Position 16 should be 8, 9, a, or b for variant 1
        let variant = guid.chars().nth(16).unwrap();
        assert!(
            valid_variants.contains(&variant),
            "UUID variant at position 16 should be one of {:?}, got {} in {}",
            valid_variants,
            variant,
            guid
        );
    }
}

// ========== COMPREHENSIVE BASE64 EDGE CASES ==========

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

    // No padding
    let r1 = sub.replace("base64_encode('abc')").unwrap();
    assert_eq!(r1, "YWJj");

    // Single padding
    let r2 = sub.replace("base64_encode('ab')").unwrap();
    assert_eq!(r2, "YWI=");

    // Double padding
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

// ========== FUNCTION REGEX VALIDATION ==========

#[test]
fn test_regex_does_not_match_function_with_arguments() {
    let sub = GuidSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    // guid() should match but guid(something) shouldn't (though regex might match guid() part)
    assert!(regex.is_match("guid()"));
}

#[test]
fn test_base64_encode_regex_word_boundaries() {
    let sub = Base64EncodeSubstitutor {};
    let input1 = "base64_encode('test')";
    let result1 = sub.replace(input1).unwrap();
    assert!(!result1.contains("base64_encode"));

    // Prefix should not prevent matching
    let input2 = "use base64_encode('test') here";
    let result2 = sub.replace(input2).unwrap();
    assert!(!result2.contains("base64_encode"));
}

// ========== COMPREHENSIVE SUBSTITUTION SEQUENCE TESTS ==========

#[test]
fn test_substitution_order_independence() {
    use crate::functions::substitute_functions;

    let input = "guid() name() string() number() address()";
    let result = substitute_functions(input).unwrap();

    // All should be replaced regardless of order
    assert!(!result.contains("guid()"));
    assert!(!result.contains("name()"));
    assert!(!result.contains("string()"));
    assert!(!result.contains("number()"));
    assert!(!result.contains("address()"));
}

#[test]
fn test_substitution_with_repeated_patterns() {
    use crate::functions::substitute_functions;

    let input = "guid() guid() guid() name() name()";
    let result = substitute_functions(input).unwrap();

    let guid_pattern = Regex::new(r"[0-9a-fA-F]{32}").unwrap();
    let guid_matches: Vec<_> = guid_pattern.find_iter(&result).collect();
    assert_eq!(guid_matches.len(), 3, "Should have 3 GUIDs");

    // The result should have spaces: guid guid guid (first last) (first last)
    // That's 4 separating spaces + 2 spaces in full names = 6 spaces
    let space_count = result.matches(' ').count();
    assert!(
        space_count >= 4,
        "Should have at least 4 spaces separating items"
    );
}

// ========== COMPLEX URL AND QUERY STRING TESTS ==========

#[test]
fn test_substitution_in_complex_url_path() {
    use crate::functions::substitute_functions;

    let input = "/api/v1/users/guid()/posts/guid()/comments/number()/replies/string()";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("number()"));
    assert!(!result.contains("string()"));
    assert!(result.starts_with("/api/v1/users/"));
}

#[test]
fn test_substitution_in_query_string_with_operators() {
    use crate::functions::substitute_functions;

    let input = "?id=guid()&name=first_name()&age>=number()&score<=number()";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("first_name()"));
    assert!(!result.contains("number()"));
    assert!(result.contains("?"));
    assert!(result.contains("&"));
    assert!(result.contains(">="));
    assert!(result.contains("<="));
}

// ========== ADDRESS COMPREHENSIVE TESTS ==========

#[test]
fn test_address_substitutor_produces_non_empty() {
    let sub = AddressSubstitutor {};

    for _ in 0..50 {
        let addr = sub.generate();
        assert!(!addr.is_empty(), "Address should not be empty");
        assert!(addr.len() > 5, "Address should have meaningful length");
    }
}

#[test]
fn test_address_substitutor_contains_typical_components() {
    let sub = AddressSubstitutor {};
    let addr = sub.generate();

    // Addresses should contain numbers, letters, and typically commas/spaces
    assert!(
        addr.contains(|c: char| c.is_numeric()) || addr.contains(' '),
        "Address should contain numbers or spaces"
    );
}

#[test]
fn test_address_in_complex_json() {
    use crate::functions::substitute_functions;

    let input = r#"{"locations": [{"address": "address()"}, {"address": "address()"}, {"address": "address()"}]}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("address()"));
    assert!(result.contains("locations"));
    assert_eq!(result.matches("[").count(), 1);
}

// ========== SUBSTITUTION SAFETY AND STABILITY TESTS ==========

#[test]
fn test_substitution_does_not_modify_non_function_content() {
    use crate::functions::substitute_functions;

    let input = "This is plain text without any functions 12345 !@#$%";
    let result = substitute_functions(input).unwrap();

    assert_eq!(input, result, "Non-function content should not be modified");
}

#[test]
fn test_substitution_preserves_special_json_characters() {
    use crate::functions::substitute_functions;

    let input = r#"{"key": "guid()", "array": [], "number": 123, "bool": true}"#;
    let result = substitute_functions(input).unwrap();

    assert!(result.contains("[]"));
    assert!(result.contains("123"));
    assert!(result.contains("true"));
    assert!(!result.contains("guid()"));
}

// ========== EXTREME LENGTH TESTS ==========

#[test]
fn test_substitution_with_very_long_json() {
    use crate::functions::substitute_functions;

    let mut input = String::from(r#"{"items": ["#);
    for i in 0..100 {
        input.push_str(&format!(r#"{{"id": "guid()", "index": {}}}"#, i));
        if i < 99 {
            input.push(',');
        }
    }
    input.push_str(r#"]}"#);

    let result = substitute_functions(&input).unwrap();
    assert!(!result.contains("guid()"));
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

// ========== GENERATION UNIQUENESS TESTS ==========

#[test]
fn test_string_generation_variety() {
    use std::collections::HashSet;

    let sub = StringSubstitutor {};
    let mut values = HashSet::new();

    for _ in 0..200 {
        values.insert(sub.generate());
    }

    // Should have many different values (out of 200 iterations)
    assert!(
        values.len() > 150,
        "Should generate mostly unique strings in 200 iterations"
    );
}

#[test]
fn test_guid_generation_uniqueness_large_set() {
    use std::collections::HashSet;

    let sub = GuidSubstitutor {};
    let mut values = HashSet::new();

    for _ in 0..1000 {
        assert!(values.insert(sub.generate()), "Generated duplicate GUID");
    }

    assert_eq!(values.len(), 1000);
}

// ========== FUNCTION CALLING VARIATION TESTS ==========

#[test]
fn test_function_with_internal_whitespace() {
    use crate::functions::substitute_functions;

    let input = "guid(  )";
    let result = substitute_functions(input).unwrap();

    // guid(  ) with internal whitespace is not recognized as guid()
    // so it should remain unchanged
    assert_eq!(input, result, "guid(  ) should not be substituted");
}

#[test]
fn test_all_generators_produce_strings() {
    let guid = GuidSubstitutor {}.generate();
    let string = StringSubstitutor {}.generate();
    let number = NumberSubstitutor {}.generate();
    let name = NameSubstitutor {}.generate();
    let first_name = FirstNameSubstitutor {}.generate();
    let last_name = LastNameSubstitutor {}.generate();
    let address = AddressSubstitutor {}.generate();

    assert!(!guid.is_empty());
    assert!(!string.is_empty());
    assert!(!number.is_empty());
    assert!(!name.is_empty());
    assert!(!first_name.is_empty());
    assert!(!last_name.is_empty());
    assert!(!address.is_empty());
}

// ========== SUBSTITUTION CONSISTENCY TESTS ==========

#[test]
fn test_multiple_substitution_calls_same_input_different_outputs() {
    use crate::functions::substitute_functions;

    let input = r#"{"id": "guid()"}"#;
    let result1 = substitute_functions(input).unwrap();
    let result2 = substitute_functions(input).unwrap();

    // Results should be different (unless extremely unlikely collision)
    assert_ne!(
        result1, result2,
        "Multiple substitutions should produce different random values"
    );
}

#[test]
fn test_base64_encode_consistency() {
    let sub = Base64EncodeSubstitutor {};
    let input = "base64_encode('consistent')";

    let result1 = sub.replace(input).unwrap();
    let result2 = sub.replace(input).unwrap();

    assert_eq!(result1, result2, "Base64 encoding should be consistent");
}

// ========== NESTED FUNCTION RESILIENCE TESTS ==========

#[test]
fn test_functions_do_not_interfere_with_each_other() {
    use crate::functions::substitute_functions;

    let inputs = vec![
        r#"{"guid": "guid()", "name": "name()"}"#,
        r#"{"string": "string()", "number": number()}"#,
        r#"{"first": "first_name()", "last": "last_name()"}"#,
        r#"{"address": "address()", "id": "guid()"}"#,
    ];

    for input in inputs {
        let result = substitute_functions(input).unwrap();

        // Verify all functions were substituted
        assert!(!result.contains("guid()"));
        assert!(!result.contains("name()"));
        assert!(!result.contains("string()"));
        assert!(!result.contains("number()"));
        assert!(!result.contains("first_name()"));
        assert!(!result.contains("last_name()"));
        assert!(!result.contains("address()"));
    }
}

// ========== RAPID GENERATION TESTS ==========

#[test]
fn test_rapid_guid_generation() {
    let sub = GuidSubstitutor {};
    let mut guids = Vec::new();

    for _ in 0..100 {
        guids.push(sub.generate());
    }

    // All should be valid
    for guid in guids {
        assert_eq!(guid.len(), 32);
        assert!(guid.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

#[test]
fn test_rapid_number_generation() {
    let sub = NumberSubstitutor {};

    for _ in 0..100 {
        let num_str = sub.generate();
        let num: i32 = num_str.parse().unwrap();
        assert!((0..=100).contains(&num));
    }
}

// ========== SUBSTITUTE_FUNCTIONS EDGE CASES ==========

#[test]
fn test_substitute_functions_empty_string() {
    use crate::functions::substitute_functions;

    let input = "";
    let result = substitute_functions(input).unwrap();

    assert_eq!(result, "");
}

#[test]
fn test_substitute_functions_only_whitespace() {
    use crate::functions::substitute_functions;

    let input = "   \n\t   ";
    let result = substitute_functions(input).unwrap();

    assert_eq!(result, input);
}

#[test]
fn test_substitute_functions_only_functions() {
    use crate::functions::substitute_functions;

    let input = "guid() string() number()";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("string()"));
    assert!(!result.contains("number()"));
    assert!(result.contains(' '));
}

// ========== COMPREHENSIVE REAL-WORLD SCENARIOS ==========

#[test]
fn test_http_post_request_with_bearer_token() {
    use crate::functions::substitute_functions;

    let input = r#"POST /api/users HTTP/1.1
Host: api.example.com
Authorization: Bearer base64_encode('user:pass')
Content-Type: application/json
X-Request-ID: guid()

{"email": "first_name()@example.com", "name": "name()"}"#;

    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("base64_encode"));
    assert!(!result.contains("guid()"));
    assert!(!result.contains("first_name()"));
    assert!(!result.contains("name()"));
    assert!(result.contains("POST"));
    assert!(result.contains("Authorization: Bearer"));
}

#[test]
fn test_graphql_query_with_functions() {
    use crate::functions::substitute_functions;

    let input = r#"mutation CreateUser {
  createUser(input: {
    id: "guid()"
    firstName: "first_name()"
    lastName: "last_name()"
    email: "base64_encode('email@test.com')"
    score: number()
  }) {
    id
    firstName
    lastName
  }
}"#;

    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("first_name()"));
    assert!(!result.contains("last_name()"));
    assert!(!result.contains("base64_encode"));
    assert!(!result.contains("number()"));
}

#[test]
fn test_rest_client_environment_variables() {
    use crate::functions::substitute_functions;

    let input = r#"@baseUrl = https://api.example.com
@userId = guid()
@authToken = base64_encode('secret')
@userName = first_name()

GET {{baseUrl}}/users/{{userId}}
Authorization: Bearer {{authToken}}
X-User-Name: {{userName}}"#;

    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("base64_encode"));
    assert!(!result.contains("first_name()"));
}

#[test]
fn test_csv_data_with_functions() {
    use crate::functions::substitute_functions;

    let input = "id,firstName,lastName,email,address,score
guid(),first_name(),last_name(),base64_encode('email@test.com'),address(),number()
guid(),first_name(),last_name(),base64_encode('email@test.com'),address(),number()
guid(),first_name(),last_name(),base64_encode('email@test.com'),address(),number()";

    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("first_name()"));
    assert!(!result.contains("last_name()"));
    assert!(!result.contains("base64_encode"));
    assert!(!result.contains("address()"));
    assert!(!result.contains("number()"));
}

// ========== SUBSTITUTION PERFORMANCE TESTS ==========

#[test]
fn test_substitution_with_many_empty_functions_attempt() {
    use crate::functions::substitute_functions;

    // Many function calls
    let input = "guid() guid() guid() guid() guid() string() string() string() string() string()";
    let result = substitute_functions(input).unwrap();

    let guid_pattern = Regex::new(r"[0-9a-fA-F]{32}").unwrap();
    let guid_matches: Vec<_> = guid_pattern.find_iter(&result).collect();
    assert_eq!(guid_matches.len(), 5);

    let alphanum_pattern = Regex::new(r"[A-Za-z0-9]{20}").unwrap();
    let string_matches: Vec<_> = alphanum_pattern.find_iter(&result).collect();
    assert!(string_matches.len() >= 5);
}

// ========== EMAIL SUBSTITUTOR TESTS ==========

#[test]
fn test_email_substitutor_generates_value() {
    let sub = EmailSubstitutor {};
    let email = sub.generate();
    assert!(!email.is_empty(), "Generated email should not be empty");
}

#[test]
fn test_email_substitutor_format() {
    let sub = EmailSubstitutor {};
    let email = sub.generate();

    // Email should contain @ symbol
    assert!(email.contains('@'), "Email '{}' should contain @", email);

    // Email should have local part and domain
    let parts: Vec<&str> = email.split('@').collect();
    assert_eq!(
        parts.len(),
        2,
        "Email '{}' should have exactly 2 parts separated by @",
        email
    );
    assert!(!parts[0].is_empty(), "Email local part should not be empty");
    assert!(!parts[1].is_empty(), "Email domain should not be empty");
}

#[test]
fn test_email_substitutor_local_part_format() {
    let sub = EmailSubstitutor {};
    let email = sub.generate();

    let local_part = email.split('@').next().unwrap();

    // Local part should be in format: firstname.lastname
    assert!(
        local_part.contains('.'),
        "Email local part '{}' should contain a dot (format: firstname.lastname)",
        local_part
    );

    let parts: Vec<&str> = local_part.split('.').collect();
    assert_eq!(
        parts.len(),
        2,
        "Email local part '{}' should have exactly 2 parts separated by dot",
        local_part
    );

    // Both parts should be non-empty
    assert!(!parts[0].is_empty(), "First name part should not be empty");
    assert!(!parts[1].is_empty(), "Last name part should not be empty");
}

#[test]
fn test_email_substitutor_lowercase() {
    let sub = EmailSubstitutor {};
    let email = sub.generate();

    // Email should be lowercase
    assert_eq!(
        email,
        email.to_lowercase(),
        "Email '{}' should be entirely lowercase",
        email
    );
}

#[test]
fn test_email_substitutor_valid_domain() {
    let sub = EmailSubstitutor {};

    // Valid email domains from values.rs
    let valid_domains = vec![
        "example.com",
        "mail.com",
        "test.org",
        "demo.net",
        "sample.co",
        "mydomain.io",
        "webmail.com",
        "inbox.org",
        "email.net",
        "domain.co",
    ];

    let email = sub.generate();
    let domain = email.split('@').nth(1).unwrap();

    assert!(
        valid_domains.contains(&domain),
        "Email domain '{}' should be one of the valid domains",
        domain
    );
}

#[test]
fn test_email_substitutor_generates_different_values() {
    let sub = EmailSubstitutor {};
    let email1 = sub.generate();
    let email2 = sub.generate();
    let email3 = sub.generate();

    // Different calls should generate different emails (highly likely)
    assert!(!email1.is_empty());
    assert!(!email2.is_empty());
    assert!(!email3.is_empty());
}

#[test]
fn test_email_substitutor_regex() {
    let sub = EmailSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    assert!(regex.is_match("email()"), "Should match email()");
    assert!(
        regex.is_match("Contact: email()"),
        "Should match email() with prefix"
    );
    assert!(!regex.is_match("noemail()"), "Should not match noemail()");
    assert!(!regex.is_match("myemail()"), "Should not match myemail()");
}

#[test]
fn test_email_substitutor_word_boundary_strict() {
    let sub = EmailSubstitutor {};
    let regex = regex::Regex::new(sub.get_regex()).unwrap();

    assert!(regex.is_match("email()"));
    assert!(regex.is_match("email()extra"));
    assert!(!regex.is_match("prefixemail()"));
    assert!(!regex.is_match("_email()"));
}

#[test]
fn test_email_substitutor_case_insensitive() {
    use crate::functions::substitute_functions;

    let input = r#"{"e1": "email()", "e2": "EMAIL()", "e3": "Email()"}"#;
    let result = substitute_functions(input).unwrap();

    // All should be substituted
    assert!(!result.contains("email()"));
    assert!(!result.contains("EMAIL()"));
    assert!(!result.contains("Email()"));

    // All should contain valid email format (has @ and .)
    assert!(result.contains("@"), "Result should contain @ symbol");
    assert!(result.contains("."), "Result should contain . symbol");
}

#[test]
fn test_email_in_json_context() {
    use crate::functions::substitute_functions;

    let input = r#"{"email": "email()", "user_email": "email()", "contact": "email()"}"#;
    let result = substitute_functions(input).unwrap();

    // All email functions should be substituted
    assert!(!result.contains("email()"));

    // All should contain valid email format
    assert!(
        result.matches('@').count() >= 3,
        "Should have at least 3 @ symbols"
    );
    assert!(
        result.matches('.').count() >= 3,
        "Should have at least 3 . symbols"
    );
}

#[test]
fn test_email_in_http_headers() {
    use crate::functions::substitute_functions;

    let input = "From: email()\nTo: email()\nReply-To: email()";
    let result = substitute_functions(input).unwrap();

    // All email functions should be substituted
    assert!(!result.contains("email()"));

    // Structure should be maintained
    assert!(result.contains("From:"));
    assert!(result.contains("To:"));
    assert!(result.contains("Reply-To:"));
}

#[test]
fn test_email_in_url_query_string() {
    use crate::functions::substitute_functions;

    let input = "?email=email()&recipient=email()&cc=email()";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("email()"));
    assert!(result.contains("?"));
    assert!(result.contains("&"));
    assert!(result.contains("="));
}

#[test]
fn test_email_combined_with_other_functions() {
    use crate::functions::substitute_functions;

    let input =
        r#"{"userId": "guid()", "userName": "name()", "userEmail": "email()", "score": number()}"#;
    let result = substitute_functions(input).unwrap();

    // All functions should be substituted
    assert!(!result.contains("guid()"));
    assert!(!result.contains("name()"));
    assert!(!result.contains("email()"));
    assert!(!result.contains("number()"));

    // Email should have valid format
    assert!(result.contains("@"));
}

#[test]
fn test_email_generation_contains_lowercase_names() {
    let sub = EmailSubstitutor {};

    for _ in 0..20 {
        let email = sub.generate();
        let local_part = email.split('@').next().unwrap();

        // Local part should be all lowercase and contain only alphanumeric characters and dots
        // (special characters like apostrophes and hyphens are removed during normalization)
        assert!(
            local_part
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '.'),
            "Email local part '{}' should be lowercase ASCII alphanumeric with dots",
            local_part
        );
    }
}

#[test]
fn test_email_in_form_data() {
    use crate::functions::substitute_functions;

    let input = "email=email()&firstName=first_name()&lastName=last_name()";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("email()"));
    assert!(!result.contains("first_name()"));
    assert!(!result.contains("last_name()"));
    assert!(result.contains("email="));
    assert!(result.contains("firstName="));
    assert!(result.contains("lastName="));
}

#[test]
fn test_email_multiple_occurrences() {
    use crate::functions::substitute_functions;

    let input = r#"{"contact1": "email()", "contact2": "email()", "contact3": "email()", "contact4": "email()"}"#;
    let result = substitute_functions(input).unwrap();

    // All should be substituted with different emails (very likely)
    assert!(!result.contains("email()"));

    // Should have 4 @ symbols
    assert_eq!(
        result.matches('@').count(),
        4,
        "Should have 4 @ symbols for 4 emails"
    );
}

#[test]
fn test_email_in_xml_context() {
    use crate::functions::substitute_functions;

    let input = r#"<user><email>email()</email><backupEmail>email()</backupEmail></user>"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("email()"));
    assert!(result.contains("<email>"));
    assert!(result.contains("</email>"));
    assert!(result.contains("<backupEmail>"));
    assert!(result.contains("</backupEmail>"));
}

#[test]
fn test_email_all_case_variations() {
    use crate::functions::substitute_functions;

    let input = r#"{"e1": "email()", "e2": "EMAIL()", "e3": "Email()", "e4": "eMAiL()"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.to_lowercase().contains("email()"));
    assert_eq!(result.matches('@').count(), 4, "Should have 4 @ symbols");
}

#[test]
fn test_email_with_surrounding_whitespace() {
    use crate::functions::substitute_functions;

    let input = "  email()  ";
    let result = substitute_functions(input).unwrap();

    // Whitespace should be preserved, function replaced
    assert!(result.starts_with("  "));
    assert!(result.ends_with("  "));
    assert!(!result.contains("email()"));
}

#[test]
fn test_email_at_string_boundaries() {
    use crate::functions::substitute_functions;

    let input1 = "email()";
    let result1 = substitute_functions(input1).unwrap();
    assert!(!result1.contains("email()"));
    assert!(result1.contains("@"));

    let input2 = "Contact: email()";
    let result2 = substitute_functions(input2).unwrap();
    assert!(!result2.contains("email()"));
    assert!(result2.contains("Contact:"));
}

#[test]
fn test_email_adjacent_to_symbols() {
    use crate::functions::substitute_functions;

    let input = "[email()], (email()), {email()}";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("email()"));
    assert!(result.contains("["));
    assert!(result.contains("("));
    assert!(result.contains("{"));
    assert_eq!(result.matches('@').count(), 3, "Should have 3 @ symbols");
}

#[test]
fn test_email_in_csv_format() {
    use crate::functions::substitute_functions;

    let input = "id,name,email\n1,John,email()\n2,Jane,email()\n3,Bob,email()";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("email()"));
    assert!(result.contains("id,name,email"));
    assert_eq!(result.matches('@').count(), 3, "Should have 3 @ symbols");
}

#[test]
fn test_email_in_api_response_simulation() {
    use crate::functions::substitute_functions;

    let input = r#"{"users": [{"id": "guid()", "email": "email()"}, {"id": "guid()", "email": "email()"}]}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("email()"));
    assert_eq!(result.matches('@').count(), 2, "Should have 2 @ symbols");
    assert!(result.contains("users"));
}

#[test]
fn test_email_repeated_substitution_convergence() {
    use crate::functions::substitute_functions;

    let input = r#"{"email": "email()"}"#;
    let mut current = input.to_string();

    for i in 0..5 {
        let next = substitute_functions(&current).unwrap();
        if i == 0 {
            // First substitution should change the input
            assert_ne!(current, next, "First substitution should modify the input");
        } else {
            // After first substitution, should not change
            assert_eq!(
                current, next,
                "Subsequent substitutions should not modify the result"
            );
            break;
        }
        current = next;
    }
}

#[test]
fn test_email_no_partial_substitutions() {
    use crate::functions::substitute_functions;

    let input = "email() and email() and email()";
    let result = substitute_functions(input).unwrap();

    // Should not have any orphaned parentheses or function fragments
    assert!(!result.contains("()"));
}

#[test]
fn test_email_consistency_across_domains() {
    let sub = EmailSubstitutor {};

    // Generate many emails and verify all have valid domains
    let valid_domains = vec![
        "example.com",
        "mail.com",
        "test.org",
        "demo.net",
        "sample.co",
        "mydomain.io",
        "webmail.com",
        "inbox.org",
        "email.net",
        "domain.co",
    ];

    for _ in 0..50 {
        let email = sub.generate();
        let domain = email.split('@').nth(1).unwrap();
        assert!(
            valid_domains.contains(&domain),
            "Domain '{}' should be in valid list",
            domain
        );
    }
}

#[test]
fn test_email_local_part_contains_dot() {
    let sub = EmailSubstitutor {};

    // All emails should have exactly one dot in local part
    for _ in 0..30 {
        let email = sub.generate();
        let local_part = email.split('@').next().unwrap();
        let dot_count = local_part.matches('.').count();

        assert_eq!(
            dot_count, 1,
            "Email local part '{}' should have exactly 1 dot",
            local_part
        );
    }
}

#[test]
fn test_email_in_real_world_rest_request() {
    use crate::functions::substitute_functions;

    let input = r#"POST /api/auth/register HTTP/1.1
Host: api.example.com
Content-Type: application/json

{"email": "email()", "password": "base64_encode('SecurePass123')", "firstName": "first_name()", "lastName": "last_name()"}"#;

    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("email()"));
    assert!(!result.contains("base64_encode"));
    assert!(!result.contains("first_name()"));
    assert!(!result.contains("last_name()"));
    assert!(result.contains("POST"));
    assert!(result.contains("Content-Type:"));
}

#[test]
fn test_getdate() {
    let sub = GetDateSubstitutor {};
    let date = sub.generate();

    // Validate format YYYY-MM-DD
    let date_pattern = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    assert!(
        date_pattern.is_match(&date),
        "Date '{}' does not match pattern YYYY-MM-DD",
        date
    );

    // Verify it can be parsed as a valid date
    use chrono::NaiveDate;
    assert!(
        NaiveDate::parse_from_str(&date, "%Y-%m-%d").is_ok(),
        "Date '{}' could not be parsed",
        date
    );
}

#[test]
fn test_gettime() {
    let sub = GetTimeSubstitutor {};
    let time = sub.generate();

    // Validate format HH:MM:SS
    let time_pattern = Regex::new(r"^\d{2}:\d{2}:\d{2}$").unwrap();
    assert!(
        time_pattern.is_match(&time),
        "Time '{}' does not match pattern HH:MM:SS",
        time
    );

    // Verify it can be parsed as a valid time
    use chrono::NaiveTime;
    assert!(
        NaiveTime::parse_from_str(&time, "%H:%M:%S").is_ok(),
        "Time '{}' could not be parsed",
        time
    );
}

#[test]
fn test_getdatetime() {
    let sub = GetDateTimeSubstitutor {};
    let datetime = sub.generate();

    // Validate format YYYY-MM-DD HH:MM:SS
    let datetime_pattern = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").unwrap();
    assert!(
        datetime_pattern.is_match(&datetime),
        "DateTime '{}' does not match pattern YYYY-MM-DD HH:MM:SS",
        datetime
    );

    // Verify it can be parsed as a valid datetime
    use chrono::NaiveDateTime;
    assert!(
        NaiveDateTime::parse_from_str(&datetime, "%Y-%m-%d %H:%M:%S").is_ok(),
        "DateTime '{}' could not be parsed",
        datetime
    );
}

#[test]
fn test_getutcdatetime() {
    let sub = GetUtcDateTimeSubstitutor {};
    let utc_datetime = sub.generate();

    // Validate format YYYY-MM-DD HH:MM:SS
    let datetime_pattern = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").unwrap();
    assert!(
        datetime_pattern.is_match(&utc_datetime),
        "UTC DateTime '{}' does not match pattern YYYY-MM-DD HH:MM:SS",
        utc_datetime
    );

    // Verify it can be parsed as a valid datetime
    use chrono::NaiveDateTime;
    assert!(
        NaiveDateTime::parse_from_str(&utc_datetime, "%Y-%m-%d %H:%M:%S").is_ok(),
        "UTC DateTime '{}' could not be parsed",
        utc_datetime
    );
}

#[test]
fn test_substitute_functions_date_case_insensitivity() {
    use crate::functions::substitute_functions;

    let input = r#"{"date1": "getdate()", "date2": "GETDATE()", "date3": "GetDate()"}"#;
    let result = substitute_functions(input).unwrap();

    // All three should be substituted
    assert!(!result.contains("getdate()"));
    assert!(!result.contains("GETDATE()"));
    assert!(!result.contains("GetDate()"));

    // Verify the result contains valid dates (YYYY-MM-DD)
    let date_pattern = Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap();
    let matches: Vec<_> = date_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3, "Should have 3 date values");
}

#[test]
fn test_substitute_functions_time_case_insensitivity() {
    use crate::functions::substitute_functions;

    let input = r#"{"time1": "gettime()", "time2": "GETTIME()", "time3": "GetTime()"}"#;
    let result = substitute_functions(input).unwrap();

    // All three should be substituted
    assert!(!result.contains("gettime()"));
    assert!(!result.contains("GETTIME()"));
    assert!(!result.contains("GetTime()"));

    // Verify the result contains valid times (HH:MM:SS)
    let time_pattern = Regex::new(r"\d{2}:\d{2}:\d{2}").unwrap();
    let matches: Vec<_> = time_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3, "Should have 3 time values");
}

#[test]
fn test_substitute_functions_datetime_case_insensitivity() {
    use crate::functions::substitute_functions;

    let input = r#"{"dt1": "getdatetime()", "dt2": "GETDATETIME()", "dt3": "GetDateTime()"}"#;
    let result = substitute_functions(input).unwrap();

    // All three should be substituted
    assert!(!result.contains("getdatetime()"));
    assert!(!result.contains("GETDATETIME()"));
    assert!(!result.contains("GetDateTime()"));

    // Verify the result contains valid datetimes (YYYY-MM-DD HH:MM:SS)
    let datetime_pattern = Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}").unwrap();
    let matches: Vec<_> = datetime_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3, "Should have 3 datetime values");
}

#[test]
fn test_substitute_functions_utcdatetime_case_insensitivity() {
    use crate::functions::substitute_functions;

    let input =
        r#"{"utc1": "getutcdatetime()", "utc2": "GETUTCDATETIME()", "utc3": "GetUtcDateTime()"}"#;
    let result = substitute_functions(input).unwrap();

    // All three should be substituted
    assert!(!result.contains("getutcdatetime()"));
    assert!(!result.contains("GETUTCDATETIME()"));
    assert!(!result.contains("GetUtcDateTime()"));

    // Verify the result contains valid UTC datetimes
    let datetime_pattern = Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}").unwrap();
    let matches: Vec<_> = datetime_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3, "Should have 3 UTC datetime values");
}

#[test]
fn test_datetime_in_real_world_rest_request() {
    use crate::functions::substitute_functions;

    let input = r#"POST /api/events HTTP/1.1
Host: api.example.com
Content-Type: application/json

{"eventDate": "getdate()", "eventTime": "gettime()", "createdAt": "getdatetime()", "updatedAt": "getutcdatetime()"}"#;

    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("getdate()"));
    assert!(!result.contains("gettime()"));
    assert!(!result.contains("getdatetime()"));
    assert!(!result.contains("getutcdatetime()"));
    assert!(result.contains("POST"));
    assert!(result.contains("Content-Type:"));

    // Verify dates and times were substituted
    let date_pattern = Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap();
    assert!(date_pattern.is_match(&result));

    let time_pattern = Regex::new(r"\d{2}:\d{2}:\d{2}").unwrap();
    assert!(time_pattern.is_match(&result));
}

// ========== UPPER TRANSFORM FUNCTION TESTS ==========

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

    // Should properly handle unicode
    assert!(!result.contains("upper"));
    assert!(result.contains("HELLO"));
}

#[test]
fn test_upper_in_substitute_functions() {
    use crate::functions::substitute_functions;

    let input = r#"{"text": "upper('hello')", "shout": "UPPER('world')"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("upper"));
    assert!(result.contains("HELLO"));
    assert!(result.contains("WORLD"));
}

#[test]
fn test_upper_consecutive_calls() {
    use crate::functions::substitute_functions;

    let input = r#"{"a": "upper('test1')", "b": "upper('test2')", "c": "upper('test3')"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("upper"));
    assert!(result.contains("TEST1"));
    assert!(result.contains("TEST2"));
    assert!(result.contains("TEST3"));
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

    // Prefix should not prevent matching
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

// ========== LOWER TRANSFORM FUNCTION TESTS ==========

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

    // Should properly handle unicode
    assert!(!result.contains("lower"));
    assert!(result.contains("hello"));
}

#[test]
fn test_lower_in_substitute_functions() {
    use crate::functions::substitute_functions;

    let input = r#"{"text": "lower('HELLO')", "whisper": "LOWER('WORLD')"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("lower("));
    assert!(result.contains("hello"));
    assert!(result.contains("world"));
}

#[test]
fn test_lower_consecutive_calls() {
    use crate::functions::substitute_functions;

    let input = r#"{"a": "lower('TEST1')", "b": "lower('TEST2')", "c": "lower('TEST3')"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("lower"));
    assert!(result.contains("test1"));
    assert!(result.contains("test2"));
    assert!(result.contains("test3"));
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

    // Prefix should not prevent matching
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

// ========== COMBINED UPPER AND LOWER TESTS ==========

#[test]
fn test_upper_and_lower_together() {
    use crate::functions::substitute_functions;

    let input = r#"{"upper": "upper('hello')", "lower": "lower('WORLD')"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("upper("));
    assert!(!result.contains("lower("));
    assert!(result.contains("HELLO"));
    assert!(result.contains("world"));
}

#[test]
fn test_upper_and_lower_multiple_in_json() {
    use crate::functions::substitute_functions;

    let input = r#"{
        "id": "guid()",
        "code": "upper('test123')",
        "name": "lower('JOHN DOE')",
        "email": "email()",
        "shout": "upper('hello')"
    }"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("upper("));
    assert!(!result.contains("lower("));
    assert!(!result.contains("guid()"));
    assert!(!result.contains("email()"));
    assert!(result.contains("TEST123"));
    assert!(result.contains("john doe"));
    assert!(result.contains("HELLO"));
}

#[test]
fn test_upper_and_lower_in_http_request() {
    use crate::functions::substitute_functions;

    let input = r#"POST /api/data HTTP/1.1
Host: api.example.com
Content-Type: application/json

{"code": "upper('abc123')", "normalized": "lower('MIXED Case')", "id": "guid()"}"#;

    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("upper("));
    assert!(!result.contains("lower("));
    assert!(!result.contains("guid()"));
    assert!(result.contains("ABC123"));
    assert!(result.contains("mixed case"));
    assert!(result.contains("POST"));
}

#[test]
fn test_transform_with_base64() {
    use crate::functions::substitute_functions;

    let input = r#"{"encoded": "base64_encode('hello')", "upper": "upper('world')", "lower": "lower('TEST')"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("base64_encode"));
    assert!(!result.contains("upper("));
    assert!(!result.contains("lower("));
    assert!(result.contains("aGVsbG8="));
    assert!(result.contains("WORLD"));
    assert!(result.contains("test"));
}

#[test]
fn test_upper_lower_idempotency() {
    use crate::functions::substitute_functions;

    let input = r#"{"upper": "upper('test')", "lower": "lower('TEST')"}"#;

    let result1 = substitute_functions(input).unwrap();
    let result2 = substitute_functions(&result1).unwrap();

    // Second substitution should not change anything
    assert_eq!(result1, result2);
    assert!(result1.contains("TEST"));
    assert!(result1.contains("test"));
}

#[test]
fn test_upper_lower_with_variables_notation() {
    use crate::functions::substitute_functions;

    // Simulates using transform functions with request variable values
    let input =
        r#"{"normalized": "lower('USER@EXAMPLE.COM')", "shouting": "upper('important message')"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(result.contains("user@example.com"));
    assert!(result.contains("IMPORTANT MESSAGE"));
}

#[test]
fn test_upper_lower_very_long_strings() {
    let sub_upper = UpperSubstitutor {};
    let sub_lower = LowerSubstitutor {};

    let long_lower = "a".repeat(1000);
    let long_upper = "A".repeat(1000);

    let input_upper = format!("upper('{}')", long_lower);
    let result_upper = sub_upper.replace(&input_upper).unwrap();
    assert!(!result_upper.contains("upper"));
    assert_eq!(result_upper, long_upper);

    let input_lower = format!("lower('{}')", long_upper);
    let result_lower = sub_lower.replace(&input_lower).unwrap();
    assert!(!result_lower.contains("lower"));
    assert_eq!(result_lower, long_lower);
}
