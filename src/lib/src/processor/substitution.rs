use crate::functions::substitute_functions;
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

    if let Some(ref body) = request.body {
        request.body = Some(substitutor(body)?);
    }

    for assertion in &mut request.assertions {
        assertion.expected_value = substitutor(&assertion.expected_value)?;
    }

    Ok(())
}

pub fn substitute_request_variables_in_request(
    request: &mut HttpRequest,
    context: &[RequestContext],
) -> Result<()> {
    apply_substitution(request, |s| {
        variables::substitute_request_variables(s, context)
    })
}

pub fn substitute_functions_in_request(request: &mut HttpRequest) -> Result<()> {
    apply_substitution(request, substitute_functions)
}
