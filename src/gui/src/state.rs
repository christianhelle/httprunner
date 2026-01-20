use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Import ExecutionResult from results_view
use crate::results_view::ExecutionResult;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct AppState {
    pub root_directory: Option<PathBuf>,
    pub selected_file: Option<PathBuf>,
    pub selected_environment: Option<String>,
    pub font_size: Option<f32>,
    pub window_size: Option<(f32, f32)>,
    pub last_results: Option<Vec<ExecutionResult>>,
    pub file_tree_visible: Option<bool>,
    pub results_compact_mode: Option<bool>,
}

impl AppState {
    const STATE_FILE_NAME: &'static str = "httprunner-gui-state.json";
    #[cfg(target_arch = "wasm32")]
    const LOCAL_STORAGE_KEY: &'static str = "httprunner-gui-state";

    #[cfg(not(target_arch = "wasm32"))]
    fn get_state_file_path() -> Option<PathBuf> {
        // Use platform-specific config directory
        if let Some(config_dir) = dirs::config_dir() {
            let app_config_dir = config_dir.join("httprunner");
            Some(app_config_dir.join(Self::STATE_FILE_NAME))
        } else {
            // Fallback to current directory
            Some(PathBuf::from(Self::STATE_FILE_NAME))
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load() -> Self {
        if let Some(state_path) = Self::get_state_file_path()
            && state_path.exists()
            && let Ok(content) = std::fs::read_to_string(&state_path)
            && let Ok(state) = serde_json::from_str::<AppState>(&content)
        {
            return state;
        }
        Self::default()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load() -> Self {
        if let Some(storage) = Self::get_local_storage()
            && let Ok(Some(state_str)) = storage.get_item(Self::LOCAL_STORAGE_KEY)
                && let Ok(state) = serde_json::from_str::<AppState>(&state_str) {
                    return state;
                }
        Self::default()
    }

    #[cfg(target_arch = "wasm32")]
    fn get_local_storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(state_path) = Self::get_state_file_path() {
            // Create parent directory if it doesn't exist
            if let Some(parent) = state_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let json = serde_json::to_string_pretty(self)?;
            std::fs::write(state_path, json)?;
        }
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(storage) = Self::get_local_storage() {
            let json = serde_json::to_string(self)?;
            storage
                .set_item(Self::LOCAL_STORAGE_KEY, &json)
                .map_err(|e| anyhow::anyhow!("Failed to save to local storage: {:?}", e))?;
        }
        Ok(())
    }
}
