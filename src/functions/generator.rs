use crate::functions::substitution::FunctionSubstitutor;

pub struct GuidSubstitutor {}
impl FunctionSubstitutor for GuidSubstitutor {
    fn get_regex(&self) -> String {
        r"\bguid\(\)".to_string()
    }

    fn generate(&self) -> String {
        use uuid::Uuid;
        Uuid::new_v4().as_simple().to_string()
    }
}

pub struct StringSubstitutor {}
impl FunctionSubstitutor for StringSubstitutor {
    fn get_regex(&self) -> String {
        r"\bstring\(\)".to_string()
    }

    fn generate(&self) -> String {
        use rand::Rng;
        use rand::distr::Alphanumeric;

        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect()
    }
}

pub struct NumberSubstitutor {}
impl FunctionSubstitutor for NumberSubstitutor {
    fn get_regex(&self) -> String {
        r"\bnumber\(\)".to_string()
    }

    fn generate(&self) -> String {
        use rand::Rng;
        use rand::distr::Uniform;

        rand::rng()
            .sample(Uniform::new_inclusive(0, 100).unwrap())
            .to_string()
    }
}
