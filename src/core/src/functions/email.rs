use super::first_name::FirstNameSubstitutor;
use super::last_name::LastNameSubstitutor;
use super::substitution::FunctionSubstitutor;

pub(crate) static EMAIL_DOMAINS: &[&str] = &[
    "example.com",
    "mail.com",
    "test.org",
    "demo.net",
    "sample.co",
    "mydomain.io",
    "webmail.com",
    "inbox.org",
    "email.net",
    "domain.co",
];

pub struct EmailSubstitutor {}
impl FunctionSubstitutor for EmailSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bemail\(\)"
    }

    fn generate(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..EMAIL_DOMAINS.len());
        let domain = EMAIL_DOMAINS[index].to_string();
        let first_name = FirstNameSubstitutor {}.generate();
        let last_name = LastNameSubstitutor {}.generate();

        format!(
            "{}.{}@{}",
            normalize_name_for_email(first_name),
            normalize_name_for_email(last_name),
            domain
        )
        .to_string()
    }
}

fn normalize_name_for_email(name: String) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_substitutor_generates_value() {
        let sub = EmailSubstitutor {};
        let email = sub.generate();
        assert!(!email.is_empty(), "Generated email should not be empty");
    }

    #[test]
    fn test_email_substitutor_format() {
        let sub = EmailSubstitutor {};
        let email = sub.generate();

        assert!(email.contains('@'), "Email '{}' should contain @", email);

        let parts: Vec<&str> = email.split('@').collect();
        assert_eq!(
            parts.len(),
            2,
            "Email '{}' should have exactly 2 parts separated by @",
            email
        );
        assert!(!parts[0].is_empty(), "Email local part should not be empty");
        assert!(!parts[1].is_empty(), "Email domain should not be empty");
    }

    #[test]
    fn test_email_substitutor_local_part_format() {
        let sub = EmailSubstitutor {};
        let email = sub.generate();

        let local_part = email.split('@').next().unwrap();

        assert!(
            local_part.contains('.'),
            "Email local part '{}' should contain a dot (format: firstname.lastname)",
            local_part
        );

        let parts: Vec<&str> = local_part.split('.').collect();
        assert_eq!(
            parts.len(),
            2,
            "Email local part '{}' should have exactly 2 parts separated by dot",
            local_part
        );

        assert!(!parts[0].is_empty(), "First name part should not be empty");
        assert!(!parts[1].is_empty(), "Last name part should not be empty");
    }

    #[test]
    fn test_email_substitutor_lowercase() {
        let sub = EmailSubstitutor {};
        let email = sub.generate();

        assert_eq!(
            email,
            email.to_lowercase(),
            "Email '{}' should be entirely lowercase",
            email
        );
    }

    #[test]
    fn test_email_substitutor_valid_domain() {
        let sub = EmailSubstitutor {};

        let valid_domains = vec![
            "example.com",
            "mail.com",
            "test.org",
            "demo.net",
            "sample.co",
            "mydomain.io",
            "webmail.com",
            "inbox.org",
            "email.net",
            "domain.co",
        ];

        let email = sub.generate();
        let domain = email.split('@').nth(1).unwrap();

        assert!(
            valid_domains.contains(&domain),
            "Email domain '{}' should be one of the valid domains",
            domain
        );
    }

    #[test]
    fn test_email_substitutor_generates_different_values() {
        let sub = EmailSubstitutor {};
        let email1 = sub.generate();
        let email2 = sub.generate();
        let email3 = sub.generate();

        assert!(!email1.is_empty());
        assert!(!email2.is_empty());
        assert!(!email3.is_empty());
    }

    #[test]
    fn test_email_substitutor_regex() {
        let sub = EmailSubstitutor {};
        let regex = regex::Regex::new(sub.get_regex()).unwrap();

        assert!(regex.is_match("email()"), "Should match email()");
        assert!(
            regex.is_match("Contact: email()"),
            "Should match email() with prefix"
        );
        assert!(!regex.is_match("noemail()"), "Should not match noemail()");
        assert!(!regex.is_match("myemail()"), "Should not match myemail()");
    }

    #[test]
    fn test_email_substitutor_word_boundary_strict() {
        let sub = EmailSubstitutor {};
        let regex = regex::Regex::new(sub.get_regex()).unwrap();

        assert!(regex.is_match("email()"));
        assert!(regex.is_match("email()extra"));
        assert!(!regex.is_match("prefixemail()"));
        assert!(!regex.is_match("_email()"));
    }

    #[test]
    fn test_email_generation_contains_lowercase_names() {
        let sub = EmailSubstitutor {};

        for _ in 0..20 {
            let email = sub.generate();
            let local_part = email.split('@').next().unwrap();

            assert!(
                local_part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c == '.'),
                "Email local part '{}' should be lowercase ASCII alphanumeric with dots",
                local_part
            );
        }
    }

    #[test]
    fn test_email_local_part_contains_dot() {
        let sub = EmailSubstitutor {};

        for _ in 0..30 {
            let email = sub.generate();
            let local_part = email.split('@').next().unwrap();
            let dot_count = local_part.matches('.').count();

            assert_eq!(
                dot_count, 1,
                "Email local part '{}' should have exactly 1 dot",
                local_part
            );
        }
    }

    #[test]
    fn test_email_consistency_across_domains() {
        let sub = EmailSubstitutor {};

        let valid_domains = vec![
            "example.com",
            "mail.com",
            "test.org",
            "demo.net",
            "sample.co",
            "mydomain.io",
            "webmail.com",
            "inbox.org",
            "email.net",
            "domain.co",
        ];

        for _ in 0..50 {
            let email = sub.generate();
            let domain = email.split('@').nth(1).unwrap();
            assert!(
                valid_domains.contains(&domain),
                "Domain '{}' should be in valid list",
                domain
            );
        }
    }
}
