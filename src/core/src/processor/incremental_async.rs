use super::processing_result::RequestProcessingResult;
use super::substitution::{
    substitute_functions_in_request, substitute_request_variables_in_request,
};
use crate::conditions;
use crate::parser;
use crate::runner;
use crate::types::{HttpRequest, RequestContext, Variable};
use anyhow::Result;
use wasm_bindgen::JsCast;

async fn sleep_ms(delay_ms: u64) {
    if delay_ms == 0 {
        return;
    }

    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        if let Some(window) = web_sys::window() {
            let callback = wasm_bindgen::closure::Closure::once_into_js(move || {
                let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
            });

            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                delay_ms.min(i32::MAX as u64) as i32,
            );
        } else {
            let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
        }
    });

    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

/// Process HTTP requests from raw content with incremental callbacks for UI updates.
///
/// This mirrors the native incremental processor so web execution preserves request
/// variables, conditions, dependencies, and delays.
pub async fn process_http_content_incremental_async<F>(
    content: &str,
    env_variables: Vec<Variable>,
    delay_ms: u64,
    mut callback: F,
) -> Result<()>
where
    F: FnMut(usize, usize, RequestProcessingResult) -> bool,
{
    let requests = parser::parse_http_content_with_variables(content, env_variables)?;
    let total = requests.len();

    if requests.is_empty() {
        return Ok(());
    }

    let mut request_contexts: Vec<RequestContext> = Vec::new();

    for (idx, mut request) in requests.into_iter().enumerate() {
        let request_count = (idx + 1) as u32;

        if idx > 0 && delay_ms > 0 {
            sleep_ms(delay_ms).await;
        }

        if let Some(dep_name) = request.depends_on.as_ref()
            && !conditions::check_dependency(&Some(dep_name.clone()), &request_contexts)
        {
            let reason = format!("Dependency on '{}' not met", dep_name);
            let should_continue = callback(
                idx,
                total,
                RequestProcessingResult::Skipped {
                    request: request.clone(),
                    reason,
                },
            );
            add_request_context(&mut request_contexts, request, None, request_count);
            if !should_continue {
                break;
            }
            continue;
        }

        if !request.conditions.is_empty() {
            match conditions::evaluate_conditions(&request.conditions, &request_contexts) {
                Ok(true) => {}
                Ok(false) => {
                    let should_continue = callback(
                        idx,
                        total,
                        RequestProcessingResult::Skipped {
                            request: request.clone(),
                            reason: "Conditions not met".to_string(),
                        },
                    );
                    add_request_context(&mut request_contexts, request, None, request_count);
                    if !should_continue {
                        break;
                    }
                    continue;
                }
                Err(e) => {
                    let should_continue = callback(
                        idx,
                        total,
                        RequestProcessingResult::Failed {
                            request: request.clone(),
                            error: format!("Condition evaluation error: {}", e),
                        },
                    );
                    add_request_context(&mut request_contexts, request, None, request_count);
                    if !should_continue {
                        break;
                    }
                    continue;
                }
            }
        }

        if let Err(e) = substitute_request_variables_in_request(&mut request, &request_contexts) {
            let should_continue = callback(
                idx,
                total,
                RequestProcessingResult::Failed {
                    request: request.clone(),
                    error: format!("Variable substitution error: {}", e),
                },
            );
            add_request_context(&mut request_contexts, request, None, request_count);
            if !should_continue {
                break;
            }
            continue;
        }

        if let Err(e) = substitute_functions_in_request(&mut request) {
            let should_continue = callback(
                idx,
                total,
                RequestProcessingResult::Failed {
                    request: request.clone(),
                    error: format!("Function substitution error: {}", e),
                },
            );
            add_request_context(&mut request_contexts, request, None, request_count);
            if !should_continue {
                break;
            }
            continue;
        }

        if let Some(pre_delay) = request.pre_delay_ms
            && pre_delay > 0
        {
            sleep_ms(pre_delay).await;
        }

        let post_delay = request.post_delay_ms;

        match runner::execute_http_request_async(&request, false, false).await {
            Ok(result) => {
                add_request_context(
                    &mut request_contexts,
                    request.clone(),
                    Some(result.clone()),
                    request_count,
                );
                let should_continue = callback(
                    idx,
                    total,
                    RequestProcessingResult::Executed { request, result },
                );
                if !should_continue {
                    break;
                }
            }
            Err(e) => {
                let should_continue = callback(
                    idx,
                    total,
                    RequestProcessingResult::Failed {
                        request: request.clone(),
                        error: e.to_string(),
                    },
                );
                add_request_context(&mut request_contexts, request, None, request_count);
                if !should_continue {
                    break;
                }
            }
        }

        if let Some(post_delay) = post_delay
            && post_delay > 0
        {
            sleep_ms(post_delay).await;
        }
    }

    Ok(())
}

fn add_request_context(
    contexts: &mut Vec<RequestContext>,
    request: HttpRequest,
    result: Option<crate::HttpResult>,
    request_count: u32,
) {
    let context_name = if let Some(ref name) = request.name {
        name.clone()
    } else {
        format!("request_{}", request_count)
    };

    contexts.push(RequestContext {
        name: context_name,
        request,
        result,
    });
}
