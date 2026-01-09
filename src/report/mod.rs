mod formatter;
mod generator;
mod html_generator;
mod writer;

pub use generator::generate_markdown;
pub use html_generator::generate_html;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod formatter_tests;
