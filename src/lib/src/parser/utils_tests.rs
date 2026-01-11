use super::utils::*;

#[test]
fn test_is_http_request_line_with_get() {
    assert!(is_http_request_line("GET https://example.com"));
    assert!(is_http_request_line("GET /api/users"));
}

#[test]
fn test_is_http_request_line_with_post() {
    assert!(is_http_request_line("POST https://api.example.com/users"));
    assert!(is_http_request_line("POST /users"));
}

#[test]
fn test_is_http_request_line_with_put() {
    assert!(is_http_request_line("PUT https://api.example.com/users/1"));
    assert!(is_http_request_line("PUT /users/1"));
}

#[test]
fn test_is_http_request_line_with_delete() {
    assert!(is_http_request_line(
        "DELETE https://api.example.com/users/1"
    ));
    assert!(is_http_request_line("DELETE /users/1"));
}

#[test]
fn test_is_http_request_line_with_patch() {
    assert!(is_http_request_line(
        "PATCH https://api.example.com/users/1"
    ));
    assert!(is_http_request_line("PATCH /users/1"));
}

#[test]
fn test_is_http_request_line_with_head() {
    assert!(is_http_request_line("HEAD https://api.example.com/health"));
    assert!(is_http_request_line("HEAD /health"));
}

#[test]
fn test_is_http_request_line_with_options() {
    assert!(is_http_request_line("OPTIONS https://api.example.com"));
    assert!(is_http_request_line("OPTIONS /"));
}

#[test]
fn test_is_http_request_line_with_trace() {
    assert!(is_http_request_line("TRACE https://api.example.com"));
}

#[test]
fn test_is_http_request_line_with_connect() {
    assert!(is_http_request_line("CONNECT proxy.example.com:443"));
}

#[test]
fn test_is_http_request_line_with_http_protocol() {
    assert!(is_http_request_line("GET /api HTTP/1.1"));
    assert!(is_http_request_line("POST /api HTTP/2.0"));
}

#[test]
fn test_is_http_request_line_rejects_non_http_lines() {
    assert!(!is_http_request_line("Content-Type: application/json"));
    assert!(!is_http_request_line("Authorization: Bearer token"));
    assert!(!is_http_request_line("# This is a comment"));
    assert!(!is_http_request_line("@variable=value"));
    assert!(!is_http_request_line(""));
    assert!(!is_http_request_line("   "));
}

#[test]
fn test_is_http_request_line_case_sensitive_method() {
    // Methods should be uppercase to match
    assert!(!is_http_request_line("get https://example.com"));
    assert!(!is_http_request_line("post /api/users"));
    assert!(!is_http_request_line("Put /users/1"));
}

#[test]
fn test_is_http_request_line_with_whitespace() {
    // The function checks for exact method prefixes, so leading whitespace won't match
    assert!(!is_http_request_line("  GET https://example.com"));
    // But trailing/internal whitespace should work
    assert!(is_http_request_line("GET  https://example.com"));
}

#[test]
fn test_is_http_request_line_partial_method_match() {
    // Should not match partial method names
    assert!(!is_http_request_line("GETTING /api"));
    assert!(!is_http_request_line("POSTS /api"));
}
