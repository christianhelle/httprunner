use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Focus state within the environment editor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorFocus {
    EnvironmentList,
    VariableList,
    Input,
}

/// Input mode for text entry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    None,
    NewEnvironment,
    NewVariableName,
    NewVariableValue,
    EditVariableValue,
}

pub struct EnvironmentEditor {
    /// The full environment config: env_name -> { var_name -> var_value }
    config: HashMap<String, HashMap<String, String>>,
    /// Path to the environment file
    env_file_path: Option<PathBuf>,
    /// Whether config has unsaved changes
    has_changes: bool,
    /// Sorted list of environment names
    env_names: Vec<String>,
    /// Selected environment index
    selected_env_index: usize,
    /// Sorted list of variable names for the selected environment
    var_names: Vec<String>,
    /// Selected variable index
    selected_var_index: usize,
    /// Current focus area
    focus: EditorFocus,
    /// Input mode
    input_mode: InputMode,
    /// Text input buffer
    input_buffer: String,
    /// Temporary storage for new variable name
    pending_var_name: String,
    /// Status message
    pub status_message: Option<String>,
}

impl EnvironmentEditor {
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
            env_file_path: None,
            has_changes: false,
            env_names: Vec::new(),
            selected_env_index: 0,
            var_names: Vec::new(),
            selected_var_index: 0,
            focus: EditorFocus::EnvironmentList,
            input_mode: InputMode::None,
            input_buffer: String::new(),
            pending_var_name: String::new(),
            status_message: None,
        }
    }

    /// Load environments from the env file associated with the given .http file path
    pub fn load_for_file(&mut self, http_file: &Path) {
        if let Some(file_str) = http_file.to_str()
            && let Ok(Some(env_file)) =
                httprunner_core::environment::find_environment_file(file_str)
            && let Ok(config) = httprunner_core::environment::parse_environment_file(&env_file)
        {
            self.config = config;
            self.env_file_path = Some(env_file);
        } else {
            self.config = HashMap::new();
            if let Some(parent) = http_file.parent() {
                self.env_file_path = Some(parent.join("http-client.env.json"));
            }
        }
        self.has_changes = false;
        self.refresh_env_names();
        self.refresh_var_names();
        self.status_message = None;
    }

    fn refresh_env_names(&mut self) {
        self.env_names = self.config.keys().cloned().collect();
        self.env_names.sort();
        if self.selected_env_index >= self.env_names.len() {
            self.selected_env_index = self.env_names.len().saturating_sub(1);
        }
    }

    fn refresh_var_names(&mut self) {
        if let Some(env_name) = self.env_names.get(self.selected_env_index) {
            if let Some(vars) = self.config.get(env_name) {
                self.var_names = vars.keys().cloned().collect();
                self.var_names.sort();
            } else {
                self.var_names.clear();
            }
        } else {
            self.var_names.clear();
        }
        if self.selected_var_index >= self.var_names.len() {
            self.selected_var_index = self.var_names.len().saturating_sub(1);
        }
    }

    /// Save the current config to file
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

    /// Get environment names
    pub fn environment_names(&self) -> Vec<String> {
        self.env_names.clone()
    }

    /// Check if there are unsaved changes
    pub fn has_changes(&self) -> bool {
        self.has_changes
    }

    /// Whether we're in input mode (text entry)
    pub fn is_in_input_mode(&self) -> bool {
        self.input_mode != InputMode::None
    }

    /// Get rendering data for the UI
    pub fn env_names(&self) -> &[String] {
        &self.env_names
    }

    pub fn selected_env_index(&self) -> usize {
        self.selected_env_index
    }

    pub fn selected_env_name(&self) -> Option<&str> {
        self.env_names.get(self.selected_env_index).map(|s| s.as_str())
    }

    pub fn var_names(&self) -> &[String] {
        &self.var_names
    }

    pub fn selected_var_index(&self) -> usize {
        self.selected_var_index
    }

    pub fn get_var_value(&self, env_name: &str, var_name: &str) -> Option<&str> {
        self.config
            .get(env_name)
            .and_then(|vars| vars.get(var_name))
            .map(|s| s.as_str())
    }

    pub fn is_env_list_focused(&self) -> bool {
        self.focus == EditorFocus::EnvironmentList
    }

    pub fn is_var_list_focused(&self) -> bool {
        self.focus == EditorFocus::VariableList
    }

    pub fn input_buffer(&self) -> &str {
        &self.input_buffer
    }

    pub fn input_prompt(&self) -> &str {
        match self.input_mode {
            InputMode::None => "",
            InputMode::NewEnvironment => "New environment name: ",
            InputMode::NewVariableName => "New variable name: ",
            InputMode::NewVariableValue => "Variable value: ",
            InputMode::EditVariableValue => "Edit value: ",
        }
    }

    /// Handle key events
    pub fn handle_key_event(&mut self, key: KeyEvent) {
        // Handle input mode first
        if self.input_mode != InputMode::None {
            self.handle_input_key(key);
            return;
        }

        match (key.code, key.modifiers) {
            // Save
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                self.save();
            }
            // Switch focus between env list and var list
            (KeyCode::Tab, _) | (KeyCode::Right, _) | (KeyCode::Left, _) => {
                self.focus = match self.focus {
                    EditorFocus::EnvironmentList => EditorFocus::VariableList,
                    EditorFocus::VariableList => EditorFocus::EnvironmentList,
                    EditorFocus::Input => EditorFocus::EnvironmentList,
                };
            }
            // Navigate
            (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                match self.focus {
                    EditorFocus::EnvironmentList => {
                        if self.selected_env_index > 0 {
                            self.selected_env_index -= 1;
                            self.refresh_var_names();
                        }
                    }
                    EditorFocus::VariableList => {
                        if self.selected_var_index > 0 {
                            self.selected_var_index -= 1;
                        }
                    }
                    EditorFocus::Input => {}
                }
            }
            (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
                match self.focus {
                    EditorFocus::EnvironmentList => {
                        if self.selected_env_index + 1 < self.env_names.len() {
                            self.selected_env_index += 1;
                            self.refresh_var_names();
                        }
                    }
                    EditorFocus::VariableList => {
                        if self.selected_var_index + 1 < self.var_names.len() {
                            self.selected_var_index += 1;
                        }
                    }
                    EditorFocus::Input => {}
                }
            }
            // Add new environment
            (KeyCode::Char('n'), KeyModifiers::NONE) => {
                if self.focus == EditorFocus::EnvironmentList {
                    self.input_mode = InputMode::NewEnvironment;
                    self.input_buffer.clear();
                    self.focus = EditorFocus::Input;
                }
            }
            // Add new variable
            (KeyCode::Char('a'), KeyModifiers::NONE) => {
                if self.focus == EditorFocus::VariableList && !self.env_names.is_empty() {
                    self.input_mode = InputMode::NewVariableName;
                    self.input_buffer.clear();
                    self.focus = EditorFocus::Input;
                }
            }
            // Edit variable value
            (KeyCode::Enter, _) | (KeyCode::Char('e'), KeyModifiers::NONE) => {
                if self.focus == EditorFocus::VariableList
                    && !self.var_names.is_empty()
                    && !self.env_names.is_empty()
                    && let Some(env_name) = self.env_names.get(self.selected_env_index)
                    && let Some(var_name) = self.var_names.get(self.selected_var_index)
                    && let Some(current_value) = self.get_var_value(env_name, var_name)
                {
                    self.input_buffer = current_value.to_string();
                    self.pending_var_name = var_name.clone();
                    self.input_mode = InputMode::EditVariableValue;
                    self.focus = EditorFocus::Input;
                }
            }
            // Delete
            (KeyCode::Char('d'), KeyModifiers::NONE) | (KeyCode::Delete, _) => {
                match self.focus {
                    EditorFocus::EnvironmentList => {
                        if let Some(env_name) = self.env_names.get(self.selected_env_index).cloned()
                        {
                            self.config.remove(&env_name);
                            self.has_changes = true;
                            self.refresh_env_names();
                            self.refresh_var_names();
                            self.status_message =
                                Some(format!("Deleted environment '{}'", env_name));
                        }
                    }
                    EditorFocus::VariableList => {
                        if let Some(env_name) = self.env_names.get(self.selected_env_index).cloned()
                            && let Some(var_name) =
                                self.var_names.get(self.selected_var_index).cloned()
                            && let Some(vars) = self.config.get_mut(&env_name)
                        {
                            vars.remove(&var_name);
                            self.has_changes = true;
                            self.refresh_var_names();
                            self.status_message =
                                Some(format!("Deleted variable '{}'", var_name));
                        }
                    }
                    EditorFocus::Input => {}
                }
            }
            _ => {}
        }
    }

    fn handle_input_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::None;
                self.input_buffer.clear();
                self.pending_var_name.clear();
                self.focus = EditorFocus::EnvironmentList;
            }
            KeyCode::Enter => {
                let value = self.input_buffer.trim().to_string();
                if !value.is_empty() {
                    match self.input_mode {
                        InputMode::NewEnvironment => {
                            if !self.config.contains_key(&value) {
                                self.config.insert(value.clone(), HashMap::new());
                                self.has_changes = true;
                                self.refresh_env_names();
                                // Select the new environment
                                if let Some(idx) = self.env_names.iter().position(|e| e == &value) {
                                    self.selected_env_index = idx;
                                }
                                self.refresh_var_names();
                                self.status_message =
                                    Some(format!("Added environment '{}'", value));
                            }
                            self.focus = EditorFocus::EnvironmentList;
                        }
                        InputMode::NewVariableName => {
                            self.pending_var_name = value;
                            self.input_buffer.clear();
                            self.input_mode = InputMode::NewVariableValue;
                            return; // Don't clear input mode yet
                        }
                        InputMode::NewVariableValue => {
                            if let Some(env_name) =
                                self.env_names.get(self.selected_env_index).cloned()
                                && let Some(vars) = self.config.get_mut(&env_name)
                            {
                                vars.insert(self.pending_var_name.clone(), value);
                                self.has_changes = true;
                                self.refresh_var_names();
                                self.status_message = Some(format!(
                                    "Added variable '{}'",
                                    self.pending_var_name
                                ));
                            }
                            self.pending_var_name.clear();
                            self.focus = EditorFocus::VariableList;
                        }
                        InputMode::EditVariableValue => {
                            if let Some(env_name) =
                                self.env_names.get(self.selected_env_index).cloned()
                                && let Some(vars) = self.config.get_mut(&env_name)
                            {
                                vars.insert(self.pending_var_name.clone(), value);
                                self.has_changes = true;
                                self.status_message = Some(format!(
                                    "Updated variable '{}'",
                                    self.pending_var_name
                                ));
                            }
                            self.pending_var_name.clear();
                            self.focus = EditorFocus::VariableList;
                        }
                        InputMode::None => {}
                    }
                }
                self.input_mode = InputMode::None;
                self.input_buffer.clear();
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
    }
}
