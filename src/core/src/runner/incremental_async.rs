use crate::conditions;
use crate::functions;
use crate::types::{HttpRequest, HttpResult, RequestContext};
use crate::variables;
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
#[cfg(not(target_arch = "wasm32"))]
use std::task::Waker;
use std::time::Duration;

pub type AsyncRequestFuture<'a> = Pin<Box<dyn Future<Output = Result<HttpResult>> + 'a>>;
pub type AsyncRequestExecutor =
    dyn for<'a> Fn(&'a HttpRequest, bool, bool) -> AsyncRequestFuture<'a>;

/// Result of processing a single request during async incremental execution.
#[derive(Debug)]
pub enum AsyncRequestProcessingResult {
    /// Request was skipped due to conditions or dependencies.
    Skipped {
        request: HttpRequest,
        reason: String,
    },
    /// Request was executed successfully or with errors.
    Executed {
        request: HttpRequest,
        result: HttpResult,
    },
    /// Request processing failed before execution.
    Failed { request: HttpRequest, error: String },
}

/// Process already-parsed HTTP requests incrementally while preserving the same
/// dependency, condition, and substitution semantics as the native processor.
pub async fn process_http_requests_incremental_async<F>(
    requests: Vec<HttpRequest>,
    insecure: bool,
    delay_ms: u64,
    mut callback: F,
    executor: &AsyncRequestExecutor,
) -> Result<()>
where
    F: FnMut(usize, usize, AsyncRequestProcessingResult) -> bool,
{
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
            let should_continue = callback(
                idx,
                total,
                AsyncRequestProcessingResult::Skipped {
                    request: request.clone(),
                    reason: format!("Dependency on '{}' not met", dep_name),
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
                        AsyncRequestProcessingResult::Skipped {
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
                Err(error) => {
                    let should_continue = callback(
                        idx,
                        total,
                        AsyncRequestProcessingResult::Failed {
                            request: request.clone(),
                            error: format!("Condition evaluation error: {}", error),
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

        if let Err(error) = substitute_request_variables_in_request(&mut request, &request_contexts)
        {
            let should_continue = callback(
                idx,
                total,
                AsyncRequestProcessingResult::Failed {
                    request: request.clone(),
                    error: format!("Variable substitution error: {}", error),
                },
            );
            add_request_context(&mut request_contexts, request, None, request_count);
            if !should_continue {
                break;
            }
            continue;
        }

        if let Err(error) = substitute_functions_in_request(&mut request) {
            let should_continue = callback(
                idx,
                total,
                AsyncRequestProcessingResult::Failed {
                    request: request.clone(),
                    error: format!("Function substitution error: {}", error),
                },
            );
            add_request_context(&mut request_contexts, request, None, request_count);
            if !should_continue {
                break;
            }
            continue;
        }

        if let Some(pre_delay_ms) = request.pre_delay_ms
            && pre_delay_ms > 0
        {
            sleep_ms(pre_delay_ms).await;
        }

        let post_delay_ms = request.post_delay_ms;

        match executor(&request, false, insecure).await {
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
                    AsyncRequestProcessingResult::Executed { request, result },
                );
                if !should_continue {
                    break;
                }
            }
            Err(error) => {
                let should_continue = callback(
                    idx,
                    total,
                    AsyncRequestProcessingResult::Failed {
                        request: request.clone(),
                        error: error.to_string(),
                    },
                );
                add_request_context(&mut request_contexts, request, None, request_count);
                if !should_continue {
                    break;
                }
            }
        }

        if let Some(post_delay_ms) = post_delay_ms
            && post_delay_ms > 0
        {
            sleep_ms(post_delay_ms).await;
        }
    }

    Ok(())
}

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

fn substitute_request_variables_in_request(
    request: &mut HttpRequest,
    context: &[RequestContext],
) -> Result<()> {
    apply_substitution(request, |value| {
        variables::substitute_request_variables(value, context)
    })
}

fn substitute_functions_in_request(request: &mut HttpRequest) -> Result<()> {
    apply_substitution(request, functions::substitute_functions)
}

fn add_request_context(
    contexts: &mut Vec<RequestContext>,
    request: HttpRequest,
    result: Option<HttpResult>,
    request_count: u32,
) {
    let context_name = request
        .name
        .clone()
        .unwrap_or_else(|| format!("request_{}", request_count));

    contexts.push(RequestContext {
        name: context_name,
        request,
        result,
    });
}

#[cfg(target_arch = "wasm32")]
async fn sleep_ms(delay_ms: u64) {
    if delay_ms == 0 {
        return;
    }

    gloo_timers::future::sleep(Duration::from_millis(delay_ms)).await;
}

#[cfg(not(target_arch = "wasm32"))]
async fn sleep_ms(delay_ms: u64) {
    if delay_ms == 0 {
        return;
    }

    NativeSleep::new(Duration::from_millis(delay_ms)).await;
}

#[cfg(not(target_arch = "wasm32"))]
struct NativeSleep {
    duration: Option<Duration>,
    state: Arc<NativeSleepState>,
}

#[cfg(not(target_arch = "wasm32"))]
struct NativeSleepState {
    completed: AtomicBool,
    waker: Mutex<Option<Waker>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl NativeSleep {
    fn new(duration: Duration) -> Self {
        Self {
            duration: Some(duration),
            state: Arc::new(NativeSleepState {
                completed: AtomicBool::new(false),
                waker: Mutex::new(None),
            }),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Future for NativeSleep {
    type Output = ();

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();

        if this.state.completed.load(Ordering::Acquire) {
            return std::task::Poll::Ready(());
        }

        {
            let mut waker = this
                .state
                .waker
                .lock()
                .expect("native sleep waker mutex poisoned");
            *waker = Some(cx.waker().clone());
        }

        if let Some(duration) = this.duration.take() {
            let state = Arc::clone(&this.state);
            std::thread::spawn(move || {
                std::thread::sleep(duration);
                state.completed.store(true, Ordering::Release);
                if let Some(waker) = state
                    .waker
                    .lock()
                    .expect("native sleep waker mutex poisoned")
                    .take()
                {
                    waker.wake();
                }
            });
        }

        std::task::Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Condition, ConditionType, Header};
    use std::collections::HashMap;
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

    #[test]
    fn test_async_incremental_preserves_request_variable_context() {
        let requests = vec![
            HttpRequest {
                name: Some("login".to_string()),
                method: "POST".to_string(),
                url: "https://example.com/login".to_string(),
                headers: vec![],
                body: None,
                assertions: vec![],
                variables: vec![],
                timeout: None,
                connection_timeout: None,
                depends_on: None,
                conditions: vec![],
                pre_delay_ms: None,
                post_delay_ms: None,
            },
            HttpRequest {
                name: Some("profile".to_string()),
                method: "GET".to_string(),
                url: "https://example.com/profile".to_string(),
                headers: vec![Header {
                    name: "Authorization".to_string(),
                    value: "Bearer {{login.response.body.$.token}}".to_string(),
                }],
                body: None,
                assertions: vec![],
                variables: vec![],
                timeout: None,
                connection_timeout: None,
                depends_on: None,
                conditions: vec![],
                pre_delay_ms: None,
                post_delay_ms: None,
            },
        ];

        let mut captured_request: Option<HttpRequest> = None;

        block_on(process_http_requests_incremental_async(
            requests,
            false,
            0,
            |idx, _total, result| {
                if idx == 1 {
                    if let AsyncRequestProcessingResult::Executed { request, .. } = result {
                        captured_request = Some(request);
                    }
                    return false;
                }

                true
            },
            &executor_with_login_token,
        ))
        .unwrap();

        let captured_request = captured_request.expect("expected selected request result");
        assert_eq!(captured_request.headers.len(), 1);
        assert_eq!(
            captured_request.headers[0].value,
            "Bearer secret-token".to_string()
        );
    }

    #[test]
    fn test_async_incremental_skips_unmet_dependencies() {
        let requests = vec![
            HttpRequest {
                name: Some("login".to_string()),
                method: "POST".to_string(),
                url: "https://example.com/login".to_string(),
                headers: vec![],
                body: None,
                assertions: vec![],
                variables: vec![],
                timeout: None,
                connection_timeout: None,
                depends_on: None,
                conditions: vec![],
                pre_delay_ms: None,
                post_delay_ms: None,
            },
            HttpRequest {
                name: Some("profile".to_string()),
                method: "GET".to_string(),
                url: "https://example.com/profile".to_string(),
                headers: vec![],
                body: None,
                assertions: vec![],
                variables: vec![],
                timeout: None,
                connection_timeout: None,
                depends_on: Some("login".to_string()),
                conditions: vec![],
                pre_delay_ms: None,
                post_delay_ms: None,
            },
        ];

        let mut dependency_reason: Option<String> = None;

        block_on(process_http_requests_incremental_async(
            requests,
            false,
            0,
            |idx, _total, result| {
                if idx == 1 {
                    if let AsyncRequestProcessingResult::Skipped { reason, .. } = result {
                        dependency_reason = Some(reason);
                    }
                    return false;
                }

                true
            },
            &executor_failed_login,
        ))
        .unwrap();

        assert_eq!(
            dependency_reason.as_deref(),
            Some("Dependency on 'login' not met")
        );
    }

    #[test]
    fn test_async_incremental_skips_when_conditions_fail() {
        let requests = vec![
            HttpRequest {
                name: Some("setup".to_string()),
                method: "GET".to_string(),
                url: "https://example.com/setup".to_string(),
                headers: vec![],
                body: None,
                assertions: vec![],
                variables: vec![],
                timeout: None,
                connection_timeout: None,
                depends_on: None,
                conditions: vec![],
                pre_delay_ms: None,
                post_delay_ms: None,
            },
            HttpRequest {
                name: Some("conditional".to_string()),
                method: "GET".to_string(),
                url: "https://example.com/conditional".to_string(),
                headers: vec![],
                body: None,
                assertions: vec![],
                variables: vec![],
                timeout: None,
                connection_timeout: None,
                depends_on: None,
                conditions: vec![Condition {
                    request_name: "setup".to_string(),
                    condition_type: ConditionType::BodyJsonPath("$.mode".to_string()),
                    expected_value: "allowed".to_string(),
                    negate: false,
                }],
                pre_delay_ms: None,
                post_delay_ms: None,
            },
        ];

        let mut skipped_reason: Option<String> = None;

        block_on(process_http_requests_incremental_async(
            requests,
            false,
            0,
            |idx, _total, result| {
                if idx == 1 {
                    if let AsyncRequestProcessingResult::Skipped { reason, .. } = result {
                        skipped_reason = Some(reason);
                    }
                    return false;
                }

                true
            },
            &executor_blocked_setup,
        ))
        .unwrap();

        assert_eq!(skipped_reason.as_deref(), Some("Conditions not met"));
    }

    fn success_result(name: Option<String>, body: Option<String>) -> HttpResult {
        HttpResult {
            request_name: name,
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 10,
            response_headers: Some(HashMap::new()),
            response_body: body,
            assertion_results: vec![],
        }
    }

    fn executor_with_login_token(
        request: &HttpRequest,
        _verbose: bool,
        _insecure: bool,
    ) -> AsyncRequestFuture<'_> {
        let request_name = request.name.clone();
        let body = if request_name.as_deref() == Some("login") {
            Some(r#"{"token":"secret-token"}"#.to_string())
        } else {
            None
        };

        Box::pin(async move { Ok(success_result(request_name, body)) })
    }

    fn executor_failed_login(
        request: &HttpRequest,
        _verbose: bool,
        _insecure: bool,
    ) -> AsyncRequestFuture<'_> {
        let request_name = request.name.clone();

        Box::pin(async move {
            Ok(HttpResult {
                request_name,
                status_code: 500,
                success: false,
                error_message: Some("login failed".to_string()),
                duration_ms: 10,
                response_headers: None,
                response_body: None,
                assertion_results: vec![],
            })
        })
    }

    fn executor_blocked_setup(
        request: &HttpRequest,
        _verbose: bool,
        _insecure: bool,
    ) -> AsyncRequestFuture<'_> {
        let request_name = request.name.clone();
        let body = if request_name.as_deref() == Some("setup") {
            Some(r#"{"mode":"blocked"}"#.to_string())
        } else {
            Some(r#"{"status":"ok"}"#.to_string())
        };

        Box::pin(async move { Ok(success_result(request_name, body)) })
    }

    fn block_on<F: Future>(future: F) -> F::Output {
        let waker = unsafe { Waker::from_raw(dummy_raw_waker()) };
        let mut context = Context::from_waker(&waker);
        let mut future = Box::pin(future);

        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(output) => return output,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    unsafe fn dummy_clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    unsafe fn dummy_no_op(_: *const ()) {}

    static DUMMY_WAKER_VTABLE: RawWakerVTable =
        RawWakerVTable::new(dummy_clone, dummy_no_op, dummy_no_op, dummy_no_op);

    fn dummy_raw_waker() -> RawWaker {
        RawWaker::new(std::ptr::null(), &DUMMY_WAKER_VTABLE)
    }
}
