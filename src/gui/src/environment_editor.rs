use dioxus::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EnvEditorState {
    pub config: HashMap<String, HashMap<String, String>>,
    pub env_file_path: Option<PathBuf>,
    pub has_changes: bool,
    pub editing_env: Option<String>,
    pub new_env_name: String,
    pub new_var_name: String,
    pub new_var_value: String,
    pub status_message: Option<String>,
    pub pending_delete_env: Option<String>,
}

impl EnvEditorState {
    pub fn environment_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.config.keys().cloned().collect();
        names.sort();
        names
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_for_file(&mut self, http_file: &std::path::Path) {
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
        self.status_message = None;
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load_for_file(&mut self, _http_file: &std::path::Path) {
        self.load_from_local_storage();
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load_from_local_storage(&mut self) {
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
        self.has_changes = false;
        self.status_message = None;
    }

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
                        self.status_message = Some("Failed to save to localStorage".to_string());
                    }
                }
            }
        }
    }
}

#[component]
pub fn EnvironmentEditor(mut state: Signal<EnvEditorState>) -> Element {
    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 8px;",
            h2 { "🌍 Environment Editor" }

            // File path info
            {
                let s = state();
                if let Some(ref path) = s.env_file_path {
                    rsx! {
                        p { style: "font-size: 12px; color: #8087a2;", "File: {path.display()}" }
                    }
                } else {
                    rsx! {
                        p { style: "font-size: 12px; color: #8087a2;", "Stored in browser localStorage" }
                    }
                }
            }

            // Status message
            if let Some(ref msg) = state().status_message {
                p { style: "color: #a6da95;", "{msg}" }
            }

            hr {}

            // Add new environment
            div {
                class: "flex items-center gap-8",
                label { style: "color: #8087a2;", "New environment:" }
                input {
                    r#type: "text",
                    style: "flex: 1;",
                    placeholder: "Environment name",
                    value: "{state().new_env_name}",
                    oninput: move |e| state.write().new_env_name = e.value(),
                    onkeydown: move |e| {
                        if e.key().to_string() == "Enter" {
                            let name = state().new_env_name.trim().to_string();
                            if !name.is_empty() && !state().config.contains_key(&name) {
                                let mut s = state.write();
                                s.config.insert(name.clone(), HashMap::new());
                                s.editing_env = Some(name);
                                s.has_changes = true;
                                s.status_message = None;
                                s.new_env_name.clear();
                            }
                        }
                    },
                }
                button {
                    onclick: move |_| {
                        let name = state().new_env_name.trim().to_string();
                        if !name.is_empty() && !state().config.contains_key(&name) {
                            let mut s = state.write();
                            s.config.insert(name.clone(), HashMap::new());
                            s.editing_env = Some(name);
                            s.has_changes = true;
                            s.status_message = None;
                            s.new_env_name.clear();
                        }
                    },
                    "➕ Add"
                }
            }

            // Environment tabs
            {
                let env_names = state().environment_names();
                if env_names.is_empty() {
                    rsx! { p { style: "color: #8087a2;", "No environments defined. Add one above." } }
                } else {
                    rsx! {
                        div {
                            class: "env-tabs",
                            for env_name in env_names.iter() {
                                {
                                    let name = env_name.clone();
                                    let nc = name.clone();
                                    let is_active = state().editing_env.as_deref() == Some(&name);
                                    rsx! {
                                        span {
                                            key: "{name}",
                                            class: if is_active { "env-tab active" } else { "env-tab" },
                                            onclick: move |_| {
                                                state.write().editing_env = Some(nc.clone());
                                                state.write().pending_delete_env = None;
                                            },
                                            "{name}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Variables for selected environment
            {
                let editing = state().editing_env.clone();
                if let Some(ref env_name) = editing {
                    let vars: Vec<(String, String)> = {
                        let mut v: Vec<_> = state()
                            .config
                            .get(env_name)
                            .cloned()
                            .unwrap_or_default()
                            .into_iter()
                            .collect();
                        v.sort_by(|a, b| a.0.cmp(&b.0));
                        v
                    };
                    let env_for_delete = env_name.clone();
                    let env_for_new = env_name.clone();

                    rsx! {
                        div {
                            // Header with delete button
                            div {
                                class: "flex items-center gap-8",
                                h3 { "📋 {env_name}" }
                                if state().pending_delete_env.as_deref() == Some(env_name) {
                                    span { style: "color: #ed8796;", "Delete this environment?" }
                                    button {
                                        style: "background: #ed8796; color: #24273a;",
                                        onclick: move |_| {
                                            let name = env_for_delete.clone();
                                            let mut s = state.write();
                                            s.config.remove(&name);
                                            s.editing_env = None;
                                            s.pending_delete_env = None;
                                            s.has_changes = true;
                                        },
                                        "Yes"
                                    }
                                    button {
                                        onclick: move |_| state.write().pending_delete_env = None,
                                        "No"
                                    }
                                } else {
                                    button {
                                        style: "margin-left: auto;",
                                        onclick: move |_| {
                                            state.write().pending_delete_env =
                                                Some(env_for_delete.clone());
                                        },
                                        "🗑 Delete"
                                    }
                                }
                            }

                            hr {}

                            // Variable table
                            table {
                                class: "var-table",
                                thead {
                                    tr {
                                        th { "Variable" }
                                        th { "Value" }
                                        th { "" }
                                    }
                                }
                                tbody {
                                    for (var_name, var_value) in vars.iter() {
                                        {
                                            let vn = var_name.clone();
                                            let vn2 = var_name.clone();
                                            let vn3 = var_name.clone();
                                            let env_ref = env_name.clone();
                                            let env_ref2 = env_name.clone();
                                            let vv = var_value.clone();
                                            rsx! {
                                                tr {
                                                    key: "{var_name}",
                                                    td { "{vn}" }
                                                    td {
                                                        input {
                                                            r#type: "text",
                                                            style: "width: 100%;",
                                                            value: "{vv}",
                                                            oninput: move |e| {
                                                                let mut s = state.write();
                                                                if let Some(env_vars) = s.config.get_mut(&env_ref) {
                                                                    env_vars.insert(vn2.clone(), e.value());
                                                                    s.has_changes = true;
                                                                    s.status_message = None;
                                                                }
                                                            },
                                                        }
                                                    }
                                                    td {
                                                        button {
                                                            onclick: move |_| {
                                                                let mut s = state.write();
                                                                if let Some(env_vars) = s.config.get_mut(&env_ref2) {
                                                                    env_vars.remove(&vn3);
                                                                    s.has_changes = true;
                                                                    s.status_message = None;
                                                                }
                                                            },
                                                            "🗑"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Add new variable row
                            div {
                                class: "flex items-center gap-8",
                                style: "margin-top: 8px;",
                                label { style: "color: #8087a2;", "Name:" }
                                input {
                                    r#type: "text",
                                    style: "flex: 1;",
                                    placeholder: "variable_name",
                                    value: "{state().new_var_name}",
                                    oninput: move |e| state.write().new_var_name = e.value(),
                                }
                                label { style: "color: #8087a2;", "Value:" }
                                input {
                                    r#type: "text",
                                    style: "flex: 1;",
                                    placeholder: "value",
                                    value: "{state().new_var_value}",
                                    oninput: move |e| state.write().new_var_value = e.value(),
                                }
                                button {
                                    onclick: move |_| {
                                        let vname = state().new_var_name.trim().to_string();
                                        if !vname.is_empty() {
                                            let vval = state().new_var_value.clone();
                                            let mut s = state.write();
                                            if let Some(env_vars) = s.config.get_mut(&env_for_new) {
                                                env_vars.insert(vname, vval);
                                                s.has_changes = true;
                                                s.status_message = None;
                                            }
                                            s.new_var_name.clear();
                                            s.new_var_value.clear();
                                        }
                                    },
                                    "➕ Add Variable"
                                }
                            }
                        }
                    }
                } else {
                    rsx! { div {} }
                }
            }

            hr {}

            // Save button
            div {
                class: "flex items-center gap-8",
                button {
                    onclick: move |_| state.write().save(),
                    "💾 Save"
                }
                if state().has_changes {
                    span { class: "warning", "● Unsaved changes" }
                }
            }
        }
    }
}
