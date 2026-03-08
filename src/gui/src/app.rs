use crate::{
    environment_editor::{EnvironmentEditor, EnvEditorState},
    request_view::{RequestView, RequestViewAction, RequestViewState},
    results_view::{ExecutionResult, ResultsView},
    state::AppState,
    text_editor::TextEditor,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::file_tree::FileTree;
use dioxus::prelude::*;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ViewMode {
    TextEditor,
    RequestDetails,
    EnvironmentEditor,
}

const APP_CSS: &str = r#"
* { box-sizing: border-box; margin: 0; padding: 0; }
html, body { height: 100%; overflow: hidden; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif; font-size: 14px; background: #24273a; color: #cad3f5; }
.app { display: flex; flex-direction: column; height: 100vh; user-select: none; outline: none; }
.top-bar { display: flex; align-items: center; padding: 4px 8px; gap: 6px; background: #1e2030; border-bottom: 1px solid #363a4f; flex-shrink: 0; min-height: 36px; }
.content-area { display: flex; flex: 1; overflow: hidden; }
.file-tree { width: 240px; border-right: 1px solid #363a4f; overflow-y: auto; padding: 6px; background: #1e2030; flex-shrink: 0; }
.main-panel { display: flex; flex-direction: column; flex: 1; overflow: hidden; }
.view-tabs { display: flex; align-items: center; padding: 4px 8px; gap: 4px; background: #1e2030; border-bottom: 1px solid #363a4f; flex-shrink: 0; }
.editor-section { overflow: auto; flex-shrink: 0; padding: 8px; }
.splitter { height: 5px; background: #363a4f; cursor: row-resize; flex-shrink: 0; }
.splitter:hover { background: #8aadf4; }
.results-section { flex: 1; overflow-y: auto; padding: 8px; }
.bottom-bar { display: flex; align-items: center; padding: 2px 8px; gap: 8px; background: #1e2030; border-top: 1px solid #363a4f; font-size: 12px; color: #8087a2; flex-shrink: 0; }
button { padding: 4px 10px; background: #363a4f; color: #cad3f5; border: 1px solid #494d64; border-radius: 4px; cursor: pointer; font-size: 13px; white-space: nowrap; }
button:hover { background: #494d64; }
button.active { background: #8aadf4; color: #24273a; border-color: #8aadf4; }
button:disabled { opacity: 0.5; cursor: not-allowed; }
input[type="text"], input[type="number"], textarea, select { background: #1e2030; color: #cad3f5; border: 1px solid #494d64; border-radius: 4px; padding: 4px 8px; font-family: inherit; font-size: 13px; outline: none; }
input[type="text"]:focus, textarea:focus, select:focus { border-color: #8aadf4; }
textarea { resize: none; font-family: 'Consolas', 'JetBrains Mono', monospace; width: 100%; }
select { cursor: pointer; }
.menu { position: relative; }
.menu-popup { position: absolute; top: calc(100% + 2px); left: 0; background: #1e2030; border: 1px solid #494d64; border-radius: 6px; z-index: 1000; min-width: 220px; padding: 4px; box-shadow: 0 4px 16px rgba(0,0,0,0.5); }
.menu-item { display: block; width: 100%; padding: 6px 12px; cursor: pointer; text-align: left; background: none; border: none; color: #cad3f5; border-radius: 4px; font-size: 13px; }
.menu-item:hover { background: #363a4f; }
.menu-separator { height: 1px; background: #363a4f; margin: 4px 0; }
.dir-header { cursor: pointer; padding: 3px 4px; display: flex; align-items: center; gap: 4px; border-radius: 4px; font-size: 13px; }
.dir-header:hover { background: #363a4f; }
.file-item { cursor: pointer; padding: 2px 4px 2px 20px; border-radius: 4px; font-size: 13px; }
.file-item:hover { background: #363a4f; }
.file-item.selected { background: #8aadf4; color: #24273a; }
.result-card { padding: 8px 10px; margin: 4px 0; border-radius: 6px; border-left: 3px solid; }
.result-success { border-left-color: #a6da95; background: rgba(166,218,149,0.08); }
.result-failure { border-left-color: #ed8796; background: rgba(237,135,150,0.08); }
.result-running { border-left-color: #8aadf4; background: rgba(138,173,244,0.08); }
.success { color: #a6da95; }
.failure { color: #ed8796; }
.running { color: #8aadf4; }
.warning { color: #eed49f; }
pre, code, .mono { font-family: 'Consolas', 'JetBrains Mono', monospace; font-size: 13px; white-space: pre-wrap; word-break: break-all; }
.code-block { background: #1e2030; border: 1px solid #363a4f; border-radius: 4px; padding: 8px; max-height: 200px; overflow-y: auto; }
.spinner { display: inline-block; width: 14px; height: 14px; border: 2px solid #363a4f; border-top-color: #8aadf4; border-radius: 50%; animation: spin 0.8s linear infinite; vertical-align: middle; }
@keyframes spin { to { transform: rotate(360deg); } }
.form-row { display: flex; align-items: center; gap: 8px; margin: 4px 0; }
.form-row label { min-width: 80px; color: #8087a2; font-size: 13px; }
.collapsible > .collapse-header { cursor: pointer; padding: 6px 8px; background: #2a2e44; border-radius: 4px; display: flex; align-items: center; gap: 4px; user-select: none; }
.collapsible > .collapse-header:hover { background: #363a4f; }
.collapse-content { padding: 4px 0 4px 12px; }
.env-tabs { display: flex; flex-wrap: wrap; gap: 4px; margin: 4px 0; }
.env-tab { padding: 3px 10px; border-radius: 4px; background: #363a4f; cursor: pointer; font-size: 13px; }
.env-tab:hover { background: #494d64; }
.env-tab.active { background: #8aadf4; color: #24273a; }
.var-table { width: 100%; border-collapse: collapse; }
.var-table th { text-align: left; padding: 4px 8px; border-bottom: 1px solid #363a4f; color: #8087a2; font-weight: 600; font-size: 12px; }
.var-table td { padding: 4px 8px; border-bottom: 1px solid rgba(54,58,79,0.3); }
h2 { font-size: 16px; font-weight: 600; color: #b7bdf8; margin: 4px 0; }
h3 { font-size: 14px; font-weight: 600; color: #b7bdf8; margin: 4px 0; }
.section-title { font-weight: 600; color: #b7bdf8; font-size: 14px; }
hr { border: none; border-top: 1px solid #363a4f; margin: 6px 0; }
.flex { display: flex; }
.items-center { align-items: center; }
.flex-wrap { flex-wrap: wrap; }
.gap-8 { gap: 8px; }
input[type="range"] { accent-color: #8aadf4; }
"#;

pub fn App() -> Element {
    let saved = AppState::load();

    let mut root_directory: Signal<PathBuf> = use_signal(move || {
        saved.root_directory.clone()
            .and_then(|p| if p.exists() { Some(p) } else { None })
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    });
    let mut selected_file: Signal<Option<PathBuf>> = use_signal(move || {
        saved.selected_file.clone().and_then(|p| if p.exists() { Some(p) } else { None })
    });
    let mut selected_environment: Signal<Option<String>> = use_signal(move || saved.selected_environment.clone());
    let mut font_size: Signal<f32> = use_signal(move || saved.font_size.unwrap_or(14.0));
    let mut file_tree_visible: Signal<bool> = use_signal(move || saved.file_tree_visible.unwrap_or(true));
    let mut telemetry_enabled: Signal<bool> = use_signal(move || saved.telemetry_enabled.unwrap_or(true));
    let mut delay_ms: Signal<u64> = use_signal(move || saved.delay_ms.unwrap_or(0));
    let mut editor_panel_ratio: Signal<f32> = use_signal(move || saved.editor_panel_ratio.unwrap_or(0.5));
    let mut results_compact_mode: Signal<bool> = use_signal(move || saved.results_compact_mode.unwrap_or(true));
    let mut view_mode: Signal<ViewMode> = use_signal(|| ViewMode::TextEditor);
    let mut environments: Signal<Vec<String>> = use_signal(Vec::new);
    let mut text_content: Signal<String> = use_signal(String::new);
    let mut text_has_changes: Signal<bool> = use_signal(|| false);
    let mut req_view_state: Signal<RequestViewState> = use_signal(RequestViewState::default);
    let mut env_editor_state: Signal<EnvEditorState> = use_signal(EnvEditorState::default);
    let mut results: Signal<Vec<ExecutionResult>> = use_signal(move || {
        saved.last_results.clone().unwrap_or_default()
    });
    let mut is_running: Signal<bool> = use_signal(|| false);
    let mut is_dragging: Signal<bool> = use_signal(|| false);
    let mut drag_last_y: Signal<f64> = use_signal(|| 0.0);
    let mut file_menu_open: Signal<bool> = use_signal(|| false);
    let mut settings_menu_open: Signal<bool> = use_signal(|| false);

    // Load initial file state
    use_effect(move || {
        if let Some(path) = selected_file() {
            load_file_state(&path, &mut req_view_state, &mut env_editor_state, &mut environments);
        }
    });

    let do_save_state = move || {
        let state = AppState {
            root_directory: Some(root_directory()),
            selected_file: selected_file(),
            selected_environment: selected_environment(),
            font_size: Some(font_size()),
            window_size: None,
            last_results: Some(results().into_iter().filter(|r| !matches!(r, ExecutionResult::Running { .. })).collect()),
            file_tree_visible: Some(file_tree_visible()),
            results_compact_mode: Some(results_compact_mode()),
            telemetry_enabled: Some(telemetry_enabled()),
            delay_ms: Some(delay_ms()),
            editor_panel_ratio: Some(editor_panel_ratio()),
        };
        if let Err(e) = state.save() {
            eprintln!("Failed to save state: {}", e);
        }
    };

    let do_run_all = move || {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(file) = selected_file() {
            crate::results_view::run_file(file, selected_environment(), delay_ms(), results, is_running);
        }
        #[cfg(target_arch = "wasm32")]
        crate::results_view::run_content_async(text_content(), selected_environment(), results, is_running);
    };

    let do_run_single = move |idx: usize| {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(file) = selected_file() {
            crate::results_view::run_single_request(file, idx, selected_environment(), delay_ms(), results, is_running);
        }
        #[cfg(target_arch = "wasm32")]
        crate::results_view::run_single_request_async(text_content(), idx, selected_environment(), results, is_running);
    };

    let do_save_text = move || {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = selected_file() {
            if std::fs::write(&path, text_content()).is_ok() {
                text_has_changes.set(false);
            }
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(w) = web_sys::window() {
            if let Ok(Some(s)) = w.local_storage() {
                let _ = s.set_item("httprunner_editor_content", &text_content());
                text_has_changes.set(false);
            }
        }
    };

    let handle_req_action = move |action: RequestViewAction| {
        match action {
            RequestViewAction::RunRequest(idx) => do_run_single(idx),
            RequestViewAction::SaveFile => {
                if let Err(e) = req_view_state.write().save_to_file() {
                    eprintln!("Failed to save requests: {}", e);
                } else {
                    #[cfg(not(target_arch = "wasm32"))]
                    if let Some(path) = selected_file() {
                        if let Ok(txt) = std::fs::read_to_string(&path) {
                            text_content.set(txt);
                            text_has_changes.set(false);
                        }
                    }
                }
            }
            RequestViewAction::None => {}
        }
    };

    let handle_file_selected = move |path: PathBuf| {
        selected_file.set(Some(path.clone()));
        load_file_state(&path, &mut req_view_state, &mut env_editor_state, &mut environments);
        do_save_state();
    };

    let handle_keydown = move |e: Event<KeyboardData>| {
        let ctrl = e.modifiers().ctrl();
        let code = e.code().to_string();
        match code.as_str() {
            "F5" => do_run_all(),
            "KeyR" if ctrl => do_run_all(),
            "KeyQ" if ctrl => { do_save_state(); #[cfg(not(target_arch = "wasm32"))] std::process::exit(0); }
            "KeyO" if ctrl => {
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    root_directory.set(path);
                    selected_file.set(None);
                    do_save_state();
                }
            }
            "KeyE" if ctrl => {
                let envs = environments();
                if !envs.is_empty() {
                    let next = if let Some(ref cur) = selected_environment() {
                        let idx = envs.iter().position(|e| e == cur).map(|i| (i + 1) % (envs.len() + 1));
                        match idx {
                            Some(i) if i < envs.len() => Some(envs[i].clone()),
                            _ => None,
                        }
                    } else { envs.first().cloned() };
                    selected_environment.set(next);
                    do_save_state();
                }
            }
            "KeyT" if ctrl => view_mode.set(match view_mode() {
                ViewMode::TextEditor => ViewMode::RequestDetails,
                ViewMode::RequestDetails => ViewMode::EnvironmentEditor,
                ViewMode::EnvironmentEditor => ViewMode::TextEditor,
            }),
            "KeyB" if ctrl => { file_tree_visible.set(!file_tree_visible()); do_save_state(); }
            "KeyD" if ctrl => { results_compact_mode.set(!results_compact_mode()); do_save_state(); }
            "KeyS" if ctrl => match view_mode() {
                ViewMode::TextEditor => do_save_text(),
                ViewMode::RequestDetails => { let _ = req_view_state.write().save_to_file(); }
                ViewMode::EnvironmentEditor => {
                    env_editor_state.write().save();
                    environments.set(env_editor_state().environment_names());
                }
            },
            "Equal" if ctrl => { font_size.set((font_size() + 1.0).min(32.0)); }
            "Minus" if ctrl => { font_size.set((font_size() - 1.0).max(8.0)); }
            "Digit0" if ctrl => { font_size.set(14.0); }
            _ => {}
        }
    };

    let env_val = selected_environment().unwrap_or_else(|| "__none__".to_string());
    let ratio_pct = format!("{}%", (editor_panel_ratio() * 100.0) as u32);
    let run_enabled = match view_mode() {
        ViewMode::TextEditor => selected_file().is_some() && !text_has_changes(),
        ViewMode::RequestDetails => selected_file().is_some() && !req_view_state().has_changes(),
        ViewMode::EnvironmentEditor => false,
    };
    #[cfg(target_arch = "wasm32")]
    let run_enabled = view_mode() != ViewMode::EnvironmentEditor;

    rsx! {
        document::Style { {APP_CSS} }

        div {
            class: "app",
            tabindex: "0",
            autofocus: true,
            onkeydown: handle_keydown,
            onmousemove: move |e| {
                if is_dragging() {
                    let y = e.client_coordinates().y;
                    let delta = y - drag_last_y();
                    drag_last_y.set(y);
                    let new_r = (editor_panel_ratio() + delta as f32 / 600.0).clamp(0.1, 0.9);
                    editor_panel_ratio.set(new_r);
                }
            },
            onmouseup: move |_| {
                if is_dragging() { is_dragging.set(false); do_save_state(); }
            },

            // TOP BAR
            div {
                class: "top-bar",
                onclick: move |_| { file_menu_open.set(false); settings_menu_open.set(false); },

                #[cfg(not(target_arch = "wasm32"))]
                div {
                    class: "menu",
                    button {
                        onclick: move |_| { file_menu_open.set(!file_menu_open()); settings_menu_open.set(false); },
                        "File ▾"
                    }
                    if file_menu_open() {
                        div {
                            class: "menu-popup",
                            button {
                                class: "menu-item",
                                onclick: move |_| {
                                    file_menu_open.set(false);
                                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                        root_directory.set(path);
                                        selected_file.set(None);
                                        do_save_state();
                                    }
                                },
                                "📁 Open Directory..."
                            }
                            div { class: "menu-separator" }
                            button {
                                class: "menu-item",
                                onclick: move |_| { do_save_state(); std::process::exit(0); },
                                "⏻ Quit"
                            }
                        }
                    }
                }

                div {
                    class: "menu",
                    button {
                        onclick: move |_| { settings_menu_open.set(!settings_menu_open()); file_menu_open.set(false); },
                        "Settings ▾"
                    }
                    if settings_menu_open() {
                        div {
                            class: "menu-popup",
                            button {
                                class: "menu-item",
                                onclick: move |_| {
                                    let v = !telemetry_enabled();
                                    telemetry_enabled.set(v);
                                    #[cfg(not(target_arch = "wasm32"))]
                                    { let _ = httprunner_core::telemetry::set_enabled(v); }
                                    do_save_state();
                                },
                                if telemetry_enabled() { "✓ Telemetry Enabled" } else { "  Telemetry Disabled" }
                            }
                            div { class: "menu-separator" }
                            div {
                                style: "padding: 6px 12px;",
                                p { style: "font-size: 12px; color: #8087a2; margin-bottom: 4px;", "Request Delay: {delay_ms()} ms" }
                                input {
                                    r#type: "range", min: "0", max: "10000", step: "100",
                                    value: "{delay_ms()}",
                                    oninput: move |e| { if let Ok(v) = e.value().parse::<u64>() { delay_ms.set(v); do_save_state(); } },
                                }
                            }
                        }
                    }
                }

                div { style: "flex: 1;" }

                span { style: "color: #8087a2; font-size: 12px;", "Env:" }
                select {
                    value: "{env_val}",
                    onchange: move |e| {
                        let v = e.value();
                        selected_environment.set(if v == "__none__" { None } else { Some(v) });
                        do_save_state();
                    },
                    option { value: "__none__", "None" }
                    for env in environments().iter() {
                        option { key: "{env}", value: "{env}", "{env}" }
                    }
                }

                button { style: "padding: 2px 6px;", onclick: move |_| font_size.set((font_size() - 1.0).max(8.0)), "A-" }
                span { style: "font-size: 11px; color: #8087a2;", "{font_size() as u32}px" }
                button { style: "padding: 2px 6px;", onclick: move |_| font_size.set((font_size() + 1.0).min(32.0)), "A+" }
            }

            // CONTENT AREA
            div {
                class: "content-area",

                #[cfg(not(target_arch = "wasm32"))]
                if file_tree_visible() {
                    div {
                        class: "file-tree",
                        FileTree {
                            root_path: root_directory,
                            selected_file,
                            on_file_selected: handle_file_selected,
                        }
                    }
                }

                div {
                    class: "main-panel",

                    div {
                        class: "view-tabs",
                        button {
                            class: if view_mode() == ViewMode::TextEditor { "active" } else { "" },
                            onclick: move |_| view_mode.set(ViewMode::TextEditor),
                            "📝 Text Editor"
                        }
                        button {
                            class: if view_mode() == ViewMode::RequestDetails { "active" } else { "" },
                            onclick: move |_| view_mode.set(ViewMode::RequestDetails),
                            "📋 Request Details"
                        }
                        button {
                            class: if view_mode() == ViewMode::EnvironmentEditor { "active" } else { "" },
                            onclick: move |_| view_mode.set(ViewMode::EnvironmentEditor),
                            "🌍 Environment"
                        }
                        span { style: "margin-left: auto; font-size: 11px; color: #8087a2;",
                            "Ctrl+T=cycle Ctrl+S=save"
                            #[cfg(not(target_arch = "wasm32"))]
                            " Ctrl+B=tree"
                        }
                    }

                    // Editor section
                    div {
                        class: "editor-section",
                        style: "height: {ratio_pct}; font-size: {font_size()}px;",

                        match view_mode() {
                            ViewMode::TextEditor => rsx! {
                                TextEditor {
                                    file: selected_file,
                                    content: text_content,
                                    has_changes: text_has_changes,
                                    on_save: move |_| do_save_text(),
                                }
                            },
                            ViewMode::RequestDetails => rsx! {
                                RequestView {
                                    state: req_view_state,
                                    file: selected_file,
                                    on_action: handle_req_action,
                                }
                            },
                            ViewMode::EnvironmentEditor => rsx! {
                                EnvironmentEditor { state: env_editor_state }
                            },
                        }
                    }

                    // Run all bar
                    if view_mode() != ViewMode::EnvironmentEditor {
                        div {
                            style: "padding: 4px 8px; display: flex; align-items: center; gap: 8px; background: #1e2030; border-top: 1px solid #363a4f; flex-shrink: 0;",
                            button {
                                disabled: !run_enabled,
                                onclick: move |_| { if run_enabled { do_run_all(); } },
                                "▶ Run All (F5 / Ctrl+R)"
                            }
                            if (view_mode() == ViewMode::TextEditor && text_has_changes()) ||
                               (view_mode() == ViewMode::RequestDetails && req_view_state().has_changes()) {
                                span { class: "warning", "● Unsaved changes" }
                            }
                        }
                    }

                    // Splitter
                    div {
                        class: "splitter",
                        onmousedown: move |e| {
                            is_dragging.set(true);
                            drag_last_y.set(e.client_coordinates().y);
                            e.stop_propagation();
                        },
                    }

                    // Results
                    div {
                        class: "results-section",
                        style: "font-size: {font_size()}px;",
                        ResultsView { results, is_running, compact_mode: results_compact_mode }
                    }
                }
            }

            // BOTTOM BAR
            div {
                class: "bottom-bar",
                #[cfg(not(target_arch = "wasm32"))]
                span { "📁 {root_directory().display()}" }
                if let Some(ref f) = selected_file() {
                    span { "| 📄 {f.file_name().and_then(|n| n.to_str()).unwrap_or(\"\")}" }
                }
                if let Ok(sk) = httprunner_core::logging::get_support_key() {
                    span { "| 🔑 {sk.short_key}" }
                }
            }
        }
    }
}


fn load_file_state(
    path: &PathBuf,
    req_view_state: &mut Signal<RequestViewState>,
    env_editor_state: &mut Signal<EnvEditorState>,
    environments: &mut Signal<Vec<String>>,
) {
    req_view_state.write().load_file(path);
    env_editor_state.write().load_for_file(path);

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(file_str) = path.to_str()
            && let Ok(Some(env_file)) = httprunner_core::environment::find_environment_file(file_str)
            && let Ok(env_config) = httprunner_core::environment::parse_environment_file(&env_file)
        {
            let mut names: Vec<String> = env_config.keys().cloned().collect();
            names.sort();
            environments.set(names);
        } else {
            environments.set(Vec::new());
        }
    }

    #[cfg(target_arch = "wasm32")]
    environments.set(env_editor_state.read().environment_names());
}
