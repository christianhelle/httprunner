mod formatter;
mod html;
mod markdown;
mod time_utils;
mod writer;

pub use html::generate_html;
pub use markdown::generate_markdown;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod formatter_tests;

#[cfg(test)]
mod html_tests;
