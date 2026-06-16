use crate::assertions;
use crate::conditions;
use crate::request_substitution::{
    substitute_functions_in_request, substitute_request_variables_in_request,
};
use crate::types::{HttpRequest, HttpResult, RequestContext};
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

/// Result of processing a single request during incremental execution.
#[derive(Debug)]
pub enum RequestProcessingResult {
    Skipped {
        request: HttpRequest,
        reason: String,
    },
    Executed {
        request: HttpRequest,
        result: HttpResult,
    },
    Failed { request: HttpRequest, error: String },
}

/// Abstraction over sleep mechanisms for sync and async paths.
pub trait Sleep {
    async fn sleep(&self, duration: Duration);
}

/// Sync sleep adapter — blocks the current thread.
pub struct SyncSleep;

impl Sleep for SyncSleep {
    async fn sleep(&self, duration: Duration) {
        if duration == Duration::ZERO {
            return;
        }
        std::thread::sleep(duration);
    }
}

/// Async sleep adapter — uses platform-appropriate async sleep.
pub struct AsyncSleep;

impl Sleep for AsyncSleep {
    async fn sleep(&self, duration: Duration) {
        async_sleep_ms(duration).await;
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn async_sleep_ms(duration: Duration) {
    if duration == Duration::ZERO {
        return;
    }
    NativeSleep::new(duration).await;
}

#[cfg(target_arch = "wasm32")]
async fn async_sleep_ms(duration: Duration) {
    if duration == Duration::ZERO {
        return;
    }
    gloo_timers::future::sleep(duration).await;
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

pub(crate) fn add_request_context(
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

/// Process requests incrementally with dependency checking, condition evaluation,
/// variable/function substitution, pre/post delays, and callback-driven control flow.
///
/// The executor is called with an owned `HttpRequest` (the loop clones it before
/// dispatching), so the original remains available for callback and context tracking.
pub async fn process_requests_incremental<F, Fut, S>(
    requests: Vec<HttpRequest>,
    insecure: bool,
    delay_ms: u64,
    mut callback: F,
    executor: &impl Fn(HttpRequest, bool, bool) -> Fut,
    sleep: S,
) -> Result<()>
where
    F: FnMut(usize, usize, RequestProcessingResult) -> bool,
    Fut: Future<Output = Result<HttpResult>>,
    S: Sleep,
{
    let total = requests.len();

    if requests.is_empty() {
        return Ok(());
    }

    let mut request_contexts: Vec<RequestContext> = Vec::new();

    for (idx, mut request) in requests.into_iter().enumerate() {
        let request_count = (idx + 1) as u32;

        if idx > 0 && delay_ms > 0 {
            sleep.sleep(Duration::from_millis(delay_ms)).await;
        }

        if let Some(dep_name) = request.depends_on.as_ref()
            && !conditions::check_dependency(&Some(dep_name.clone()), &request_contexts)
        {
            let should_continue = callback(
                idx,
                total,
                RequestProcessingResult::Skipped {
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
                Err(error) => {
                    let should_continue = callback(
                        idx,
                        total,
                        RequestProcessingResult::Failed {
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
                RequestProcessingResult::Failed {
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
                RequestProcessingResult::Failed {
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
            sleep.sleep(Duration::from_millis(pre_delay_ms)).await;
        }

        let post_delay_ms = request.post_delay_ms;

        // Clone the request for the executor so the original remains available
        // for the callback and context tracking.
        match executor(request.clone(), false, insecure).await {
            Ok(mut result) => {
                if !request.assertions.is_empty() {
                    let assertion_results =
                        assertions::evaluate_assertions(&request.assertions, &result);
                    let all_passed = assertion_results.iter().all(|r| r.passed);
                    result.success = all_passed;
                    result.assertion_results = assertion_results;
                }
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
            Err(error) => {
                let should_continue = callback(
                    idx,
                    total,
                    RequestProcessingResult::Failed {
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
            sleep.sleep(Duration::from_millis(post_delay_ms)).await;
        }
    }

    Ok(())
}

/// Block on a future using a no-op waker.
pub(crate) fn block_on<F: Future>(future: F) -> F::Output {
    use std::task::{Context, Poll};

    let waker = std::task::Waker::noop();
    let mut context = Context::from_waker(waker);
    let mut future = Box::pin(future);

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}
