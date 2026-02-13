mod address;
mod base64_encode;
mod date;
mod datetime;
mod email;
mod first_name;
mod guid;
mod job_title;
mod last_name;
mod lorem_ipsum;
mod lower;
mod name;
mod number;
mod string_gen;
mod substitution;
mod time;
mod upper;
mod utc_datetime;

pub use substitution::substitute_functions;

#[cfg(test)]
mod tests;
