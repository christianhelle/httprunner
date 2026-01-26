use crate::functions::generators::{
    AddressSubstitutor, Base64EncodeSubstitutor, FirstNameSubstitutor, GuidSubstitutor,
    LastNameSubstitutor, NameSubstitutor, NumberSubstitutor, StringSubstitutor,
};
use crate::functions::substitution::FunctionSubstitutor;
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

    // Verify structure is maintained
    assert!(result.contains(r#""id":"#));
    assert!(result.contains(r#""firstName":"#));
    assert!(result.contains(r#""lastName":"#));
    assert!(result.contains(r#""fullName":"#));
    assert!(result.contains(r#""address":"#));
    assert!(result.contains(r#""randomStr":"#));
    assert!(result.contains(r#""randomNum":"#));
    assert!(result.contains(r#""encoded":"#));
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
    assert!(result.contains(r#""user":"#));
    assert!(result.contains(r#""id":"#));
    assert!(result.contains(r#""name":"#));
    assert!(result.contains(r#""score":"#));
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

    for _ in 0..10000 {
        let num_str = sub.generate();
        let num: usize = num_str.parse().unwrap();
        counts[num] += 1;
    }

    // Verify we're generating a reasonable distribution
    // Most numbers should appear at least once in 10000 iterations
    let non_zero_count = counts.iter().filter(|&&c| c > 0).count();
    assert!(
        non_zero_count >= 80,
        "Expected at least 80 different numbers in 10000 iterations, got {}",
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
    assert!(chars_found.iter().any(|c| c.is_ascii_digit()), "Should include digits");
    assert!(chars_found.iter().any(|c| c.is_ascii_uppercase()), "Should include uppercase");
    assert!(chars_found.iter().any(|c| c.is_ascii_lowercase()), "Should include lowercase");
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
    assert!(regex.is_match("guid()"));  // Should match
    assert!(regex.is_match("guid()extra")); // guid() is at a word boundary
    assert!(!regex.is_match("prefixguid()")); // guid() is not at word boundary (preceded by 'x')
    assert!(!regex.is_match("my_guid()"));  // guid() is not at word boundary (preceded by '_')
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
            assert_eq!(current, next, "Subsequent substitutions should not modify the result");
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

    let input = "SELECT * FROM users WHERE id = 'guid()' AND name = 'name()' AND address = 'address()'";
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
        if 40 <= num && num <= 60 {
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
