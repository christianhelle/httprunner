use crate::types::Header;

/// Encodes a body as application/x-www-form-urlencoded content.
///
/// Spaces are encoded as `+`, and special characters are percent-encoded
/// according to RFC 1866 (application/x-www-form-urlencoded).
pub fn encode_form_body(body: &str) -> String {
    form_urlencoded::byte_serialize(body.as_bytes()).collect()
}

/// Checks if the request headers indicate `application/x-www-form-urlencoded`
/// content type.
///
/// Returns `true` if any header named "Content-Type" (case-insensitive) has a
/// value starting with `application/x-www-form-urlencoded`.
pub fn needs_form_encoding(headers: &[Header]) -> bool {
    headers.iter().any(|h| {
        if h.name.to_lowercase() != "content-type" {
            return false;
        }
        h.value
            .to_lowercase()
            .starts_with("application/x-www-form-urlencoded")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_form_body_encodes_spaces_as_plus() {
        let input = "scope=scopea scopeb";
        let result = encode_form_body(input);
        assert_eq!(result, "scope%3Dscopea+scopeb");
    }

    #[test]
    fn test_encode_form_body_preserves_already_encoded_chars() {
        let input = "key=value%20encoded";
        let result = encode_form_body(input);
        assert_eq!(result, "key%3Dvalue%2520encoded");
    }

    #[test]
    fn test_encode_form_body_handles_ampersands() {
        let input = "a=1&b=2";
        let result = encode_form_body(input);
        assert_eq!(result, "a%3D1%26b%3D2");
    }

    #[test]
    fn test_encode_form_body_handles_equals_signs() {
        let input = "key=value=pair";
        let result = encode_form_body(input);
        assert_eq!(result, "key%3Dvalue%3Dpair");
    }

    #[test]
    fn test_encode_form_body_handles_special_characters() {
        let input = "key=hello world!@#$%";
        let result = encode_form_body(input);
        assert!(!result.contains(' '));
        assert!(result.contains('%'));
    }

    #[test]
    fn test_encode_form_body_empty_string() {
        let result = encode_form_body("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_encode_form_body_no_special_chars_unchanged() {
        let input = "abc=123&xyz=456";
        let result = encode_form_body(input);
        assert_eq!(result, "abc%3D123%26xyz%3D456");
    }

    #[test]
    fn test_encode_form_body_with_unicode() {
        let input = "name=test%20value";
        let result = encode_form_body(input);
        assert!(!result.contains(' '));
    }

    #[test]
    fn test_needs_form_encoding_with_form_content_type() {
        let headers = vec![Header {
            name: "Content-Type".to_string(),
            value: "application/x-www-form-urlencoded".to_string(),
        }];
        assert!(needs_form_encoding(&headers));
    }

    #[test]
    fn test_needs_form_encoding_with_charset() {
        let headers = vec![Header {
            name: "Content-Type".to_string(),
            value: "application/x-www-form-urlencoded; charset=utf-8".to_string(),
        }];
        assert!(needs_form_encoding(&headers));
    }

    #[test]
    fn test_needs_form_encoding_case_insensitive() {
        let headers = vec![Header {
            name: "content-type".to_string(),
            value: "APPLICATION/X-WWW-FORM-URLENCODED".to_string(),
        }];
        assert!(needs_form_encoding(&headers));
    }

    #[test]
    fn test_needs_form_encoding_with_json_content_type() {
        let headers = vec![Header {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }];
        assert!(!needs_form_encoding(&headers));
    }

    #[test]
    fn test_needs_form_encoding_with_no_content_type() {
        let headers: Vec<Header> = vec![];
        assert!(!needs_form_encoding(&headers));
    }

    #[test]
    fn test_needs_form_encoding_with_other_header() {
        let headers = vec![Header {
            name: "Authorization".to_string(),
            value: "Bearer token".to_string(),
        }];
        assert!(!needs_form_encoding(&headers));
    }

    #[test]
    fn test_needs_form_encoding_with_multiple_headers() {
        let headers = vec![
            Header {
                name: "Authorization".to_string(),
                value: "Bearer token".to_string(),
            },
            Header {
                name: "Content-Type".to_string(),
                value: "application/x-www-form-urlencoded".to_string(),
            },
        ];
        assert!(needs_form_encoding(&headers));
    }
}
