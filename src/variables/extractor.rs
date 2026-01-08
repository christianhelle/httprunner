use super::json::extract_json_property;
use crate::error::Result;
use crate::types::{RequestContext, RequestVariable, RequestVariableSource, RequestVariableTarget};

pub fn extract_request_variable_value(
    request_var: &RequestVariable,
    context: &[RequestContext],
) -> Result<Option<String>> {
    let target_context = context
        .iter()
        .find(|ctx| ctx.name == request_var.request_name);

    if target_context.is_none() {
        return Ok(None);
    }

    let ctx = target_context.unwrap();

    match request_var.source {
        RequestVariableSource::Request => extract_from_request(request_var, &ctx.request),
        RequestVariableSource::Response => {
            if let Some(ref result) = ctx.result {
                extract_from_response(request_var, result)
            } else {
                Ok(None)
            }
        }
    }
}

fn extract_from_request(
    request_var: &RequestVariable,
    request: &crate::types::HttpRequest,
) -> Result<Option<String>> {
    match request_var.target {
        RequestVariableTarget::Body => {
            if let Some(ref body) = request.body {
                Ok(Some(body.clone()))
            } else {
                Ok(None)
            }
        }
        RequestVariableTarget::Headers => {
            for header in &request.headers {
                if header.name.eq_ignore_ascii_case(&request_var.path) {
                    return Ok(Some(header.value.clone()));
                }
            }
            Ok(None)
        }
    }
}

fn extract_from_response(
    request_var: &RequestVariable,
    result: &crate::types::HttpResult,
) -> Result<Option<String>> {
    match request_var.target {
        RequestVariableTarget::Body => {
            if let Some(ref body) = result.response_body {
                if request_var.path == "*" {
                    return Ok(Some(body.clone()));
                }

                if request_var.path.starts_with("$.") {
                    let property = &request_var.path[2..];
                    return extract_json_property(body, property);
                }

                Ok(Some(body.clone()))
            } else {
                Ok(None)
            }
        }
        RequestVariableTarget::Headers => {
            if let Some(ref headers) = result.response_headers {
                for (name, value) in headers {
                    if name.eq_ignore_ascii_case(&request_var.path) {
                        return Ok(Some(value.clone()));
                    }
                }
            }
            Ok(None)
        }
    }
}
