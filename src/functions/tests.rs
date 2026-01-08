use super::*;

#[test]
fn test_generate() {
    assert_eq!(generate_guid().len(), 32);
}
