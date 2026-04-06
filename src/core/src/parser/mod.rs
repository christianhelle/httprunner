mod condition_parser;
#[cfg(test)]
mod file_parser;
mod substitution;
mod timeout_parser;
mod utils;

mod pest_parse_tree;
mod pest_parser;
mod pest_semantic_assembler;

pub use pest_semantic_assembler::{parse_http_content, parse_http_file};

#[cfg(test)]
pub(crate) use file_parser::{
    parse_http_content as parse_http_content_with_legacy_backend,
    parse_http_file as parse_http_file_with_legacy_backend,
};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod timeout_parser_tests;

#[cfg(test)]
mod utils_tests;

#[cfg(test)]
mod condition_parser_tests;

#[cfg(test)]
mod substitution_tests;
