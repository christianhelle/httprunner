pub fn generate_guid() -> String {
    use uuid::Uuid;
    Uuid::new_v4().as_simple().to_string()
}

pub fn generate_string(length: usize) -> String {
    use rand::Rng;
    use rand::distr::Alphanumeric;

    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
