mod generators;
mod substitution;
mod values;

pub use substitution::substitute_functions;

#[cfg(test)]
mod tests;
