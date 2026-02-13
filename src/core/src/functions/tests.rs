use crate::functions::address::AddressSubstitutor;
use crate::functions::base64_encode::Base64EncodeSubstitutor;
use crate::functions::date::GetDateSubstitutor;
use crate::functions::datetime::GetDateTimeSubstitutor;
use crate::functions::email::EmailSubstitutor;
use crate::functions::first_name::FirstNameSubstitutor;
use crate::functions::guid::GuidSubstitutor;
use crate::functions::last_name::LastNameSubstitutor;
use crate::functions::lower::LowerSubstitutor;
use crate::functions::name::NameSubstitutor;
use crate::functions::number::NumberSubstitutor;
use crate::functions::string_gen::StringSubstitutor;
use crate::functions::substitution::FunctionSubstitutor;
use crate::functions::time::GetTimeSubstitutor;
use crate::functions::upper::UpperSubstitutor;
use crate::functions::utc_datetime::GetUtcDateTimeSubstitutor;
use regex::Regex;

#[test]
fn test_substitute_functions_case_insensitivity() {
    use crate::functions::substitute_functions;

    let input = r#"{"guid": "guid()", "GUID": "GUID()", "Guid": "Guid()"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("GUID()"));
    assert!(!result.contains("Guid()"));

    let guid_pattern = Regex::new(r"[0-9a-fA-F]{32}").unwrap();
    let matches: Vec<_> = guid_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3, "Should have 3 GUID values");
}

#[test]
fn test_substitute_functions_string_case_insensitivity() {
    use crate::functions::substitute_functions;

    let input = r#"{"s1": "string()", "s2": "STRING()", "s3": "String()"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("string()"));
    assert!(!result.contains("STRING()"));
    assert!(!result.contains("String()"));

    let alphanum_pattern = Regex::new(r#""s\d": "([A-Za-z0-9]{20})""#).unwrap();
    let matches: Vec<_> = alphanum_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3, "Should have 3 string values");
}

#[test]
fn test_substitute_functions_number_case_insensitivity() {
    use crate::functions::substitute_functions;

    let input = r#"{"n1": number(), "n2": NUMBER(), "n3": Number()}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("number()"));
    assert!(!result.contains("NUMBER()"));
    assert!(!result.contains("Number()"));

    let number_pattern = Regex::new(r#""n\d": (\d+)"#).unwrap();
    let matches: Vec<_> = number_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3, "Should have 3 number values");
}

#[test]
fn test_substitute_functions_base64_case_insensitivity() {
    use crate::functions::substitute_functions;

    let input = r#"{"b1": "base64_encode('test')", "b2": "BASE64_ENCODE('test')", "b3": "Base64_Encode('test')"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("base64_encode"));
    assert!(!result.contains("BASE64_ENCODE"));
    assert!(!result.contains("Base64_Encode"));

    assert!(
        result.contains("dGVzdA=="),
        "Should contain base64 encoded 'test'"
    );

    let b64_pattern = Regex::new(r#""b\d": "dGVzdA==""#).unwrap();
    let matches: Vec<_> = b64_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3, "Should have 3 base64 values");
}

#[test]
fn test_substitute_functions_multiple_in_same_string() {
    use crate::functions::substitute_functions;

    let input = r#"{"id": "guid()", "name": "User string()", "score": number()}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("string()"));
    assert!(!result.contains("number()"));

    assert!(result.contains(r#""id":"#));
    assert!(result.contains(r#""name":"#));
    assert!(result.contains(r#""score":"#));
}

#[test]
fn test_substitute_functions_in_url() {
    use crate::functions::substitute_functions;

    let input = "https://api.example.com/users/guid()/posts/number()";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("number()"));

    assert!(result.starts_with("https://api.example.com/users/"));
    assert!(result.contains("/posts/"));
}

#[test]
fn test_substitute_functions_in_headers() {
    use crate::functions::substitute_functions;

    let input = "Authorization: Bearer base64_encode('user:pass')\nX-Request-ID: guid()";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("base64_encode"));
    assert!(!result.contains("guid()"));

    assert!(result.contains("Authorization: Bearer"));
    assert!(result.contains("X-Request-ID:"));
}

#[test]
fn test_name_substitutor_case_insensitive() {
    use crate::functions::substitute_functions;

    let input = r#"{"n1": "name()", "n2": "NAME()", "n3": "Name()"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("name()"));
    assert!(!result.contains("NAME()"));
    assert!(!result.contains("Name()"));

    let space_count = result.matches(' ').count();
    assert!(
        space_count >= 3,
        "Should have at least 3 spaces (one per generated name)"
    );
}

#[test]
fn test_first_name_substitutor_case_insensitive() {
    use crate::functions::substitute_functions;

    let input = r#"{"f1": "first_name()", "f2": "FIRST_NAME()", "f3": "First_Name()"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("first_name()"));
    assert!(!result.contains("FIRST_NAME()"));
    assert!(!result.contains("First_Name()"));
}

#[test]
fn test_last_name_substitutor_case_insensitive() {
    use crate::functions::substitute_functions;

    let input = r#"{"l1": "last_name()", "l2": "LAST_NAME()", "l3": "Last_Name()"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("last_name()"));
    assert!(!result.contains("LAST_NAME()"));
    assert!(!result.contains("Last_Name()"));
}

#[test]
fn test_all_functions_combined() {
    use crate::functions::substitute_functions;

    let input = r#"{"id": "guid()", "firstName": "first_name()", "lastName": "last_name()", "fullName": "name()", "address": "address()", "randomStr": "string()", "randomNum": "number()", "encoded": "base64_encode('secret')"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("first_name()"));
    assert!(!result.contains("last_name()"));
    assert!(!result.contains("name()"));
    assert!(!result.contains("address()"));
    assert!(!result.contains("string()"));
    assert!(!result.contains("number()"));
    assert!(!result.contains("base64_encode"));

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

#[test]
fn test_guid_with_whitespace_around() {
    use crate::functions::substitute_functions;

    let input = "  guid()  ";
    let result = substitute_functions(input).unwrap();

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
fn test_function_not_replaced_when_incorrect_syntax() {
    use crate::functions::substitute_functions;

    let input1 = "guid(";
    let result1 = substitute_functions(input1).unwrap();
    assert_eq!(result1, input1, "Should not replace malformed function");

    let input2 = "guid)";
    let result2 = substitute_functions(input2).unwrap();
    assert_eq!(result2, input2, "Should not replace malformed function");

    let input3 = "guid() extra";
    let result3 = substitute_functions(input3).unwrap();
    assert!(
        result3.contains("extra"),
        "Should preserve non-function text"
    );
    assert!(!result3.contains("guid()"));
}

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

#[test]
fn test_multiple_same_function_generates_different_values() {
    use crate::functions::substitute_functions;

    let input = r#"{"id1": "guid()", "id2": "guid()", "id3": "guid()"}"#;
    let result = substitute_functions(input).unwrap();

    let guid_pattern = Regex::new(r"[0-9a-fA-F]{32}").unwrap();
    let matches: Vec<_> = guid_pattern.find_iter(&result).collect();

    assert_eq!(matches.len(), 3, "Should have 3 GUIDs");

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

    assert_eq!(result1, result2);
}

#[test]
fn test_functions_with_newlines() {
    use crate::functions::substitute_functions;

    let input = "{\n  \"id\": \"guid()\",\n  \"name\": \"string()\"\n}";
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("guid()"));
    assert!(!result.contains("string()"));
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

#[test]
fn test_all_case_variations_of_guid() {
    use crate::functions::substitute_functions;

    let input = r#"{"g1": "guid()", "g2": "GUID()", "g3": "Guid()", "g4": "gUiD()"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.to_lowercase().contains("guid()"));
}

#[test]
fn test_all_case_variations_of_string() {
    use crate::functions::substitute_functions;

    let input = r#"{"s1": "string()", "s2": "STRING()", "s3": "String()", "s4": "sTrInG()"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.to_lowercase().contains("string()"));
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

    assert!(!result.contains("address()"));
    assert!(!result.contains("ADDRESS()"));
    assert!(!result.contains("Address()"));
}

#[test]
fn test_base64_encode_with_nested_quotes() {
    use crate::functions::substitute_functions;

    let input = r#"{"token": "base64_encode('test')"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("base64_encode"));
    assert!(result.contains("dGVzdA=="));
}

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

#[test]
fn test_repeated_substitution_convergence() {
    use crate::functions::substitute_functions;

    let input = r#"{"id": "guid()", "value": 123}"#;
    let mut current = input.to_string();

    for i in 0..5 {
        let next = substitute_functions(&current).unwrap();
        if i == 0 {
            assert_ne!(current, next, "First substitution should modify the input");
        } else {
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
fn test_generator_stability_across_calls() {
    let guid_sub = GuidSubstitutor {};
    let string_sub = StringSubstitutor {};
    let number_sub = NumberSubstitutor {};

    let guid = guid_sub.generate();
    assert_eq!(guid.len(), 32);

    let s = string_sub.generate();
    assert_eq!(s.len(), 20);

    let n = number_sub.generate();
    assert!(n.parse::<i32>().is_ok());
}

#[test]
fn test_no_partial_function_substitutions() {
    use crate::functions::substitute_functions;

    let input = "guid() and string() and number()";
    let result = substitute_functions(input).unwrap();

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

#[test]
fn test_substitution_preserves_order() {
    use crate::functions::substitute_functions;

    let input = r#"["guid()", "string()", "number()"]"#;
    let result = substitute_functions(input).unwrap();

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

    assert!(!result.contains("guid()"));
    assert!(!result.contains("first_name()"));
    assert!(!result.contains("last_name()"));
    assert!(!result.contains("name()"));
    assert!(!result.contains("address()"));
    assert!(!result.contains("number()"));
    assert!(!result.contains("base64_encode"));
    assert!(!result.contains("string()"));

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

#[test]
fn test_address_all_case_variations() {
    use crate::functions::substitute_functions;

    let input = r#"{"a1": "address()", "a2": "ADDRESS()", "a3": "Address()", "a4": "aDdReSs()"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.to_lowercase().contains("address()"));
}

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
fn test_function_with_internal_whitespace() {
    use crate::functions::substitute_functions;

    let input = "guid(  )";
    let result = substitute_functions(input).unwrap();

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

#[test]
fn test_multiple_substitution_calls_same_input_different_outputs() {
    use crate::functions::substitute_functions;

    let input = r#"{"id": "guid()"}"#;
    let result1 = substitute_functions(input).unwrap();
    let result2 = substitute_functions(input).unwrap();

    assert_ne!(
        result1, result2,
        "Multiple substitutions should produce different random values"
    );
}

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

        assert!(!result.contains("guid()"));
        assert!(!result.contains("name()"));
        assert!(!result.contains("string()"));
        assert!(!result.contains("number()"));
        assert!(!result.contains("first_name()"));
        assert!(!result.contains("last_name()"));
        assert!(!result.contains("address()"));
    }
}

#[test]
fn test_substitution_order_independence() {
    use crate::functions::substitute_functions;

    let input = "guid() name() string() number() address()";
    let result = substitute_functions(input).unwrap();

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

    let space_count = result.matches(' ').count();
    assert!(
        space_count >= 4,
        "Should have at least 4 spaces separating items"
    );
}

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

#[test]
fn test_address_in_complex_json() {
    use crate::functions::substitute_functions;

    let input = r#"{"locations": [{"address": "address()"}, {"address": "address()"}, {"address": "address()"}]}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("address()"));
    assert!(result.contains("locations"));
    assert_eq!(result.matches("[").count(), 1);
}

#[test]
fn test_substitution_with_many_empty_functions_attempt() {
    use crate::functions::substitute_functions;

    let input = "guid() guid() guid() guid() guid() string() string() string() string() string()";
    let result = substitute_functions(input).unwrap();

    let guid_pattern = Regex::new(r"[0-9a-fA-F]{32}").unwrap();
    let guid_matches: Vec<_> = guid_pattern.find_iter(&result).collect();
    assert_eq!(guid_matches.len(), 5);

    let alphanum_pattern = Regex::new(r"[A-Za-z0-9]{20}").unwrap();
    let string_matches: Vec<_> = alphanum_pattern.find_iter(&result).collect();
    assert!(string_matches.len() >= 5);
}

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

#[test]
fn test_substitute_functions_date_case_insensitivity() {
    use crate::functions::substitute_functions;

    let input = r#"{"date1": "getdate()", "date2": "GETDATE()", "date3": "GetDate()"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("getdate()"));
    assert!(!result.contains("GETDATE()"));
    assert!(!result.contains("GetDate()"));

    let date_pattern = Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap();
    let matches: Vec<_> = date_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3, "Should have 3 date values");
}

#[test]
fn test_substitute_functions_time_case_insensitivity() {
    use crate::functions::substitute_functions;

    let input = r#"{"time1": "gettime()", "time2": "GETTIME()", "time3": "GetTime()"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("gettime()"));
    assert!(!result.contains("GETTIME()"));
    assert!(!result.contains("GetTime()"));

    let time_pattern = Regex::new(r"\d{2}:\d{2}:\d{2}").unwrap();
    let matches: Vec<_> = time_pattern.find_iter(&result).collect();
    assert_eq!(matches.len(), 3, "Should have 3 time values");
}

#[test]
fn test_substitute_functions_datetime_case_insensitivity() {
    use crate::functions::substitute_functions;

    let input = r#"{"dt1": "getdatetime()", "dt2": "GETDATETIME()", "dt3": "GetDateTime()"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("getdatetime()"));
    assert!(!result.contains("GETDATETIME()"));
    assert!(!result.contains("GetDateTime()"));

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

    assert!(!result.contains("getutcdatetime()"));
    assert!(!result.contains("GETUTCDATETIME()"));
    assert!(!result.contains("GetUtcDateTime()"));

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

    let date_pattern = Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap();
    assert!(date_pattern.is_match(&result));

    let time_pattern = Regex::new(r"\d{2}:\d{2}:\d{2}").unwrap();
    assert!(time_pattern.is_match(&result));
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

    assert_eq!(result1, result2);
    assert!(result1.contains("TEST"));
    assert!(result1.contains("test"));
}

#[test]
fn test_upper_lower_with_variables_notation() {
    use crate::functions::substitute_functions;

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

#[test]
fn test_email_substitutor_case_insensitive() {
    use crate::functions::substitute_functions;

    let input = r#"{"e1": "email()", "e2": "EMAIL()", "e3": "Email()"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("email()"));
    assert!(!result.contains("EMAIL()"));
    assert!(!result.contains("Email()"));

    assert!(result.contains("@"), "Result should contain @ symbol");
    assert!(result.contains("."), "Result should contain . symbol");
}

#[test]
fn test_email_in_json_context() {
    use crate::functions::substitute_functions;

    let input = r#"{"email": "email()", "user_email": "email()", "contact": "email()"}"#;
    let result = substitute_functions(input).unwrap();

    assert!(!result.contains("email()"));

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

    assert!(!result.contains("email()"));

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

    assert!(!result.contains("guid()"));
    assert!(!result.contains("name()"));
    assert!(!result.contains("email()"));
    assert!(!result.contains("number()"));

    assert!(result.contains("@"));
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

    assert!(!result.contains("email()"));

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
            assert_ne!(current, next, "First substitution should modify the input");
        } else {
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

    assert!(!result.contains("()"));
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
