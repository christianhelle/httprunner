use crate::error::Result;
use crate::functions::substitute_functions;
use crate::types::{HttpRequest, RequestContext};
use crate::variables;

pub fn substitute_request_variables_in_request(
    request: &mut HttpRequest,
    context: &[RequestContext],
) -> Result<()> {
    request.url = variables::substitute_request_variables(&request.url, context)?;

    for header in &mut request.headers {
        header.name = variables::substitute_request_variables(&header.name, context)?;
        header.value = variables::substitute_request_variables(&header.value, context)?;
    }

    if let Some(ref body) = request.body {
        request.body = Some(variables::substitute_request_variables(body, context)?);
    }

    for assertion in &mut request.assertions {
        assertion.expected_value =
            variables::substitute_request_variables(&assertion.expected_value, context)?;
    }

    Ok(())
}

pub fn substitute_functions_in_request(request: &mut HttpRequest) -> Result<()> {
    request.url = substitute_functions(&request.url)?;

    for header in &mut request.headers {
        header.name = substitute_functions(&header.name)?;
        header.value = substitute_functions(&header.value)?;
    }

    if let Some(ref body) = request.body {
        request.body = Some(substitute_functions(body)?);
    }

    for assertion in &mut request.assertions {
        assertion.expected_value = substitute_functions(&assertion.expected_value)?;
    }

    Ok(())
}
