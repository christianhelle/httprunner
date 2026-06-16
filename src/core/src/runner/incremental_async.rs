use crate::processor::incremental_loop::{
    AsyncSleep, process_requests_incremental, RequestProcessingResult,
};
use anyhow::Result;

/// Re-export types from the unified loop for the async path.
pub use crate::processor::incremental_loop::{
    AsyncRequestExecutor, AsyncRequestFuture,
};
pub use crate::processor::RequestProcessingResult as AsyncRequestProcessingResult;

/// Process already-parsed HTTP requests incrementally with async sleep.
pub async fn process_http_requests_incremental_async<F>(
    requests: Vec<crate::types::HttpRequest>,
    insecure: bool,
    delay_ms: u64,
    callback: F,
    executor: &AsyncRequestExecutor,
) -> Result<()>
where
    F: FnMut(usize, usize, RequestProcessingResult) -> bool,
{
    let wrapped = |request: crate::types::HttpRequest, verbose: bool, insecure: bool| {
        async move { executor(&request, verbose, insecure).await }
    };
    process_requests_incremental(
        requests,
        insecure,
        delay_ms,
        callback,
        &wrapped,
        AsyncSleep,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        Assertion, AssertionType, Condition, ConditionType, Header, HttpRequest, HttpResult,
    };
    use std::collections::HashMap;
    use std::future::Future;
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
                    if let RequestProcessingResult::Executed { request, .. } = result {
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
                    if let RequestProcessingResult::Skipped { reason, .. } = result {
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
                    if let RequestProcessingResult::Skipped { reason, .. } = result {
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

    #[test]
    fn test_async_incremental_empty_requests() {
        block_on(process_http_requests_incremental_async(
            vec![],
            false,
            0,
            |_idx, _total, _result| true,
            &executor_trivial_ok,
        ))
        .unwrap();
    }

    #[test]
    fn test_async_incremental_executor_error() {
        let mut captured: Option<String> = None;

        block_on(process_http_requests_incremental_async(
            vec![HttpRequest {
                name: Some("fail".to_string()),
                method: "GET".to_string(),
                url: "https://example.com/fail".to_string(),
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
            }],
            false,
            0,
            |_idx, _total, result| {
                if let RequestProcessingResult::Failed { error, .. } = result {
                    captured = Some(error);
                }
                false
            },
            &executor_always_err,
        ))
        .unwrap();

        assert!(captured.is_some());
    }

    #[test]
    fn test_async_incremental_assertion_passed() {
        let mut captured_result: Option<HttpResult> = None;

        block_on(process_http_requests_incremental_async(
            vec![HttpRequest {
                name: Some("test".to_string()),
                method: "GET".to_string(),
                url: "https://example.com/test".to_string(),
                headers: vec![],
                body: None,
                assertions: vec![Assertion {
                    assertion_type: AssertionType::Status,
                    expected_value: "200".to_string(),
                }],
                variables: vec![],
                timeout: None,
                connection_timeout: None,
                depends_on: None,
                conditions: vec![],
                pre_delay_ms: None,
                post_delay_ms: None,
            }],
            false,
            0,
            |_idx, _total, result| {
                if let RequestProcessingResult::Executed { result, .. } = result {
                    captured_result = Some(result);
                }
                false
            },
            &executor_200_ok,
        ))
        .unwrap();

        let result = captured_result.expect("expected executed result");
        assert!(result.success);
        assert_eq!(result.assertion_results.len(), 1);
        assert!(result.assertion_results[0].passed);
    }

    #[test]
    fn test_async_incremental_assertion_failed() {
        let mut captured_result: Option<HttpResult> = None;

        block_on(process_http_requests_incremental_async(
            vec![HttpRequest {
                name: Some("test".to_string()),
                method: "GET".to_string(),
                url: "https://example.com/test".to_string(),
                headers: vec![],
                body: None,
                assertions: vec![Assertion {
                    assertion_type: AssertionType::Status,
                    expected_value: "404".to_string(),
                }],
                variables: vec![],
                timeout: None,
                connection_timeout: None,
                depends_on: None,
                conditions: vec![],
                pre_delay_ms: None,
                post_delay_ms: None,
            }],
            false,
            0,
            |_idx, _total, result| {
                if let RequestProcessingResult::Executed { result, .. } = result {
                    captured_result = Some(result);
                }
                false
            },
            &executor_200_ok,
        ))
        .unwrap();

        let result = captured_result.expect("expected executed result");
        assert!(!result.success);
        assert_eq!(result.assertion_results.len(), 1);
        assert!(!result.assertion_results[0].passed);
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

    fn executor_trivial_ok(
        request: &HttpRequest,
        _verbose: bool,
        _insecure: bool,
    ) -> AsyncRequestFuture<'_> {
        let request_name = request.name.clone();
        Box::pin(async move { Ok(success_result(request_name, None)) })
    }

    fn executor_always_err(
        _request: &HttpRequest,
        _verbose: bool,
        _insecure: bool,
    ) -> AsyncRequestFuture<'_> {
        Box::pin(async move { Err(anyhow::anyhow!("executor failed")) })
    }

    fn executor_200_ok(
        request: &HttpRequest,
        _verbose: bool,
        _insecure: bool,
    ) -> AsyncRequestFuture<'_> {
        let request_name = request.name.clone();
        Box::pin(async move {
            Ok(HttpResult {
                request_name,
                status_code: 200,
                success: true,
                error_message: None,
                duration_ms: 10,
                response_headers: Some(HashMap::new()),
                response_body: Some("OK".to_string()),
                assertion_results: vec![],
            })
        })
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
