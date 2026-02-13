mod executor;
mod formatter;
mod incremental;
mod substitution;

pub use executor::{
    process_http_files, process_http_files_with_config, process_http_files_with_executor,
    process_http_files_with_silent, ProcessorConfig,
};

pub use formatter::format_json_if_valid;

pub use incremental::{
    process_http_file_incremental, process_http_file_incremental_with_executor,
    RequestProcessingResult,
};

#[cfg(test)]
mod mock_executor;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod executor_tests;

#[cfg(test)]
mod incremental_tests;
