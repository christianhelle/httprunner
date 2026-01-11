mod dependency;
mod evaluator;
mod formatter;
mod json_extractor;

pub use dependency::check_dependency;
pub use evaluator::{evaluate_conditions, evaluate_conditions_verbose};

#[allow(unused_imports)]
pub use evaluator::ConditionEvaluationResult;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod json_extractor_tests;

#[cfg(test)]
mod formatter_tests;
