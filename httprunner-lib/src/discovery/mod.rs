mod scanner;

pub use scanner::run_discovery_mode;

#[cfg(test)]
pub(crate) use scanner::discover_http_files;

#[cfg(test)]
mod tests;
