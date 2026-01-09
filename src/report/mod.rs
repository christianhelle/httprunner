mod formatter;
mod generator;
mod writer;

pub use generator::generate_markdown;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod formatter_tests;
