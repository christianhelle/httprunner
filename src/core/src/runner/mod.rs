mod executor;
mod incremental_async;
mod response_processor;

#[cfg(target_arch = "wasm32")]
mod executor_async;

#[cfg(not(target_arch = "wasm32"))]
pub use executor::execute_http_request;

pub use incremental_async::{
    AsyncRequestExecutor, AsyncRequestFuture, AsyncRequestProcessingResult,
    process_http_requests_incremental_async,
};

#[cfg(target_arch = "wasm32")]
pub use executor_async::execute_http_request_async;
