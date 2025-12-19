use crate::request_variables;
use crate::types::{HttpRequest, RequestContext};
use anyhow::Result;

pub fn substitute_request_variables_in_request(
    request: &mut HttpRequest,
    context: &[RequestContext],
) -> Result<()> {
    request.url = request_variables::substitute_request_variables(&request.url, context)?;

    for header in &mut request.headers {
        header.name = request_variables::substitute_request_variables(&header.name, context)?;
        header.value = request_variables::substitute_request_variables(&header.value, context)?;
    }

    if let Some(ref body) = request.body {
        request.body = Some(request_variables::substitute_request_variables(
            body, context,
        )?);
    }

    for assertion in &mut request.assertions {
        assertion.expected_value =
            request_variables::substitute_request_variables(&assertion.expected_value, context)?;
    }

    Ok(())
}
