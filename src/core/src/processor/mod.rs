#[cfg(not(target_arch = "wasm32"))]
mod executor;
mod formatter;
#[cfg(not(target_arch = "wasm32"))]
mod incremental;
#[cfg(target_arch = "wasm32")]
mod incremental_async;
mod processing_result;
mod substitution;

#[cfg(not(target_arch = "wasm32"))]
pub use executor::{
    ProcessorConfig, process_http_files, process_http_files_with_config,
    process_http_files_with_executor, process_http_files_with_silent,
};

pub use formatter::format_json_if_valid;
pub use processing_result::RequestProcessingResult;

#[cfg(not(target_arch = "wasm32"))]
pub use incremental::{process_http_file_incremental, process_http_file_incremental_with_executor};

#[cfg(target_arch = "wasm32")]
pub use incremental_async::process_http_content_incremental_async;

#[cfg(test)]
mod mock_executor;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod executor_tests;

#[cfg(test)]
mod incremental_tests;
