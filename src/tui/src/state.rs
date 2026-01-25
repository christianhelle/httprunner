use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppState {
    pub root_directory: Option<PathBuf>,
    pub selected_file: Option<PathBuf>,
    pub selected_environment: Option<String>,
    // GUI-specific fields (kept for potential future TUI features)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_size: Option<(f32, f32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_tree_visible: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results_compact_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_results: Option<String>,
}

impl AppState {
    pub fn load() -> Self {
        let config_dir = dirs::config_dir()
            .map(|d| d.join("httprunner"))
            .unwrap_or_else(|| PathBuf::from("."));

        let state_file = config_dir.join("tui_state.json");

        if state_file.exists()
            && let Ok(content) = std::fs::read_to_string(&state_file)
                && let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }

        Self {
            root_directory: None,
            selected_file: None,
            selected_environment: None,
            window_size: None,
            font_size: None,
            file_tree_visible: None,
            results_compact_mode: None,
            last_results: None,
        }
    }

    pub fn save(&self) {
        let config_dir = dirs::config_dir()
            .map(|d| d.join("httprunner"))
            .unwrap_or_else(|| PathBuf::from("."));

        if !config_dir.exists() {
            let _ = std::fs::create_dir_all(&config_dir);
        }

        let state_file = config_dir.join("tui_state.json");

        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&state_file, content);
        }
    }
}
