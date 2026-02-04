use super::substitution::{
    substitute_functions_in_request, substitute_request_variables_in_request,
};
use crate::conditions;
use crate::parser;
use crate::runner;
use crate::types::{HttpRequest, HttpResult, RequestContext};
use anyhow::Result;

/// Result of processing a single request
pub enum RequestProcessingResult {
    /// Request was skipped due to conditions or dependencies
    Skipped {
        request: HttpRequest,
        reason: String,
    },
    /// Request was executed successfully or with errors
    Executed {
        request: HttpRequest,
        result: HttpResult,
    },
    /// Request processing failed with an error
    Failed {
        request: HttpRequest,
        error: String,
    },
}

/// Process HTTP requests from a file with incremental callbacks for UI updates
///
/// This function processes requests one at a time, maintaining proper context
/// for variable substitution, function evaluation, and condition checking.
/// It calls the provided callback after each request is processed.
pub fn process_http_file_incremental<F>(
    file_path: &str,
    environment: Option<&str>,
    insecure: bool,
    mut callback: F,
) -> Result<()>
where
    F: FnMut(usize, usize, RequestProcessingResult),
{
    // Parse the file
    let requests = parser::parse_http_file(file_path, environment)?;
    let total = requests.len();

    if requests.is_empty() {
        return Ok(());
    }

    let mut request_contexts: Vec<RequestContext> = Vec::new();

    for (idx, mut request) in requests.into_iter().enumerate() {
        let request_count = (idx + 1) as u32;
        
        // Check dependencies
        if let Some(dep_name) = request.depends_on.as_ref() {
            if !conditions::check_dependency(&Some(dep_name.clone()), &request_contexts) {
                let reason = format!("Dependency on '{}' not met", dep_name);
                callback(
                    idx,
                    total,
                    RequestProcessingResult::Skipped {
                        request: request.clone(),
                        reason,
                    },
                );
                add_request_context(&mut request_contexts, request, None, request_count);
                continue;
            }
        }

        // Check conditions
        if !request.conditions.is_empty() {
            match conditions::evaluate_conditions(&request.conditions, &request_contexts) {
                Ok(true) => {
                    // Conditions met, continue
                }
                Ok(false) => {
                    // Conditions not met, skip
                    callback(
                        idx,
                        total,
                        RequestProcessingResult::Skipped {
                            request: request.clone(),
                            reason: "Conditions not met".to_string(),
                        },
                    );
                    add_request_context(&mut request_contexts, request, None, request_count);
                    continue;
                }
                Err(e) => {
                    callback(
                        idx,
                        total,
                        RequestProcessingResult::Failed {
                            request: request.clone(),
                            error: format!("Condition evaluation error: {}", e),
                        },
                    );
                    add_request_context(&mut request_contexts, request, None, request_count);
                    continue;
                }
            }
        }

        // Apply variable substitutions
        if let Err(e) = substitute_request_variables_in_request(&mut request, &request_contexts) {
            callback(
                idx,
                total,
                RequestProcessingResult::Failed {
                    request: request.clone(),
                    error: format!("Variable substitution error: {}", e),
                },
            );
            add_request_context(&mut request_contexts, request, None, request_count);
            continue;
        }

        // Apply function substitutions
        if let Err(e) = substitute_functions_in_request(&mut request) {
            callback(
                idx,
                total,
                RequestProcessingResult::Failed {
                    request: request.clone(),
                    error: format!("Function substitution error: {}", e),
                },
            );
            add_request_context(&mut request_contexts, request, None, request_count);
            continue;
        }

        // Execute the request
        match runner::execute_http_request(&request, false, insecure) {
            Ok(result) => {
                add_request_context(
                    &mut request_contexts,
                    request.clone(),
                    Some(result.clone()),
                    request_count,
                );
                callback(
                    idx,
                    total,
                    RequestProcessingResult::Executed {
                        request,
                        result,
                    },
                );
            }
            Err(e) => {
                callback(
                    idx,
                    total,
                    RequestProcessingResult::Failed {
                        request: request.clone(),
                        error: e.to_string(),
                    },
                );
                add_request_context(&mut request_contexts, request, None, request_count);
            }
        }
    }

    Ok(())
}

fn add_request_context(
    contexts: &mut Vec<RequestContext>,
    request: HttpRequest,
    result: Option<HttpResult>,
    request_count: u32,
) {
    let context_name = if let Some(ref name) = request.name {
        name.clone()
    } else {
        format!("Request {}", request_count)
    };

    contexts.push(RequestContext {
        name: context_name,
        request,
        result,
    });
}
