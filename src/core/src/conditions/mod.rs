mod dependency;
mod evaluator;
mod formatter;

pub use dependency::check_dependency;
pub use evaluator::{evaluate_conditions, evaluate_conditions_verbose};

#[allow(unused_imports)]
pub use evaluator::ConditionEvaluationResult;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod formatter_tests;
