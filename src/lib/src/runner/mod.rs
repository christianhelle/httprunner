mod executor;

#[cfg(target_arch = "wasm32")]
mod executor_async;

#[cfg(not(target_arch = "wasm32"))]
pub use executor::execute_http_request;

#[cfg(target_arch = "wasm32")]
pub use executor_async::execute_http_request_async;
