pub fn parse_timeout_value(value: &str) -> Option<u64> {
    let value = value.trim();

    if let Some(stripped) = value.strip_suffix("ms") {
        let num_str = stripped.trim();
        num_str.parse::<u64>().ok()
    } else if let Some(stripped) = value.strip_suffix('m') {
        let num_str = stripped.trim();
        num_str
            .parse::<u64>()
            .ok()
            .and_then(|m| m.checked_mul(60_000))
    } else if let Some(stripped) = value.strip_suffix('s') {
        let num_str = stripped.trim();
        num_str
            .parse::<u64>()
            .ok()
            .and_then(|s| s.checked_mul(1_000))
    } else {
        value.parse::<u64>().ok().and_then(|s| s.checked_mul(1_000))
    }
}
