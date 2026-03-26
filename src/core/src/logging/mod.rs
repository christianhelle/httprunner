mod support;
mod writer;

pub use support::get_support_key;
pub use writer::{Log, strip_ansi_codes};

#[cfg(test)]
mod tests;
