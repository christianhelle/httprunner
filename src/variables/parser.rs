use crate::types::{RequestVariable, RequestVariableSource, RequestVariableTarget};
use crate::err;
use crate::error::Result;

pub fn parse_request_variable(reference: &str) -> Result<RequestVariable> {
    let mut cleaned = reference;
    if reference.starts_with("{{") && reference.ends_with("}}") {
        cleaned = &reference[2..reference.len() - 2];
    }

    let parts: Vec<&str> = cleaned.split('.').collect();
    if parts.len() < 4 {
        return Err(err!("Invalid request variable format"));
    }

    let request_name = parts[0];
    let source_str = parts[1];
    let target_str = parts[2];
    let path = parts[3..].join(".");

    let source = match source_str {
        "request" => RequestVariableSource::Request,
        "response" => RequestVariableSource::Response,
        _ => return Err(err!("Invalid source: {}", source_str)),
    };

    let target = match target_str {
        "body" => RequestVariableTarget::Body,
        "headers" => RequestVariableTarget::Headers,
        _ => return Err(err!("Invalid target: {}", target_str)),
    };

    Ok(RequestVariable {
        reference: reference.to_string(),
        request_name: request_name.to_string(),
        source,
        target,
        path,
    })
}
