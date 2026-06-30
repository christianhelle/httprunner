mod executor;
mod formatter;
mod incremental;
pub(crate) mod incremental_loop;
mod output;

pub use executor::{ProcessorConfig, default_executor, process_http_files};

pub use formatter::format_json_if_valid;

pub use incremental::{
    RequestProcessingResult, process_http_file_incremental,
    process_http_file_incremental_with_executor,
};

#[cfg(test)]
mod mock_executor;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod executor_tests;

#[cfg(test)]
mod incremental_tests;
