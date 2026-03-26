mod formatter;
mod html;
mod markdown;
mod time_utils;
mod writer;

pub use html::{generate_html, generate_html_with_options};
pub use markdown::{generate_markdown, generate_markdown_with_options};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod formatter_tests;

#[cfg(test)]
mod html_tests;

#[cfg(test)]
mod time_utils_tests;
