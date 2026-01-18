mod generator;
mod substitution;

pub use substitution::substitute_functions;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod generator_tests;
