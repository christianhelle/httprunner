use super::substitution::FunctionSubstitutor;

pub struct NumberSubstitutor {}
impl FunctionSubstitutor for NumberSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bnumber\(\)"
    }

    fn generate(&self) -> String {
        use rand::Rng;

        rand::thread_rng().gen_range(0..=100).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_number() {
        let number_str = NumberSubstitutor {}.generate();
        let number: i32 = number_str
            .parse()
            .expect("Generated number string could not be parsed as i32");
        assert!(
            (0..=100).contains(&number),
            "Generated number {} is not within range 0..=100",
            number
        );
        assert_ne!(number, -1, "Generated number should not be -1");
    }

    #[test]
    fn test_number_substitutor_regex() {
        let sub = NumberSubstitutor {};
        let regex = regex::Regex::new(sub.get_regex()).unwrap();

        assert!(regex.is_match("number()"));
        assert!(regex.is_match("value: number()"));
        assert!(!regex.is_match("nonumber()"));
        assert!(!regex.is_match("mynumber()"));
    }

    #[test]
    fn test_number_substitutor_generates_in_range() {
        let sub = NumberSubstitutor {};

        for _ in 0..100 {
            let num_str = sub.generate();
            let num: i32 = num_str.parse().unwrap();
            assert!((0..=100).contains(&num));
        }
    }

    #[test]
    fn test_number_substitutor_generates_numeric_string() {
        let sub = NumberSubstitutor {};
        let num_str = sub.generate();

        assert!(num_str.parse::<i32>().is_ok());
    }

    #[test]
    fn test_number_not_matches_with_prefix() {
        let sub = NumberSubstitutor {};
        let regex = regex::Regex::new(sub.get_regex()).unwrap();

        assert!(!regex.is_match("mynumber()"));
        assert!(!regex.is_match("_number()"));
        assert!(!regex.is_match("number_()"));
    }

    #[test]
    fn test_number_word_boundary_strict() {
        let sub = NumberSubstitutor {};
        let regex = regex::Regex::new(sub.get_regex()).unwrap();

        assert!(!regex.is_match("numberextra()"));
        assert!(!regex.is_match("prefixnumber()"));
    }

    #[test]
    fn test_number_at_boundaries() {
        let sub = NumberSubstitutor {};

        let mut found_zero = false;
        let mut found_hundred = false;

        for _ in 0..1000 {
            let num_str = sub.generate();
            let num: i32 = num_str.parse().unwrap();
            assert!((0..=100).contains(&num), "Number {} out of range", num);

            if num == 0 {
                found_zero = true;
            }
            if num == 100 {
                found_hundred = true;
            }
        }

        assert!(
            found_zero || found_hundred,
            "Should generate boundary values in 1000 iterations"
        );
    }

    #[test]
    fn test_number_distribution() {
        let sub = NumberSubstitutor {};
        let mut counts = [0; 101];

        for _ in 0..1000 {
            let num_str = sub.generate();
            let num: usize = num_str.parse().unwrap();
            counts[num] += 1;
        }

        let non_zero_count = counts.iter().filter(|&&c| c > 0).count();
        assert!(
            non_zero_count >= 50,
            "Expected at least 50 different numbers in 1000 iterations, got {}",
            non_zero_count
        );
    }

    #[test]
    fn test_number_boundary_comprehensive() {
        let sub = NumberSubstitutor {};
        let mut has_low = false;
        let mut has_mid = false;
        let mut has_high = false;

        for _ in 0..500 {
            let num_str = sub.generate();
            let num: i32 = num_str.parse().unwrap();

            if num <= 10 {
                has_low = true;
            }
            if (40..=60).contains(&num) {
                has_mid = true;
            }
            if num >= 90 {
                has_high = true;
            }
        }

        assert!(has_low, "Should generate some low numbers");
        assert!(has_mid, "Should generate some mid-range numbers");
        assert!(has_high, "Should generate some high numbers");
    }

    #[test]
    fn test_rapid_number_generation() {
        let sub = NumberSubstitutor {};

        for _ in 0..100 {
            let num_str = sub.generate();
            let num: i32 = num_str.parse().unwrap();
            assert!((0..=100).contains(&num));
        }
    }
}
