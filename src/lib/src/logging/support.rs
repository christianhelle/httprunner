use std::path::PathBuf;

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
            if key.len() >= 8 {
                let short_key = &key[..8];
                return Ok(SupportKey {
                    key: key.clone(),
                    short_key: short_key.to_string(),
                });
            }
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
