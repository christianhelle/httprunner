mod loader;

pub use loader::load_environment_file;

#[cfg(test)]
pub(crate) use loader::{find_environment_file, parse_environment_file};

#[cfg(test)]
mod tests;
