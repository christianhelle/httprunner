use crate::functions;
use crate::types::{HttpRequest, RequestContext};
use crate::variables;
use anyhow::Result;

fn apply_substitution<F>(request: &mut HttpRequest, substitutor: F) -> Result<()>
where
    F: Fn(&str) -> Result<String>,
{
    request.url = substitutor(&request.url)?;

    for header in &mut request.headers {
        header.name = substitutor(&header.name)?;
        header.value = substitutor(&header.value)?;
    }

    if let Some(body) = request.body.as_ref() {
        request.body = Some(substitutor(body)?);
    }

    for assertion in &mut request.assertions {
        assertion.expected_value = substitutor(&assertion.expected_value)?;
    }

    Ok(())
}

pub(crate) fn substitute_request_variables_in_request(
    request: &mut HttpRequest,
    context: &[RequestContext],
) -> Result<()> {
    apply_substitution(request, |value| {
        variables::substitute_request_variables(value, context)
    })
}

pub(crate) fn substitute_functions_in_request(request: &mut HttpRequest) -> Result<()> {
    apply_substitution(request, functions::substitute_functions)
}
