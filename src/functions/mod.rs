mod generator;
mod substitution;

pub use generator::generate_guid;
pub use generator::generate_string;

pub use substitution::substitute_functions;

#[cfg(test)]
mod tests;
