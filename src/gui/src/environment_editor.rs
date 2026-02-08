use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Environment editor component for viewing and editing http-client.env.json files
pub struct EnvironmentEditor {
    /// The full environment config: env_name -> { var_name -> var_value }
    config: HashMap<String, HashMap<String, String>>,
    /// Path to the environment file (None on WASM)
    env_file_path: Option<PathBuf>,
    /// Whether config has unsaved changes
    has_changes: bool,
    /// Currently selected environment name for editing
    editing_environment: Option<String>,
    /// New environment name input
    new_env_name: String,
    /// New variable name input
    new_var_name: String,
    /// New variable value input
    new_var_value: String,
    /// Status message to display
    status_message: Option<String>,
    /// Pending delete environment name (for confirmation)
    pending_delete_env: Option<String>,
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

    /// Load environments from the env file associated with the given .http file path
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_for_file(&mut self, http_file: &Path) {
        if let Some(file_str) = http_file.to_str()
            && let Ok(Some(env_file)) =
                httprunner_core::environment::find_environment_file(file_str)
            && let Ok(config) = httprunner_core::environment::parse_environment_file(&env_file)
        {
            self.config = config;
            self.env_file_path = Some(env_file);
            self.has_changes = false;
            self.status_message = None;
        } else {
            // No env file found — start fresh, will create on save
            self.config = HashMap::new();
            // Default to creating env file next to the .http file
            if let Some(parent) = http_file.parent() {
                self.env_file_path = Some(parent.join("http-client.env.json"));
            }
            self.has_changes = false;
            self.status_message = None;
        }
    }

    /// Load environments from localStorage on WASM
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
            && let Ok(config) = serde_json::from_str::<HashMap<String, HashMap<String, String>>>(&json_str)
        {
            self.config = config;
        } else {
            self.config = HashMap::new();
        }
        self.env_file_path = None;
        self.has_changes = false;
        self.status_message = None;
    }

    /// Save the current config
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&mut self) {
        if let Some(ref path) = self.env_file_path {
            match httprunner_core::environment::save_environment_file(path, &self.config) {
                Ok(()) => {
                    self.has_changes = false;
                    self.status_message = Some("Environment file saved".to_string());
                }
                Err(e) => {
                    self.status_message = Some(format!("Failed to save: {}", e));
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
                        self.status_message =
                            Some("Failed to save to localStorage".to_string());
                    }
                }
            }
        }
    }

    /// Get environment names for use by the app (e.g. populating environment selector)
    pub fn environment_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.config.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get the full config (for WASM env variable resolution)
    pub fn get_config(&self) -> &HashMap<String, HashMap<String, String>> {
        &self.config
    }

    /// Check if there are unsaved changes
    pub fn has_changes(&self) -> bool {
        self.has_changes
    }

    /// Display the environment editor UI
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("🌍 Environment Editor");
        ui.separator();

        // Show file path info
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(ref path) = self.env_file_path {
            ui.label(
                egui::RichText::new(format!("File: {}", path.display()))
                    .small()
                    .color(egui::Color32::GRAY),
            );
        }

        #[cfg(target_arch = "wasm32")]
        ui.label(
            egui::RichText::new("Stored in browser localStorage")
                .small()
                .color(egui::Color32::GRAY),
        );

        ui.separator();

        // Status message
        if let Some(ref msg) = self.status_message {
            ui.colored_label(egui::Color32::from_rgb(100, 200, 100), msg);
            ui.separator();
        }

        // Add new environment section
        ui.horizontal(|ui| {
            ui.label("New environment:");
            ui.text_edit_singleline(&mut self.new_env_name);
            if ui.button("➕ Add").clicked() && !self.new_env_name.trim().is_empty() {
                let name = self.new_env_name.trim().to_string();
                if !self.config.contains_key(&name) {
                    self.config.insert(name.clone(), HashMap::new());
                    self.editing_environment = Some(name);
                    self.has_changes = true;
                    self.status_message = None;
                }
                self.new_env_name.clear();
            }
        });

        ui.separator();

        // Environment tabs / selector
        let env_names = self.environment_names();

        if env_names.is_empty() {
            ui.label("No environments defined. Add one above.");
            return;
        }

        // Environment selection buttons
        ui.horizontal_wrapped(|ui| {
            for env_name in &env_names {
                let is_selected = self.editing_environment.as_ref() == Some(env_name);
                if ui
                    .selectable_label(is_selected, env_name)
                    .clicked()
                {
                    self.editing_environment = Some(env_name.clone());
                    self.pending_delete_env = None;
                }
            }
        });

        ui.separator();

        // Show variables for the selected environment
        if let Some(ref editing_env) = self.editing_environment.clone() {
            if let Some(vars) = self.config.get(editing_env).cloned() {
                ui.horizontal(|ui| {
                    ui.heading(
                        egui::RichText::new(format!("📋 {}", editing_env)).size(16.0),
                    );

                    // Delete environment button
                    if self.pending_delete_env.as_ref() == Some(editing_env) {
                        ui.colored_label(egui::Color32::RED, "Delete this environment?");
                        if ui.button("Yes").clicked() {
                            self.config.remove(editing_env);
                            self.editing_environment = None;
                            self.pending_delete_env = None;
                            self.has_changes = true;
                            return;
                        }
                        if ui.button("No").clicked() {
                            self.pending_delete_env = None;
                        }
                    } else if ui.button("🗑 Delete").clicked() {
                        self.pending_delete_env = Some(editing_env.clone());
                    }
                });

                ui.separator();

                // Variable table
                let mut vars_to_remove: Vec<String> = Vec::new();
                let mut vars_to_update: Vec<(String, String)> = Vec::new();

                let mut sorted_vars: Vec<(String, String)> = vars.into_iter().collect();
                sorted_vars.sort_by(|a, b| a.0.cmp(&b.0));

                egui::Grid::new("env_vars_grid")
                    .striped(true)
                    .num_columns(3)
                    .min_col_width(100.0)
                    .show(ui, |ui| {
                        ui.strong("Variable");
                        ui.strong("Value");
                        ui.strong("");
                        ui.end_row();

                        for (var_name, var_value) in &sorted_vars {
                            ui.label(var_name);

                            let mut value = var_value.clone();
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut value).desired_width(300.0),
                            );
                            if response.changed() {
                                vars_to_update.push((var_name.clone(), value));
                            }

                            if ui.button("🗑").clicked() {
                                vars_to_remove.push(var_name.clone());
                            }
                            ui.end_row();
                        }
                    });

                // Apply updates
                if let Some(env_vars) = self.config.get_mut(editing_env) {
                    for (name, value) in vars_to_update {
                        env_vars.insert(name, value);
                        self.has_changes = true;
                        self.status_message = None;
                    }
                    for name in vars_to_remove {
                        env_vars.remove(&name);
                        self.has_changes = true;
                        self.status_message = None;
                    }
                }

                ui.separator();

                // Add new variable
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.new_var_name);
                    ui.label("Value:");
                    ui.text_edit_singleline(&mut self.new_var_value);
                    if ui.button("➕ Add Variable").clicked()
                        && !self.new_var_name.trim().is_empty()
                    {
                        if let Some(env_vars) = self.config.get_mut(editing_env) {
                            env_vars.insert(
                                self.new_var_name.trim().to_string(),
                                self.new_var_value.clone(),
                            );
                            self.has_changes = true;
                            self.status_message = None;
                        }
                        self.new_var_name.clear();
                        self.new_var_value.clear();
                    }
                });
            }
        }

        ui.separator();

        // Save button and change indicator
        ui.horizontal(|ui| {
            if ui.button("💾 Save").clicked() {
                self.save();
            }

            if self.has_changes {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 165, 0),
                    "● Unsaved changes",
                );
            }
        });
    }
}
