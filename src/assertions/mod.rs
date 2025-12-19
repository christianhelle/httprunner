mod evaluator;

pub use evaluator::evaluate_assertions;

#[cfg(test)]
pub(crate) use evaluator::evaluate_assertion;

#[cfg(test)]
mod tests;
