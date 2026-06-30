use super::{encode_form_body, needs_form_encoding};
use crate::types::HttpRequest;
use anyhow::Result;

/// Parse the request's method string into a [`reqwest::Method`].
///
/// Shared by the blocking and async executors so method handling lives in one place.
pub(super) fn parse_method(request: &HttpRequest) -> Result<reqwest::Method> {
    let method = request.method.to_uppercase();
    Ok(reqwest::Method::from_bytes(method.as_bytes())?)
}

/// Resolve the request body, applying form-encoding when the headers call for it.
///
/// Shared by the blocking and async executors so body handling lives in one place.
pub(super) fn resolve_body(request: &HttpRequest) -> Option<String> {
    request.body.as_ref().map(|body| {
        if needs_form_encoding(&request.headers) {
            encode_form_body(body)
        } else {
            body.clone()
        }
    })
}

/// Client configuration derived from a request: connection/read timeouts and TLS policy.
///
/// Shared by the blocking and async executors so timeout/TLS handling lives in one place.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct ClientConfig {
    pub(super) connect_timeout_ms: u64,
    pub(super) timeout_ms: u64,
    pub(super) insecure: bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl ClientConfig {
    pub(super) fn from_request(request: &HttpRequest, insecure: bool) -> Self {
        Self {
            connect_timeout_ms: request.connection_timeout.unwrap_or(30_000),
            timeout_ms: request.timeout.unwrap_or(60_000),
            insecure,
        }
    }
}
