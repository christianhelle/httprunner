use crate::types::Header;

/// Encodes a body as application/x-www-form-urlencoded content.
///
/// The input must already use `&` exclusively as field separators. Any `&` characters
/// inside field values must be percent-encoded as `%26` before calling this function.
/// This function encodes only values (not keys or structural characters).
/// Malformed fields without `=` are left unchanged.
///
/// Spaces are encoded as `+`, and other special characters are percent-encoded
/// according to RFC 1866 (application/x-www-form-urlencoded).
///
/// Splits the body by `&` (form field separator), then for each field splits
/// by the first `=` (key-value delimiter). Only the value is encoded — the
/// key and structural characters (`&`, `=`) are preserved.
pub fn encode_form_body(body: &str) -> String {
    if body.is_empty() {
        return String::new();
    }

    let encoded: Vec<String> = body
        .split('&')
        .map(|field| {
            if let Some((key, value)) = field.split_once('=') {
                let encoded_value = {
                    let bytes = value.as_bytes();
                    let mut result = String::new();
                    let mut i = 0;
                    while i < bytes.len() {
                        if i + 2 < bytes.len()
                            && bytes[i] == b'%'
                            && bytes[i + 1].is_ascii_hexdigit()
                            && bytes[i + 2].is_ascii_hexdigit()
                        {
                            result.push_str(&value[i..i + 3]);
                            i += 3;
                        } else {
                            result.push_str(
                                &form_urlencoded::byte_serialize(&[bytes[i]]).collect::<String>(),
                            );
                            i += 1;
                        }
                    }
                    result
                };
                format!("{}={}", key, encoded_value)
            } else {
                field.to_string()
            }
        })
        .collect();

    encoded.join("&")
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
        assert_eq!(result, "scope=scopea+scopeb");
    }

    #[test]
    fn test_encode_form_body_preserves_ampersands() {
        let input = "a=1&b=2";
        let result = encode_form_body(input);
        assert_eq!(result, "a=1&b=2");
    }

    #[test]
    fn test_encode_form_body_encodes_spaces_in_values() {
        let input = "scope=scopea scopeb&grant_type=client_credentials";
        let result = encode_form_body(input);
        assert_eq!(result, "scope=scopea+scopeb&grant_type=client_credentials");
    }

    #[test]
    fn test_encode_form_body_handles_equals_in_values() {
        let input = "equation=a=b=c&normal=value";
        let result = encode_form_body(input);
        assert_eq!(result, "equation=a%3Db%3Dc&normal=value");
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
        assert_eq!(result, "abc=123&xyz=456");
    }

    #[test]
    fn test_encode_form_body_preserves_keys() {
        let input = "username=testuser&password=testpass";
        let result = encode_form_body(input);
        assert_eq!(result, "username=testuser&password=testpass");
    }

    #[test]
    fn test_encode_form_body_no_double_encoding() {
        let input = "state=abc%20def";
        let result = encode_form_body(input);
        assert_eq!(result, "state=abc%20def");

        let input2 = "key=a b";
        let result2 = encode_form_body(input2);
        assert_eq!(result2, "key=a+b");

        let input3 = "state=abc%20def ghi";
        let result3 = encode_form_body(input3);
        assert_eq!(result3, "state=abc%20def+ghi");
    }

    #[test]
    fn test_encode_form_body_field_without_equals() {
        let input = "flag&key=value";
        let result = encode_form_body(input);
        assert_eq!(result, "flag&key=value");
    }

    #[test]
    fn test_encode_form_body_oauth_scope() {
        let input = "grant_type=client_credentials&scope=openid profile email&client_id=myapp";
        let result = encode_form_body(input);
        assert_eq!(
            result,
            "grant_type=client_credentials&scope=openid+profile+email&client_id=myapp"
        );
    }

    #[test]
    fn test_encode_form_body_redirect_uri() {
        let input = "redirect_uri=http://localhost:8080/callback";
        let result = encode_form_body(input);
        assert_eq!(
            result,
            "redirect_uri=http%3A%2F%2Flocalhost%3A8080%2Fcallback"
        );
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
