mod formatter;
mod markdown;
mod html;
mod writer;

pub use markdown::generate_markdown;
pub use html::generate_html;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod formatter_tests;
