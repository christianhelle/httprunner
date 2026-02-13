use super::substitution::FunctionSubstitutor;

pub struct GuidSubstitutor {}
impl FunctionSubstitutor for GuidSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bguid\(\)"
    }

    fn generate(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 16];
        rng.fill(&mut bytes);
        format!(
            "{:08x}{:04x}{:04x}{:04x}{:012x}",
            u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            u16::from_be_bytes([bytes[4], bytes[5]]),
            (u16::from_be_bytes([bytes[6], bytes[7]]) & 0x0fff) | 0x4000,
            (u16::from_be_bytes([bytes[8], bytes[9]]) & 0x3fff) | 0x8000,
            u64::from_be_bytes([
                0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
            ]) & 0xffffffffffff
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_generate_guid() {
        let guid = GuidSubstitutor {}.generate();
        let hex_pattern = Regex::new(r"^[0-9a-fA-F]{32}$").unwrap();
        assert!(
            hex_pattern.is_match(&guid),
            "GUID '{}' does not match pattern /^[0-9a-fA-F]{{32}}$/",
            guid
        );
    }

    #[test]
    fn test_guid_substitutor_regex() {
        let sub = GuidSubstitutor {};
        let regex = regex::Regex::new(sub.get_regex()).unwrap();

        assert!(regex.is_match("guid()"));
        assert!(regex.is_match("Bearer guid()"));
        assert!(!regex.is_match("noguid()"));
        assert!(!regex.is_match("myguid()"));
    }

    #[test]
    fn test_guid_substitutor_generates_valid_uuid() {
        let sub = GuidSubstitutor {};
        let guid = sub.generate();

        assert_eq!(guid.len(), 32);
        assert!(guid.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_guid_substitutor_generates_unique_values() {
        let sub = GuidSubstitutor {};
        let guid1 = sub.generate();
        let guid2 = sub.generate();

        assert_ne!(guid1, guid2);
    }

    #[test]
    fn test_guid_is_valid_uuid_v4_format() {
        let guid = GuidSubstitutor {}.generate();

        assert_eq!(guid.len(), 32, "GUID should be 32 hex characters");

        for (i, c) in guid.chars().enumerate() {
            assert!(
                c.is_ascii_hexdigit(),
                "Character at position {} ('{}') is not hex",
                i,
                c
            );
        }

        assert_eq!(
            guid.chars().nth(12),
            Some('4'),
            "Version should be 4 at position 12"
        );

        let variant = guid.chars().nth(16).unwrap();
        assert!(
            ['8', '9', 'a', 'b', 'A', 'B'].contains(&variant),
            "Variant should be 8, 9, a, or b at position 16, got {}",
            variant
        );
    }

    #[test]
    fn test_guid_uniqueness_large_sample() {
        use std::collections::HashSet;

        let sub = GuidSubstitutor {};
        let mut guids = HashSet::new();

        for _ in 0..100 {
            let guid = sub.generate();
            assert!(
                guids.insert(guid.clone()),
                "Generated duplicate GUID: {}",
                guid
            );
        }

        assert_eq!(guids.len(), 100, "Should have 100 unique GUIDs");
    }

    #[test]
    fn test_guid_not_matches_with_prefix() {
        let sub = GuidSubstitutor {};
        let regex = regex::Regex::new(sub.get_regex()).unwrap();

        assert!(!regex.is_match("myguid()"));
        assert!(!regex.is_match("_guid()"));
        assert!(!regex.is_match("guid_"));
    }

    #[test]
    fn test_guid_word_boundary_strict() {
        let sub = GuidSubstitutor {};
        let regex = regex::Regex::new(sub.get_regex()).unwrap();

        assert!(regex.is_match("guid()"));
        assert!(regex.is_match("guid()extra"));
        assert!(!regex.is_match("prefixguid()"));
        assert!(!regex.is_match("my_guid()"));
    }

    #[test]
    fn test_guid_version_4_compliance() {
        let sub = GuidSubstitutor {};

        for _ in 0..50 {
            let guid = sub.generate();
            assert_eq!(
                guid.chars().nth(12),
                Some('4'),
                "UUID version should be 4 at position 12 in {}",
                guid
            );
        }
    }

    #[test]
    fn test_guid_variant_validation() {
        let sub = GuidSubstitutor {};
        let valid_variants = ['8', '9', 'a', 'b', 'A', 'B'];

        for _ in 0..50 {
            let guid = sub.generate();
            let variant = guid.chars().nth(16).unwrap();
            assert!(
                valid_variants.contains(&variant),
                "UUID variant at position 16 should be one of {:?}, got {} in {}",
                valid_variants,
                variant,
                guid
            );
        }
    }

    #[test]
    fn test_guid_generation_uniqueness_large_set() {
        use std::collections::HashSet;

        let sub = GuidSubstitutor {};
        let mut values = HashSet::new();

        for _ in 0..1000 {
            assert!(values.insert(sub.generate()), "Generated duplicate GUID");
        }

        assert_eq!(values.len(), 1000);
    }

    #[test]
    fn test_rapid_guid_generation() {
        let sub = GuidSubstitutor {};
        let mut guids = Vec::new();

        for _ in 0..100 {
            guids.push(sub.generate());
        }

        for guid in guids {
            assert_eq!(guid.len(), 32);
            assert!(guid.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn test_regex_does_not_match_function_with_arguments() {
        let sub = GuidSubstitutor {};
        let regex = regex::Regex::new(sub.get_regex()).unwrap();

        assert!(regex.is_match("guid()"));
    }
}
