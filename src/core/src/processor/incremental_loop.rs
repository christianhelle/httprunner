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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Assertion, AssertionType, Condition, ConditionType};
    use std::pin::Pin;
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    };
    use std::task::{Context, Poll};

    struct MockSleep {
        calls: Arc<Mutex<Vec<Duration>>>,
    }

    impl MockSleep {
        fn new() -> Self {
            Self {
                calls: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl Sleep for MockSleep {
        async fn sleep(&self, duration: Duration) {
            self.calls.lock().unwrap().push(duration);
        }
    }

    fn make_result(name: Option<String>) -> HttpResult {
        HttpResult {
            request_name: name,
            status_code: 200,
            success: true,
            error_message: None,
            duration_ms: 10,
            response_headers: None,
            response_body: None,
            assertion_results: vec![],
        }
    }

    fn make_request(name: &str) -> HttpRequest {
        HttpRequest {
            name: Some(name.to_string()),
            method: "GET".to_string(),
            url: "https://example.com/test".to_string(),
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
        }
    }

    fn ok_executor() -> impl Fn(HttpRequest, bool, bool) -> Pin<Box<dyn Future<Output = Result<HttpResult>>>> {
        |request: HttpRequest, _verbose: bool, _insecure: bool| {
            let name = request.name.clone();
            Box::pin(async move { Ok(make_result(name)) })
        }
    }

    // --- Sleep trait tests ---

    #[test]
    fn test_sync_sleep_zero_duration() {
        let sleep = SyncSleep;
        block_on(sleep.sleep(Duration::ZERO));
    }

    #[test]
    fn test_sync_sleep_nonzero() {
        let sleep = SyncSleep;
        let start = std::time::Instant::now();
        block_on(sleep.sleep(Duration::from_millis(5)));
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(5));
    }

    // --- block_on tests ---

    #[test]
    fn test_block_on_ready() {
        let result = block_on(async { 42 });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_block_on_pending_once() {
        struct YieldOnce {
            yielded: bool,
        }
        impl Future for YieldOnce {
            type Output = &'static str;
            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                if self.yielded {
                    Poll::Ready("done")
                } else {
                    self.yielded = true;
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
        }
        let result = block_on(YieldOnce { yielded: false });
        assert_eq!(result, "done");
    }

    // --- process_requests_incremental tests ---

    #[test]
    fn test_process_requests_empty() {
        let executor = ok_executor();
        block_on(process_requests_incremental(
            vec![],
            false,
            0,
            |_, _, _| true,
            &executor,
            MockSleep::new(),
        ))
        .unwrap();
    }

    #[test]
    fn test_process_requests_single_execution() {
        let requests = vec![make_request("req1")];
        let executor = ok_executor();
        let results = Arc::new(Mutex::new(Vec::new()));
        let r = Arc::clone(&results);

        block_on(process_requests_incremental(
            requests,
            false,
            0,
            |idx, total, result| {
                r.lock().unwrap().push((idx, total, result));
                true
            },
            &executor,
            MockSleep::new(),
        ))
        .unwrap();

        let results = results.lock().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 0);
        assert_eq!(results[0].1, 1);
        assert!(matches!(results[0].2, RequestProcessingResult::Executed { .. }));
    }

    #[test]
    fn test_process_requests_multiple_executions() {
        let requests = vec![make_request("a"), make_request("b"), make_request("c")];
        let executor = ok_executor();
        let results = Arc::new(Mutex::new(Vec::new()));
        let r = Arc::clone(&results);

        block_on(process_requests_incremental(
            requests,
            false,
            0,
            |idx, total, result| {
                r.lock().unwrap().push((idx, total, result));
                true
            },
            &executor,
            MockSleep::new(),
        ))
        .unwrap();

        let results = results.lock().unwrap();
        assert_eq!(results.len(), 3);
        for (i, (idx, total, _)) in results.iter().enumerate() {
            assert_eq!(*idx, i);
            assert_eq!(*total, 3);
        }
    }

    #[test]
    fn test_process_requests_callback_stops_early() {
        let requests = vec![make_request("a"), make_request("b"), make_request("c")];
        let executor = ok_executor();
        let results = Arc::new(Mutex::new(Vec::new()));
        let r = Arc::clone(&results);

        block_on(process_requests_incremental(
            requests,
            false,
            0,
            |idx, _total, result| {
                r.lock().unwrap().push(result);
                idx < 1
            },
            &executor,
            MockSleep::new(),
        ))
        .unwrap();

        assert_eq!(results.lock().unwrap().len(), 2);
    }

    #[test]
    fn test_process_requests_inter_request_delay() {
        let requests = vec![make_request("a"), make_request("b")];
        let executor = ok_executor();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let mock_sleep = MockSleep { calls: Arc::clone(&calls) };

        block_on(process_requests_incremental(
            requests,
            false,
            50,
            |_, _, _| true,
            &executor,
            mock_sleep,
        ))
        .unwrap();

        assert_eq!(calls.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_process_requests_dependency_skip() {
        let requests = vec![HttpRequest {
            depends_on: Some("missing".to_string()),
            ..make_request("req1")
        }];
        let executor = ok_executor();
        let results = Arc::new(Mutex::new(Vec::new()));
        let r = Arc::clone(&results);

        block_on(process_requests_incremental(
            requests,
            false,
            0,
            |_idx, _total, result| {
                r.lock().unwrap().push(result);
                true
            },
            &executor,
            MockSleep::new(),
        ))
        .unwrap();

        let results = results.lock().unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0], RequestProcessingResult::Skipped { reason, .. }
            if reason.contains("Dependency")));
    }

    #[test]
    fn test_process_requests_condition_skip() {
        let requests = vec![HttpRequest {
            conditions: vec![Condition {
                request_name: "other".to_string(),
                condition_type: ConditionType::Status,
                expected_value: "200".to_string(),
                negate: false,
            }],
            ..make_request("req1")
        }];
        let executor = ok_executor();
        let results = Arc::new(Mutex::new(Vec::new()));
        let r = Arc::clone(&results);

        block_on(process_requests_incremental(
            requests,
            false,
            0,
            |_idx, _total, result| {
                r.lock().unwrap().push(result);
                true
            },
            &executor,
            MockSleep::new(),
        ))
        .unwrap();

        let results = results.lock().unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0], RequestProcessingResult::Skipped { reason, .. }
            if reason == "Conditions not met"));
    }

    #[test]
    fn test_process_requests_executor_error() {
        let requests = vec![make_request("req1")];
        let results = Arc::new(Mutex::new(Vec::new()));
        let r = Arc::clone(&results);

        let err_executor = |_: HttpRequest, _: bool, _: bool| {
            Box::pin(async move { Err(anyhow::anyhow!("executor failure")) })
                as Pin<Box<dyn Future<Output = Result<HttpResult>>>>
        };

        block_on(process_requests_incremental(
            requests,
            false,
            0,
            |_idx, _total, result| {
                r.lock().unwrap().push(result);
                true
            },
            &err_executor,
            MockSleep::new(),
        ))
        .unwrap();

        let results = results.lock().unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0], RequestProcessingResult::Failed { error, .. }
            if error == "executor failure"));
    }

    #[test]
    fn test_process_requests_assertion_failure() {
        let requests = vec![HttpRequest {
            assertions: vec![Assertion {
                assertion_type: AssertionType::Status,
                expected_value: "404".to_string(),
            }],
            ..make_request("req1")
        }];
        let executor = ok_executor();
        let results = Arc::new(Mutex::new(Vec::new()));
        let r = Arc::clone(&results);

        block_on(process_requests_incremental(
            requests,
            false,
            0,
            |_idx, _total, result| {
                r.lock().unwrap().push(result);
                true
            },
            &executor,
            MockSleep::new(),
        ))
        .unwrap();

        let results = results.lock().unwrap();
        assert_eq!(results.len(), 1);
        match &results[0] {
            RequestProcessingResult::Executed { result, .. } => {
                assert!(!result.success);
                assert_eq!(result.assertion_results.len(), 1);
                assert!(!result.assertion_results[0].passed);
            }
            _ => panic!("expected Executed result"),
        }
    }

    #[test]
    fn test_process_requests_no_delay_when_zero() {
        let requests = vec![make_request("a")];
        let executor = ok_executor();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let mock_sleep = MockSleep { calls: Arc::clone(&calls) };

        block_on(process_requests_incremental(
            requests,
            false,
            0,
            |_, _, _| true,
            &executor,
            mock_sleep,
        ))
        .unwrap();

        assert_eq!(calls.lock().unwrap().len(), 0);
    }

    #[test]
    fn test_process_requests_pre_delay() {
        let requests = vec![HttpRequest {
            pre_delay_ms: Some(5),
            ..make_request("req1")
        }];
        let calls = Arc::new(Mutex::new(Vec::new()));
        let mock_sleep = MockSleep { calls: Arc::clone(&calls) };
        let executed = Arc::new(AtomicBool::new(false));
        let exec_flag = Arc::clone(&executed);

        let exec = move |_: HttpRequest, _: bool, _: bool| {
            exec_flag.store(true, Ordering::SeqCst);
            Box::pin(async move { Ok(make_result(None)) })
                as Pin<Box<dyn Future<Output = Result<HttpResult>>>>
        };

        block_on(process_requests_incremental(
            requests,
            false,
            0,
            |_, _, _| true,
            &exec,
            mock_sleep,
        ))
        .unwrap();

        assert!(executed.load(Ordering::SeqCst));
        let calls = calls.lock().unwrap();
        assert!(calls.iter().any(|d| *d == Duration::from_millis(5)));
    }

    #[test]
    fn test_process_requests_post_delay() {
        let requests = vec![HttpRequest {
            post_delay_ms: Some(5),
            ..make_request("req1")
        }];
        let calls = Arc::new(Mutex::new(Vec::new()));
        let mock_sleep = MockSleep { calls: Arc::clone(&calls) };
        let executor = ok_executor();

        block_on(process_requests_incremental(
            requests,
            false,
            0,
            |_, _, _| true,
            &executor,
            mock_sleep,
        ))
        .unwrap();

        let calls = calls.lock().unwrap();
        assert!(calls.iter().any(|d| *d == Duration::from_millis(5)));
    }

    #[test]
    fn test_process_requests_callback_receives_correct_indices() {
        let executor = ok_executor();
        let results = Arc::new(Mutex::new(Vec::new()));
        let r = Arc::clone(&results);

        let requests = vec![
            make_request("a"),
            HttpRequest {
                depends_on: Some("nonexistent".to_string()),
                ..make_request("b")
            },
            make_request("c"),
        ];

        block_on(process_requests_incremental(
            requests,
            false,
            0,
            |idx, total, result| {
                r.lock().unwrap().push((idx, total, result));
                true
            },
            &executor,
            MockSleep::new(),
        ))
        .unwrap();

        let results = results.lock().unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, 0);
        assert_eq!(results[1].0, 1);
        assert_eq!(results[2].0, 2);
        assert_eq!(results[0].1, 3);
        assert!(matches!(&results[1].2, RequestProcessingResult::Skipped { .. }));
        assert!(matches!(&results[2].2, RequestProcessingResult::Executed { .. }));
    }
}
