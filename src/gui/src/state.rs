use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct AppState {
    pub root_directory: Option<PathBuf>,
    pub selected_file: Option<PathBuf>,
    pub selected_environment: Option<String>,
    pub font_size: Option<f32>,
    pub window_size: Option<(f32, f32)>,
}

impl AppState {
    const STATE_FILE_NAME: &'static str = "httprunner-gui-state.json";

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

    pub fn load() -> Self {
        if let Some(state_path) = Self::get_state_file_path() {
            if state_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&state_path) {
                    if let Ok(state) = serde_json::from_str::<AppState>(&content) {
                        return state;
                    }
                }
            }
        }
        Self::default()
    }

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
}
