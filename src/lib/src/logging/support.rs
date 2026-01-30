#[cfg(not(target_arch = "wasm32"))]
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
            // Check that key is valid (at least 8 characters)
            if key.len() >= 8 {
                let n = std::cmp::min(8, key.len());
                let short_key: String = key.chars().take(n).collect();
                return Ok(SupportKey {
                    key: key.clone(),
                    short_key,
                });
            }
            // Persisted key is invalid or too short; regenerate and overwrite.
        }
        let support_key = generate_support_key();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
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
    let new_key = crate::functions::generate_uuid_v4();
    let short_key = &new_key[..8];
    SupportKey {
        key: new_key.clone(),
        short_key: short_key.to_string(),
    }
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
