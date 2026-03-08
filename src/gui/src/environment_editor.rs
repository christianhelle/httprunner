use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Environment editor state for viewing and editing http-client.env.json files.
pub struct EnvironmentEditor {
    pub(crate) config: HashMap<String, HashMap<String, String>>,
    pub(crate) env_file_path: Option<PathBuf>,
    pub(crate) has_changes: bool,
    pub(crate) editing_environment: Option<String>,
    pub(crate) new_env_name: String,
    pub(crate) new_var_name: String,
    pub(crate) new_var_value: String,
    pub(crate) status_message: Option<String>,
    pub(crate) pending_delete_env: Option<String>,
}

impl EnvironmentEditor {
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
            env_file_path: None,
            has_changes: false,
            editing_environment: None,
            new_env_name: String::new(),
            new_var_name: String::new(),
            new_var_value: String::new(),
            status_message: None,
            pending_delete_env: None,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_for_file(&mut self, http_file: &Path) {
        if let Some(file_str) = http_file.to_str()
            && let Ok(Some(env_file)) =
                httprunner_core::environment::find_environment_file(file_str)
            && let Ok(config) = httprunner_core::environment::parse_environment_file(&env_file)
        {
            self.config = config;
            self.env_file_path = Some(env_file);
            self.editing_environment = self.environment_names().first().cloned();
            self.has_changes = false;
            self.status_message = None;
            self.pending_delete_env = None;
        } else {
            self.config = HashMap::new();
            self.editing_environment = None;
            if let Some(parent) = http_file.parent() {
                self.env_file_path = Some(parent.join("http-client.env.json"));
            }
            self.has_changes = false;
            self.status_message = None;
            self.pending_delete_env = None;
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load_for_file(&mut self, _http_file: &Path) {
        self.load_from_local_storage();
    }

    #[cfg(target_arch = "wasm32")]
    fn load_from_local_storage(&mut self) {
        use web_sys::window;

        if let Some(window) = window()
            && let Ok(Some(storage)) = window.local_storage()
            && let Ok(Some(json_str)) = storage.get_item("httprunner_env_config")
            && let Ok(config) =
                serde_json::from_str::<HashMap<String, HashMap<String, String>>>(&json_str)
        {
            self.config = config;
        } else {
            self.config = HashMap::new();
        }

        self.env_file_path = None;
        self.editing_environment = self.environment_names().first().cloned();
        self.has_changes = false;
        self.status_message = None;
        self.pending_delete_env = None;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&mut self) {
        if let Some(path) = &self.env_file_path {
            match httprunner_core::environment::save_environment_file(path, &self.config) {
                Ok(()) => {
                    self.has_changes = false;
                    self.status_message = Some("Environment file saved".to_string());
                }
                Err(error) => {
                    self.status_message = Some(format!("Failed to save: {}", error));
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&mut self) {
        use web_sys::window;

        if let Ok(json_str) = serde_json::to_string(&self.config) {
            if let Some(window) = window()
                && let Ok(Some(storage)) = window.local_storage()
            {
                match storage.set_item("httprunner_env_config", &json_str) {
                    Ok(()) => {
                        self.has_changes = false;
                        self.status_message = Some("Environment config saved".to_string());
                    }
                    Err(_) => {
                        self.status_message = Some("Failed to save to localStorage".to_string());
                    }
                }
            }
        }
    }

    pub fn environment_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.config.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn has_changes(&self) -> bool {
        self.has_changes
    }
}
