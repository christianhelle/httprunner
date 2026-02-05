#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
static FILE_NAME: &str = "support_key.txt";

#[cfg(target_arch = "wasm32")]
const LOCAL_STORAGE_KEY: &str = "httprunner-support-key";

#[allow(dead_code)]
pub struct SupportKey {
    pub key: String,
    pub short_key: String,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_support_key() -> Result<SupportKey, Box<dyn std::error::Error>> {
    if let Some(path) = get_state_file_path() {
        if path.exists() {
            let contents = std::fs::read_to_string(&path)?;
            let key = contents.trim().to_string();
            let n = std::cmp::min(8, key.len());
            let short_key: String = key.chars().take(n).collect();
            return Ok(SupportKey {
                key: key.clone(),
                short_key,
            });
        }
        let support_key = generate_support_key();
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(&path, &support_key.key)?;
        return Ok(support_key);
    }
    Ok(generate_support_key())
}

#[cfg(target_arch = "wasm32")]
pub fn get_support_key() -> Result<SupportKey, Box<dyn std::error::Error>> {
    if let Some(storage) = get_local_storage() {
        if let Ok(Some(key)) = storage.get_item(LOCAL_STORAGE_KEY) {
            let n = std::cmp::min(8, key.chars().count());
            let short_key: String = key.chars().take(n).collect();
            return Ok(SupportKey {
                key: key.clone(),
                short_key,
            });
        }
        // Generate and persist new key
        let support_key = generate_support_key();
        let _ = storage.set_item(LOCAL_STORAGE_KEY, &support_key.key);
        return Ok(support_key);
    }
    Ok(generate_support_key())
}

#[cfg(target_arch = "wasm32")]
fn get_local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

pub fn generate_support_key() -> SupportKey {
    let new_key = generate_uuid();
    let short_key = &new_key[..8];
    SupportKey {
        key: new_key.clone(),
        short_key: short_key.to_string(),
    }
}

fn generate_uuid() -> String {
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

#[cfg(not(target_arch = "wasm32"))]
fn get_state_file_path() -> Option<PathBuf> {
    use dirs;
    if let Some(config_dir) = dirs::config_dir() {
        let app_config_dir = config_dir.join("httprunner");
        Some(app_config_dir.join(FILE_NAME))
    } else {
        Some(PathBuf::from(FILE_NAME))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_support_key_creates_valid_key() {
        let support_key = generate_support_key();
        assert!(!support_key.key.is_empty());
        assert!(!support_key.short_key.is_empty());
    }

    #[test]
    fn generate_support_key_short_key_is_8_chars() {
        let support_key = generate_support_key();
        assert_eq!(support_key.short_key.len(), 8);
    }

    #[test]
    fn generate_support_key_short_key_is_prefix_of_key() {
        let support_key = generate_support_key();
        assert!(support_key.key.starts_with(&support_key.short_key));
    }

    #[test]
    fn generate_uuid_returns_32_char_hex_string() {
        let uuid = generate_uuid();
        assert_eq!(uuid.len(), 32);
        assert!(uuid.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn generate_uuid_is_unique() {
        let uuid1 = generate_uuid();
        let uuid2 = generate_uuid();
        assert_ne!(uuid1, uuid2);
    }

    #[test]
    fn generate_uuid_has_version_4_marker() {
        let uuid = generate_uuid();
        // UUID v4 has '4' at position 12 (13th char, 0-indexed at 12)
        assert_eq!(uuid.chars().nth(12), Some('4'));
    }

    #[test]
    fn generate_uuid_has_valid_variant_marker() {
        let uuid = generate_uuid();
        // UUID variant bits at position 16 should be 8, 9, a, or b
        let variant_char = uuid.chars().nth(16).unwrap();
        assert!(
            variant_char == '8'
                || variant_char == '9'
                || variant_char == 'a'
                || variant_char == 'b',
            "Expected variant char to be 8, 9, a, or b, got: {}",
            variant_char
        );
    }

    #[test]
    fn support_key_struct_fields_are_accessible() {
        let key = SupportKey {
            key: "test-key-12345678".to_string(),
            short_key: "test-key".to_string(),
        };
        assert_eq!(key.key, "test-key-12345678");
        assert_eq!(key.short_key, "test-key");
    }

    #[test]
    fn short_key_handles_short_input_safely() {
        // Simulate what happens with a key shorter than 8 chars
        let key = "abc";
        let n = std::cmp::min(8, key.len());
        let short_key: String = key.chars().take(n).collect();
        assert_eq!(short_key, "abc");
    }

    #[test]
    fn short_key_handles_utf8_safely() {
        // Test with multi-byte UTF-8 characters (9 chars, should take first 8)
        let key = "日本語テスト文字列";
        let n = std::cmp::min(8, key.chars().count());
        let short_key: String = key.chars().take(n).collect();
        assert_eq!(short_key.chars().count(), 8);
        assert_eq!(short_key, "日本語テスト文字");
    }

    #[test]
    fn short_key_handles_mixed_utf8_ascii() {
        let key = "abc日本xyz";
        let n = std::cmp::min(8, key.chars().count());
        let short_key: String = key.chars().take(n).collect();
        assert_eq!(short_key.chars().count(), 8);
        assert_eq!(short_key, "abc日本xyz");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn get_state_file_path_returns_some() {
        let path = get_state_file_path();
        assert!(path.is_some());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn get_state_file_path_ends_with_filename() {
        let path = get_state_file_path().unwrap();
        assert!(path.ends_with(FILE_NAME));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn get_support_key_returns_valid_key() {
        let result = get_support_key();
        assert!(result.is_ok());
        let support_key = result.unwrap();
        assert!(!support_key.key.is_empty());
        assert!(!support_key.short_key.is_empty());
        assert_eq!(support_key.short_key.len(), 8);
    }
}
