use super::first_name::FirstNameSubstitutor;
use super::last_name::LastNameSubstitutor;
use super::substitution::FunctionSubstitutor;

pub struct NameSubstitutor {}
impl FunctionSubstitutor for NameSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bname\(\)"
    }

    fn generate(&self) -> String {
        let first_name = FirstNameSubstitutor {}.generate();
        let last_name = LastNameSubstitutor {}.generate();
        format!("{} {}", first_name, last_name).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_substitutor() {
        let sub = NameSubstitutor {};
        let name = sub.generate();
        assert!(!name.is_empty(), "Generated name should not be empty");
        assert!(
            name.contains(' '),
            "Generated name '{}' should contain a space between first and last name",
            name
        );
    }

    #[test]
    fn test_name_substitutor_regex() {
        let sub = NameSubstitutor {};
        let regex = regex::Regex::new(sub.get_regex()).unwrap();

        assert!(regex.is_match("name()"));
        assert!(regex.is_match("User: name()"));
        assert!(!regex.is_match("noname()"));
        assert!(!regex.is_match("myname()"));
    }

    #[test]
    fn test_name_word_boundary_strict() {
        let sub = NameSubstitutor {};
        let regex = regex::Regex::new(sub.get_regex()).unwrap();

        assert!(!regex.is_match("nameextra()"));
        assert!(!regex.is_match("prefixname()"));
    }

    #[test]
    fn test_full_name_structure() {
        let sub = NameSubstitutor {};
        let name = sub.generate();
        let parts: Vec<&str> = name.split(' ').collect();

        assert_eq!(parts.len(), 2, "Full name should have exactly 2 parts");
        assert!(!parts[0].is_empty(), "First name part should not be empty");
        assert!(!parts[1].is_empty(), "Last name part should not be empty");
    }
}
