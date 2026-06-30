mod scanner;

pub use scanner::run_discovery_mode;

#[cfg(not(target_arch = "wasm32"))]
pub use scanner::discover_http_file_paths;

#[cfg(test)]
pub(crate) use scanner::discover_http_files;

#[cfg(test)]
mod tests;
