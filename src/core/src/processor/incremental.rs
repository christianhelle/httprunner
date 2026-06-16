use super::incremental_loop::{SyncSleep, block_on, process_requests_incremental};
use crate::parser;
use crate::runner;
use crate::types::{HttpRequest, HttpResult};
use anyhow::Result;

pub use super::incremental_loop::RequestProcessingResult;

/// Process HTTP requests from a file with incremental callbacks for UI updates
pub fn process_http_file_incremental<F>(
    file_path: &str,
    environment: Option<&str>,
    insecure: bool,
    delay_ms: u64,
    callback: F,
) -> Result<()>
where
    F: FnMut(usize, usize, RequestProcessingResult) -> bool,
{
    process_http_file_incremental_with_executor(
        file_path,
        environment,
        insecure,
        delay_ms,
        callback,
        &|request, _verbose, insecure| runner::execute_http_request(request, false, insecure),
    )
}

/// Process HTTP requests from a file with incremental callbacks and a custom executor.
pub fn process_http_file_incremental_with_executor<F, E>(
    file_path: &str,
    environment: Option<&str>,
    insecure: bool,
    delay_ms: u64,
    callback: F,
    executor: &E,
) -> Result<()>
where
    F: FnMut(usize, usize, RequestProcessingResult) -> bool,
    E: Fn(&HttpRequest, bool, bool) -> Result<HttpResult>,
{
    let requests = parser::parse_http_file(file_path, environment)?;

    let wrapped = move |request: HttpRequest, verbose: bool, insecure: bool| {
        async move { executor(&request, verbose, insecure) }
    };

    block_on(process_requests_incremental(
        requests,
        insecure,
        delay_ms,
        callback,
        &wrapped,
        SyncSleep,
    ))
}
