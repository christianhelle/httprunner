#[cfg(target_arch = "wasm32")]
use crate::results_view_async;
use crate::{
    environment_editor::EnvironmentEditor,
    file_tree,
    request_editor::RequestEditor,
    results_view::{self, ExecutionResult},
    state::AppState,
    text_editor::TextEditor,
};
use dioxus::prelude::keyboard_types::{Code, Key};
use dioxus::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use dioxus_desktop::{DesktopContext, use_window};
#[cfg(not(target_arch = "wasm32"))]
use httprunner_core::telemetry;
use httprunner_core::types::AssertionResult;
#[cfg(target_arch = "wasm32")]
use httprunner_core::types::Variable;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc::UnboundedSender;

const APP_STYLE: &str = include_str!("../assets/app.css");
const NEW_FILE_TEMPLATE: &str = "### New Request\nGET https://httpbin.org/get\n";

#[derive(Debug)]
pub enum AppEvent {
    DiscoveryStarted,
    FileDiscovered(PathBuf),
    DiscoveryFinished,
    ExecutionStarted { message: String },
    ExecutionCleared,
    ExecutionPush(ExecutionResult),
    ExecutionReplace(Vec<ExecutionResult>),
    ExecutionFinished,
    WindowSizeChanged((f32, f32)),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    TextEditor,
    RequestDetails,
    EnvironmentEditor,
}

pub fn app() -> Element {
    let mut app_state = use_signal(AppModel::load);
    let sender = use_hook({
        let mut app_state = app_state;
        move || {
            let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<AppEvent>();
            spawn(async move {
                while let Some(event) = receiver.recv().await {
                    app_state.with_mut(|model| model.apply_event(event));
                }
            });
            sender
        }
    });

    use_hook({
        let sender = sender.clone();
        let root_directory = app_state.peek().root_directory.clone();
        move || {
            file_tree::start_discovery(root_directory, sender);
        }
    });

    #[cfg(not(target_arch = "wasm32"))]
    let desktop_window = use_window();

    #[cfg(not(target_arch = "wasm32"))]
    use_hook({
        let sender = sender.clone();
        let window = desktop_window.window.clone();
        move || {
            std::thread::spawn(move || {
                let mut last_size = None;
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    let physical_size = window.inner_size();
                    let logical_size = physical_size.to_logical::<f64>(window.scale_factor());
                    let current_size = (logical_size.width as f32, logical_size.height as f32);
                    if last_size != Some(current_size) {
                        let _ = sender.send(AppEvent::WindowSizeChanged(current_size));
                        last_size = Some(current_size);
                    }
                }
            });
        }
    });

    let font_size = app_state.read().font_size;
    let editor_panel_ratio = app_state.read().editor_panel_ratio.clamp(0.1, 0.9);
    let dragging_splitter = app_state.read().dragging_splitter;
    let content_class = content_class(app_state.peek().file_tree_visible);

    #[cfg(not(target_arch = "wasm32"))]
    let top_bar = render_top_bar(app_state, sender.clone(), desktop_window.clone());
    #[cfg(target_arch = "wasm32")]
    let top_bar = render_top_bar(app_state, sender.clone());

    let editor_rows = 100.0 * editor_panel_ratio;
    let results_rows = 100.0 * (1.0 - editor_panel_ratio);
    let file_tree_panel = render_optional_file_tree_panel(app_state);

    rsx! {
        style { "{APP_STYLE}" }
        div {
            class: "app-shell",
            style: format!("--app-font-size: {}px;", font_size),
            tabindex: 0,
            autofocus: true,
            onkeydown: move |event| {
                #[cfg(not(target_arch = "wasm32"))]
                handle_keyboard_shortcut(event, app_state, sender.clone(), desktop_window.clone());
                #[cfg(target_arch = "wasm32")]
                handle_keyboard_shortcut(event, app_state, sender.clone());
            },
            {top_bar}
            div {
                class: content_class,
                onresize: move |event| {
                    if let Ok(size) = event.data().get_content_box_size() {
                        app_state.with_mut(|model| {
                            model.content_height = size.height;
                        });
                    }
                },
                {file_tree_panel}
                div {
                    class: "main-column",
                    style: format!("grid-template-rows: {:.3}% 6px {:.3}%;", editor_rows, results_rows),
                    {render_editor_panel(app_state, sender.clone())}
                    div {
                        class: "splitter",
                        onmousedown: move |_| {
                            app_state.with_mut(|model| {
                                model.dragging_splitter = true;
                            });
                        }
                    }
                    {render_results_panel(app_state)}
                }
                if dragging_splitter {
                    div {
                        class: "splitter-overlay",
                        onmousemove: move |event: MouseEvent| {
                            let content_height = app_state.peek().content_height;
                            if content_height > 0.0 {
                                let y = event.data().element_coordinates().y.clamp(0.0, content_height);
                                app_state.with_mut(|model| {
                                    model.editor_panel_ratio = (y / content_height).clamp(0.1, 0.9) as f32;
                                });
                            }
                        },
                        onmouseup: move |_| {
                            app_state.with_mut(|model| {
                                model.dragging_splitter = false;
                                model.persist();
                            });
                        },
                        onmouseleave: move |_| {
                            app_state.with_mut(|model| {
                                model.dragging_splitter = false;
                                model.persist();
                            });
                        },
                    }
                }
            }
            {render_status_bar(app_state)}
        }
    }
}

struct AppModel {
    root_directory: PathBuf,
    selected_file: Option<PathBuf>,
    request_editor: RequestEditor,
    text_editor: TextEditor,
    environment_editor: EnvironmentEditor,
    results: Vec<ExecutionResult>,
    results_compact_mode: bool,
    status_message: Option<String>,
    http_files: Vec<PathBuf>,
    expanded_dirs: HashSet<PathBuf>,
    is_discovering: bool,
    discovered_count: usize,
    font_size: f32,
    environment_selector_open: bool,
    last_saved_window_size: Option<(f32, f32)>,
    view_mode: ViewMode,
    file_tree_visible: bool,
    telemetry_enabled: bool,
    delay_ms: u64,
    editor_panel_ratio: f32,
    support_key: Option<String>,
    environments: Vec<String>,
    selected_environment: Option<String>,
    content_height: f64,
    editor_scroll_top: f64,
    editor_scroll_left: f64,
    dragging_splitter: bool,
    is_running: bool,
}

impl AppModel {
    const DEFAULT_FONT_SIZE: f32 = 14.0;
    const MIN_FONT_SIZE: f32 = 8.0;
    const MAX_FONT_SIZE: f32 = 32.0;
    const FONT_SIZE_STEP: f32 = 1.0;

    fn load() -> Self {
        let state = AppState::load();
        let root_directory = state
            .root_directory
            .and_then(|path| if path.exists() { Some(path) } else { None })
            .unwrap_or_else(default_root_directory);

        let mut model = Self {
            root_directory,
            selected_file: None,
            request_editor: RequestEditor::new(),
            text_editor: TextEditor::new(),
            environment_editor: EnvironmentEditor::new(),
            results: state.last_results.unwrap_or_default(),
            results_compact_mode: state.results_compact_mode.unwrap_or(true),
            status_message: None,
            http_files: Vec::new(),
            expanded_dirs: HashSet::new(),
            is_discovering: false,
            discovered_count: 0,
            font_size: state.font_size.unwrap_or(Self::DEFAULT_FONT_SIZE),
            environment_selector_open: false,
            last_saved_window_size: state.window_size,
            view_mode: ViewMode::TextEditor,
            file_tree_visible: state.file_tree_visible.unwrap_or(true),
            telemetry_enabled: state.telemetry_enabled.unwrap_or(true),
            delay_ms: state.delay_ms.unwrap_or(0),
            editor_panel_ratio: state.editor_panel_ratio.unwrap_or(0.5),
            support_key: httprunner_core::logging::get_support_key()
                .ok()
                .map(|support_key| support_key.short_key),
            environments: Vec::new(),
            selected_environment: None,
            content_height: 600.0,
            editor_scroll_top: 0.0,
            editor_scroll_left: 0.0,
            dragging_splitter: false,
            is_running: false,
        };

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(saved_file) = state.selected_file
            && saved_file.exists()
        {
            model.select_file(saved_file.clone());
            if let Some(saved_environment) = state.selected_environment
                && model.environments.contains(&saved_environment)
            {
                model.selected_environment = Some(saved_environment);
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            model.text_editor.load_file(Path::new("."));
            model.environment_editor.load_for_file(Path::new("."));
            model.environments = model.environment_editor.environment_names();
            if let Some(saved_environment) = state.selected_environment
                && model.environments.contains(&saved_environment)
            {
                model.selected_environment = Some(saved_environment);
            }
            if !model.text_editor.get_content().trim().is_empty() {
                model
                    .request_editor
                    .load_content(model.text_editor.get_content());
            }
        }

        model
    }

    fn apply_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::DiscoveryStarted => {
                self.http_files.clear();
                self.is_discovering = true;
                self.discovered_count = 0;
            }
            AppEvent::FileDiscovered(path) => {
                if !self.http_files.contains(&path) {
                    self.http_files.push(path);
                    self.http_files.sort();
                    self.discovered_count = self.http_files.len();
                }
            }
            AppEvent::DiscoveryFinished => {
                self.http_files.sort();
                self.is_discovering = false;
                self.discovered_count = self.http_files.len();
            }
            AppEvent::ExecutionStarted { message } => {
                self.is_running = true;
                self.status_message = Some(message);
                self.results.clear();
            }
            AppEvent::ExecutionCleared => {
                self.results.clear();
            }
            AppEvent::ExecutionPush(result) => {
                self.results.push(result);
                self.status_message = None;
            }
            AppEvent::ExecutionReplace(results) => {
                self.results = results;
                self.status_message = None;
            }
            AppEvent::ExecutionFinished => {
                self.is_running = false;
                self.status_message = None;
                self.persist();
            }
            AppEvent::WindowSizeChanged(size) => {
                if self.last_saved_window_size != Some(size) {
                    self.last_saved_window_size = Some(size);
                    self.persist();
                }
            }
        }
    }

    fn persist(&self) {
        let state = AppState {
            root_directory: Some(self.root_directory.clone()),
            selected_file: self.selected_file.clone(),
            selected_environment: self.selected_environment.clone(),
            font_size: Some(self.font_size),
            window_size: self.last_saved_window_size,
            last_results: Some(self.results.clone()),
            file_tree_visible: Some(self.file_tree_visible),
            results_compact_mode: Some(self.results_compact_mode),
            telemetry_enabled: Some(self.telemetry_enabled),
            delay_ms: Some(self.delay_ms),
            editor_panel_ratio: Some(self.editor_panel_ratio),
        };

        if let Err(error) = state.save() {
            eprintln!("Failed to save application state: {}", error);
        }
    }

    fn select_file(&mut self, file: PathBuf) {
        self.selected_file = Some(file.clone());
        self.text_editor.load_file(&file);
        self.request_editor.load_file(&file);
        self.load_environments(&file);
        self.editor_scroll_top = 0.0;
        self.editor_scroll_left = 0.0;
        if let Some(parent) = file.parent() {
            self.expanded_dirs.insert(parent.to_path_buf());
        }
        self.persist();
    }

    fn load_environments(&mut self, file: &Path) {
        self.environment_editor.load_for_file(file);
        self.environments = self.environment_editor.environment_names();
        if let Some(selected_environment) = &self.selected_environment
            && !self.environments.contains(selected_environment)
        {
            self.selected_environment = None;
        }
    }

    fn refresh_environment_names(&mut self) {
        self.environments = self.environment_editor.environment_names();
        if self.environment_editor.editing_environment.is_none() {
            self.environment_editor.editing_environment = self.environments.first().cloned();
        }
        if let Some(editing_environment) = self.environment_editor.editing_environment.clone()
            && !self.environments.contains(&editing_environment)
        {
            self.environment_editor.editing_environment = self.environments.first().cloned();
        }
        if let Some(selected_environment) = self.selected_environment.clone()
            && !self.environments.contains(&selected_environment)
        {
            self.selected_environment = None;
            self.persist();
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn sync_request_editor_from_text(&mut self) {
        self.request_editor
            .load_content(self.text_editor.get_content());
    }

    fn save_request_editor_changes(&mut self) -> anyhow::Result<()> {
        if self.request_editor.current_file().is_some() {
            self.request_editor.save_to_file()?;
            if let Some(file) = &self.selected_file {
                self.text_editor.load_file(file);
            }
        } else {
            let serialized = httprunner_core::serializer::serialize_http_requests(
                self.request_editor.get_requests(),
            );
            self.text_editor.replace_content(serialized);
            self.text_editor.save_to_file()?;
            self.request_editor
                .load_content(self.text_editor.get_content());
        }

        self.request_editor.mark_saved();
        Ok(())
    }

    fn cycle_environment(&mut self) {
        if self.environment_selector_open || self.environments.is_empty() {
            return;
        }

        if let Some(selected_environment) = self.selected_environment.clone() {
            if let Some(index) = self
                .environments
                .iter()
                .position(|environment| environment == &selected_environment)
            {
                let next_index = index + 1;
                self.selected_environment = if next_index >= self.environments.len() {
                    None
                } else {
                    Some(self.environments[next_index].clone())
                };
            } else {
                self.selected_environment = self.environments.first().cloned();
            }
        } else {
            self.selected_environment = self.environments.first().cloned();
        }

        self.persist();
    }

    fn cycle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::TextEditor => ViewMode::RequestDetails,
            ViewMode::RequestDetails => ViewMode::EnvironmentEditor,
            ViewMode::EnvironmentEditor => ViewMode::TextEditor,
        };

        #[cfg(target_arch = "wasm32")]
        if matches!(self.view_mode, ViewMode::RequestDetails) {
            self.sync_request_editor_from_text();
        }
    }

    fn change_font_size(&mut self, delta: f32) {
        self.font_size = (self.font_size + delta).clamp(Self::MIN_FONT_SIZE, Self::MAX_FONT_SIZE);
        self.persist();
    }

    fn reset_font_size(&mut self) {
        self.font_size = Self::DEFAULT_FONT_SIZE;
        self.persist();
    }

    fn start_run_all(&self, sender: UnboundedSender<AppEvent>) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(file) = &self.selected_file {
            results_view::start_run_file(
                file,
                self.selected_environment.as_deref(),
                self.delay_ms,
                sender,
            );
        }

        #[cfg(target_arch = "wasm32")]
        results_view_async::start_run_content(
            self.text_editor.get_content().to_string(),
            self.selected_environment_variables(),
            self.delay_ms,
            sender,
        );
    }

    fn start_run_single_request(&self, index: usize, sender: UnboundedSender<AppEvent>) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(file) = &self.selected_file {
            results_view::start_run_single_request(
                file,
                index,
                self.selected_environment.as_deref(),
                self.delay_ms,
                sender,
            );
        }

        #[cfg(target_arch = "wasm32")]
        results_view_async::start_run_single_request_content(
            self.text_editor.get_content().to_string(),
            index,
            self.selected_environment_variables(),
            self.delay_ms,
            sender,
        );
    }

    #[cfg(target_arch = "wasm32")]
    fn selected_environment_variables(&self) -> Vec<Variable> {
        self.selected_environment
            .as_ref()
            .and_then(|environment| self.environment_editor.config.get(environment))
            .map(|variables| {
                variables
                    .iter()
                    .map(|(name, value)| Variable {
                        name: name.clone(),
                        value: value.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn can_run_from_text_editor(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.selected_file.is_some() && !self.text_editor.has_changes()
        }

        #[cfg(target_arch = "wasm32")]
        {
            !self.text_editor.get_content().trim().is_empty()
        }
    }

    fn can_run_from_request_details(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.selected_file.is_some() && !self.request_editor.has_changes()
        }

        #[cfg(target_arch = "wasm32")]
        {
            !self.request_editor.get_requests().is_empty() && !self.request_editor.has_changes()
        }
    }
}

fn default_root_directory() -> PathBuf {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }

    #[cfg(target_arch = "wasm32")]
    {
        PathBuf::from(".")
    }
}

fn content_class(file_tree_visible: bool) -> &'static str {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if file_tree_visible {
            "content with-tree"
        } else {
            "content without-tree"
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = file_tree_visible;
        "content without-tree"
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn render_top_bar(
    mut app_state: Signal<AppModel>,
    sender: UnboundedSender<AppEvent>,
    desktop_window: DesktopContext,
) -> Element {
    let environments = app_state.read().environments.clone();
    let selected_environment = app_state.read().selected_environment.clone();
    let telemetry_enabled = app_state.read().telemetry_enabled;
    let delay_ms = app_state.read().delay_ms;
    let open_directory_sender = sender.clone();
    let new_file_sender = sender.clone();

    rsx! {
        div {
            class: "topbar",
            div {
                class: "toolbar-row",
                details {
                    class: "dropdown",
                    summary { class: "dropdown-trigger btn", "File" }
                    div {
                        class: "dropdown-menu",
                        button {
                            class: "btn dropdown-item",
                            onclick: move |_| open_directory_dialog(app_state, open_directory_sender.clone()),
                            "Open Directory..."
                        }
                        button {
                            class: "btn dropdown-item",
                            onclick: move |_| create_new_http_file(app_state, new_file_sender.clone()),
                            "New .http File..."
                        }
                        button {
                            class: "btn dropdown-item",
                            onclick: move |_| quit_application(app_state, desktop_window.clone()),
                            "Quit"
                        }
                    }
                }
                details {
                    class: "dropdown",
                    summary { class: "dropdown-trigger btn", "Settings" }
                    div {
                        class: "dropdown-menu settings-menu",
                        button {
                            class: if telemetry_enabled { "btn dropdown-item btn-toggle is-active" } else { "btn dropdown-item btn-toggle" },
                            onclick: move |_| {
                                app_state.with_mut(|model| {
                                    model.telemetry_enabled = !model.telemetry_enabled;
                                    if let Err(error) = telemetry::set_enabled(model.telemetry_enabled) {
                                        eprintln!("Failed to save telemetry setting: {}", error);
                                    }
                                    model.persist();
                                });
                            },
                            if telemetry_enabled { "✓ Telemetry Enabled" } else { "Telemetry Disabled" }
                        }
                        label {
                            class: "settings-label",
                            "Request Delay (ms):"
                        }
                        input {
                            class: "range-input",
                            r#type: "range",
                            min: "0",
                            max: "10000",
                            step: "100",
                            value: format!("{}", delay_ms),
                            oninput: move |event| {
                                if let Ok(value) = event.value().parse::<u64>() {
                                    app_state.with_mut(|model| {
                                        model.delay_ms = value;
                                        model.persist();
                                    });
                                }
                            }
                        }
                        span { class: "small-hint", "{delay_ms} ms" }
                        p {
                            class: "small-hint",
                            "Telemetry helps improve the app. No personal data is collected."
                        }
                    }
                }
                div {
                    class: "environment-selector",
                    label { class: "field-label", "Environment:" }
                    select {
                        class: "select-input",
                        value: selected_environment.clone().unwrap_or_default(),
                        onfocus: move |_| {
                            app_state.with_mut(|model| model.environment_selector_open = true);
                        },
                        onblur: move |_| {
                            app_state.with_mut(|model| model.environment_selector_open = false);
                        },
                        onchange: move |event| {
                            let value = event.value();
                            app_state.with_mut(|model| {
                                model.selected_environment = if value.is_empty() {
                                    None
                                } else {
                                    Some(value)
                                };
                                model.persist();
                            });
                        },
                        option { value: "", "None" }
                        for environment in environments {
                            option { value: environment.clone(), "{environment}" }
                        }
                    }
                }
                div { class: "toolbar-spacer" }
                span {
                    class: "small-hint",
                    "F5 / Ctrl+R run all | Ctrl+O open folder | Ctrl+T switch view | Ctrl+B toggle files"
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn render_top_bar(mut app_state: Signal<AppModel>, sender: UnboundedSender<AppEvent>) -> Element {
    let environments = app_state.read().environments.clone();
    let selected_environment = app_state.read().selected_environment.clone();
    let delay_ms = app_state.read().delay_ms;

    rsx! {
        div {
            class: "topbar",
            div {
                class: "toolbar-row",
                details {
                    class: "dropdown",
                    summary { class: "dropdown-trigger btn", "Settings" }
                    div {
                        class: "dropdown-menu settings-menu",
                        label {
                            class: "settings-label",
                            "Request Delay (ms):"
                        }
                        input {
                            class: "range-input",
                            r#type: "range",
                            min: "0",
                            max: "10000",
                            step: "100",
                            value: format!("{}", delay_ms),
                            oninput: move |event| {
                                if let Ok(value) = event.value().parse::<u64>() {
                                    app_state.with_mut(|model| {
                                        model.delay_ms = value;
                                        model.persist();
                                    });
                                }
                            }
                        }
                        span { class: "small-hint", "{delay_ms} ms" }
                    }
                }
                div {
                    class: "environment-selector",
                    label { class: "field-label", "Environment:" }
                    select {
                        class: "select-input",
                        value: selected_environment.clone().unwrap_or_default(),
                        onfocus: move |_| {
                            app_state.with_mut(|model| model.environment_selector_open = true);
                        },
                        onblur: move |_| {
                            app_state.with_mut(|model| model.environment_selector_open = false);
                        },
                        onchange: move |event| {
                            let value = event.value();
                            app_state.with_mut(|model| {
                                model.selected_environment = if value.is_empty() {
                                    None
                                } else {
                                    Some(value)
                                };
                                model.persist();
                            });
                        },
                        option { value: "", "None" }
                        for environment in environments {
                            option { value: environment.clone(), "{environment}" }
                        }
                    }
                }
                button {
                    class: "btn btn-primary",
                    disabled: !app_state.read().can_run_from_text_editor(),
                    onclick: move |_| {
                        let model = app_state.peek();
                        model.start_run_all(sender.clone());
                    },
                    "▶ Run All Requests"
                }
                div { class: "toolbar-spacer" }
                span {
                    class: "small-hint",
                    "Ctrl+R run all | Ctrl+T switch view | Ctrl+S save"
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn render_optional_file_tree_panel(app_state: Signal<AppModel>) -> Element {
    if app_state.read().file_tree_visible {
        rsx! { {render_file_tree_panel(app_state)} }
    } else {
        rsx! {}
    }
}

#[cfg(target_arch = "wasm32")]
fn render_optional_file_tree_panel(_app_state: Signal<AppModel>) -> Element {
    rsx! {}
}

fn render_file_tree_panel(app_state: Signal<AppModel>) -> Element {
    let root_directory = app_state.read().root_directory.clone();
    let files = app_state.read().http_files.clone();
    let grouped_files = file_tree::group_files_by_directory(&root_directory, &files);
    let selected_file = app_state.read().selected_file.clone();
    let is_discovering = app_state.read().is_discovering;
    let discovered_count = app_state.read().discovered_count;

    rsx! {
        div {
            class: "file-tree-panel",
            div {
                class: "card-header",
                h2 { class: "card-title", "HTTP Files" }
            }
            div {
                class: "card-content file-tree-content",
                if is_discovering {
                    div {
                        class: "status-chip info",
                        "Discovering .http files... ({discovered_count})"
                    }
                }
                if files.is_empty() && !is_discovering {
                    div {
                        class: "empty-state",
                        p { "No .http files found in this directory." }
                        p { class: "small-hint", "Use File → Open Directory to choose a different folder." }
                    }
                } else {
                    for (directory, directory_files) in grouped_files {
                        match directory {
                            Some(directory_path) => render_directory_group(
                                app_state,
                                directory_path,
                                directory_files,
                                selected_file.clone(),
                            ),
                            None => rsx! {
                                div {
                                    class: "file-group-root",
                                    for file in directory_files {
                                        {render_file_entry(app_state, file, selected_file.clone())}
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

fn render_directory_group(
    mut app_state: Signal<AppModel>,
    directory_path: PathBuf,
    files: Vec<PathBuf>,
    selected_file: Option<PathBuf>,
) -> Element {
    let root_directory = app_state.read().root_directory.clone();
    let is_expanded = app_state.read().expanded_dirs.contains(&directory_path);
    let directory_name = file_tree::relative_directory_name(&root_directory, &directory_path);

    rsx! {
        div {
            class: "directory-group",
            button {
                class: "directory-toggle btn btn-ghost",
                onclick: move |_| {
                    app_state.with_mut(|model| {
                        if model.expanded_dirs.contains(&directory_path) {
                            model.expanded_dirs.remove(&directory_path);
                        } else {
                            model.expanded_dirs.insert(directory_path.clone());
                        }
                    });
                },
                if is_expanded { "📂 {directory_name}" } else { "📁 {directory_name}" }
            }
            if is_expanded {
                div {
                    class: "directory-files",
                    for file in files {
                        {render_file_entry(app_state, file, selected_file.clone())}
                    }
                }
            }
        }
    }
}

fn render_file_entry(
    mut app_state: Signal<AppModel>,
    file: PathBuf,
    selected_file: Option<PathBuf>,
) -> Element {
    let is_selected = selected_file.as_ref() == Some(&file);
    let file_name = file
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();

    rsx! {
        button {
            class: if is_selected { "file-entry btn is-active" } else { "file-entry btn btn-ghost" },
            onclick: move |_| {
                app_state.with_mut(|model| {
                    model.select_file(file.clone());
                });
            },
            "📄 {file_name}"
        }
    }
}

fn render_editor_panel(
    mut app_state: Signal<AppModel>,
    sender: UnboundedSender<AppEvent>,
) -> Element {
    let view_mode = app_state.read().view_mode;
    let shortcut_hint = editor_shortcut_hint();

    rsx! {
        div {
            class: "editor-card",
            div {
                class: "card-header editor-header",
                div {
                    class: "tab-bar",
                    button {
                        class: if view_mode == ViewMode::TextEditor { "btn is-active" } else { "btn" },
                        onclick: move |_| {
                            app_state.with_mut(|model| {
                                model.view_mode = ViewMode::TextEditor;
                            });
                        },
                        "📝 Text Editor"
                    }
                    button {
                        class: if view_mode == ViewMode::RequestDetails { "btn is-active" } else { "btn" },
                        onclick: move |_| {
                            app_state.with_mut(|model| {
                                model.view_mode = ViewMode::RequestDetails;
                                #[cfg(target_arch = "wasm32")]
                                model.sync_request_editor_from_text();
                            });
                        },
                        "📋 Request Details"
                    }
                    button {
                        class: if view_mode == ViewMode::EnvironmentEditor { "btn is-active" } else { "btn" },
                        onclick: move |_| {
                            app_state.with_mut(|model| {
                                model.view_mode = ViewMode::EnvironmentEditor;
                            });
                        },
                        "🌍 Environment"
                    }
                }
                div { class: "toolbar-spacer" }
                span { class: "small-hint", "{shortcut_hint}" }
            }
            div {
                class: "card-content editor-content",
                match view_mode {
                    ViewMode::TextEditor => render_text_editor_pane(app_state, sender),
                    ViewMode::RequestDetails => render_request_details_pane(app_state, sender),
                    ViewMode::EnvironmentEditor => render_environment_editor_pane(app_state),
                }
            }
        }
    }
}

fn render_text_editor_pane(
    mut app_state: Signal<AppModel>,
    sender: UnboundedSender<AppEvent>,
) -> Element {
    let selected_file = app_state.read().selected_file.clone();
    let content = app_state.read().text_editor.get_content().to_string();
    let has_changes = app_state.read().text_editor.has_changes();
    let can_run = app_state.read().can_run_from_text_editor();
    let editor_body = render_text_editor_body(app_state, selected_file, content);

    rsx! {
        div {
            class: "pane-layout",
            {editor_body}
            div {
                class: "pane-footer",
                button {
                    class: "btn",
                    onclick: move |_| {
                        app_state.with_mut(|model| {
                            if let Err(error) = model.text_editor.save_to_file() {
                                eprintln!("Failed to save file: {}", error);
                            } else {
                                #[cfg(target_arch = "wasm32")]
                                model.sync_request_editor_from_text();
                                model.persist();
                            }
                        });
                    },
                    "💾 Save"
                }
                button {
                    class: "btn btn-primary",
                    disabled: !can_run,
                    onclick: move |_| {
                        let model = app_state.peek();
                        model.start_run_all(sender.clone());
                    },
                    "▶ Run All Requests"
                }
                if has_changes {
                    span { class: "status-chip warning", "● Unsaved changes" }
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn render_text_editor_body(
    app_state: Signal<AppModel>,
    selected_file: Option<PathBuf>,
    content: String,
) -> Element {
    if selected_file.is_none() {
        rsx! {
            div {
                class: "empty-state",
                p { "No file selected. Select a .http file from the left panel." }
            }
        }
    } else {
        render_text_editor_input(app_state, content)
    }
}

#[cfg(target_arch = "wasm32")]
fn render_text_editor_body(
    app_state: Signal<AppModel>,
    selected_file: Option<PathBuf>,
    content: String,
) -> Element {
    if selected_file.is_none() && content.trim().is_empty() {
        let editor = render_text_editor_input(app_state, content);
        rsx! {
            div {
                class: "empty-state",
                h3 { "✏️ Paste your HTTP requests here" }
                p { "You can paste the contents of an .http file below and run them directly in the browser." }
                pre { class: "code-block", "GET https://httpbin.org/get\nAccept: application/json" }
            }
            {editor}
        }
    } else {
        render_text_editor_input(app_state, content)
    }
}

fn render_text_editor_input(mut app_state: Signal<AppModel>, content: String) -> Element {
    let highlighted_html = app_state.read().text_editor.highlighted_html();
    let line_numbers = app_state.read().text_editor.line_numbers();
    let scroll_top = app_state.read().editor_scroll_top;
    let scroll_left = app_state.read().editor_scroll_left;

    rsx! {
        div {
            class: "editor-surface",
            div {
                class: "editor-gutter",
                pre {
                    class: "editor-line-numbers",
                    style: format!("transform: translateY({:.1}px);", -scroll_top),
                    "{line_numbers}"
                }
            }
            div {
                class: "editor-stack",
                pre {
                    class: "editor-highlight-layer",
                    style: format!("transform: translate({:.1}px, {:.1}px);", -scroll_left, -scroll_top),
                    dangerous_inner_html: highlighted_html
                }
                textarea {
                    class: "text-editor code-editor-textarea",
                    spellcheck: false,
                    wrap: "off",
                    value: content,
                    oninput: move |event| {
                        let value = event.value();
                        app_state.with_mut(|model| {
                            model.text_editor.set_content(value.clone());
                            #[cfg(target_arch = "wasm32")]
                            model.sync_request_editor_from_text();
                        });
                    },
                    onscroll: move |event| {
                        app_state.with_mut(|model| {
                            model.editor_scroll_top = event.data().scroll_top();
                            model.editor_scroll_left = event.data().scroll_left();
                        });
                    },
                }
            }
        }
    }
}

fn render_request_details_pane(
    app_state: Signal<AppModel>,
    sender: UnboundedSender<AppEvent>,
) -> Element {
    let selected_file = app_state.read().selected_file.clone();
    let is_editing = app_state.read().request_editor.is_editing();
    let requests = app_state.read().request_editor.get_requests().to_vec();
    let has_changes = app_state.read().request_editor.has_changes();
    let can_run = app_state.read().can_run_from_request_details();
    let request_details_body = render_request_details_body(
        app_state,
        sender,
        selected_file,
        is_editing,
        requests,
        has_changes,
        can_run,
    );

    rsx! {
        div {
            class: "pane-layout",
            {request_details_body}
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn render_request_details_body(
    mut app_state: Signal<AppModel>,
    sender: UnboundedSender<AppEvent>,
    selected_file: Option<PathBuf>,
    is_editing: bool,
    requests: Vec<httprunner_core::HttpRequest>,
    has_changes: bool,
    can_run: bool,
) -> Element {
    if selected_file.is_none() {
        rsx! {
            div {
                class: "empty-state",
                p { "No file selected. Select a .http file from the left panel." }
            }
        }
    } else if is_editing {
        rsx! { {render_request_editor_form(app_state)} }
    } else if requests.is_empty() {
        rsx! {
            div {
                class: "empty-state",
                p { "No requests found in this file." }
                button {
                    class: "btn",
                    onclick: move |_| {
                        app_state.with_mut(|model| model.request_editor.start_new_request());
                    },
                    "➕ Add New Request"
                }
            }
        }
    } else {
        let mut add_request_state = app_state;
        let run_all_state = app_state;
        rsx! {
            div {
                class: "request-list",
                for (index, request) in requests.iter().enumerate() {
                    {render_request_card(app_state, sender.clone(), index, request.clone())}
                }
            }
            div {
                class: "pane-footer",
                button {
                    class: "btn",
                    onclick: move |_| {
                        add_request_state.with_mut(|model| model.request_editor.start_new_request());
                    },
                    "➕ Add New Request"
                }
                button {
                    class: "btn btn-primary",
                    disabled: !can_run,
                    onclick: move |_| {
                        let model = run_all_state.peek();
                        model.start_run_all(sender.clone());
                    },
                    "▶ Run All Requests"
                }
                if has_changes {
                    span { class: "status-chip warning", "● Unsaved changes" }
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn render_request_details_body(
    mut app_state: Signal<AppModel>,
    sender: UnboundedSender<AppEvent>,
    _selected_file: Option<PathBuf>,
    is_editing: bool,
    requests: Vec<httprunner_core::HttpRequest>,
    has_changes: bool,
    can_run: bool,
) -> Element {
    if is_editing {
        rsx! { {render_request_editor_form(app_state)} }
    } else if requests.is_empty() {
        rsx! {
            div {
                class: "empty-state",
                p { "No requests found in the current editor content." }
                button {
                    class: "btn",
                    onclick: move |_| {
                        app_state.with_mut(|model| model.request_editor.start_new_request());
                    },
                    "➕ Add New Request"
                }
            }
        }
    } else {
        let mut add_request_state = app_state;
        let run_all_state = app_state;
        rsx! {
            div {
                class: "request-list",
                for (index, request) in requests.iter().enumerate() {
                    {render_request_card(app_state, sender.clone(), index, request.clone())}
                }
            }
            div {
                class: "pane-footer",
                button {
                    class: "btn",
                    onclick: move |_| {
                        add_request_state.with_mut(|model| model.request_editor.start_new_request());
                    },
                    "➕ Add New Request"
                }
                button {
                    class: "btn btn-primary",
                    disabled: !can_run,
                    onclick: move |_| {
                        let model = run_all_state.peek();
                        model.start_run_all(sender.clone());
                    },
                    "▶ Run All Requests"
                }
                if has_changes {
                    span { class: "status-chip warning", "● Unsaved changes" }
                }
            }
        }
    }
}

fn render_request_card(
    mut app_state: Signal<AppModel>,
    sender: UnboundedSender<AppEvent>,
    index: usize,
    request: httprunner_core::HttpRequest,
) -> Element {
    let header_text = request
        .name
        .as_ref()
        .map(|name| format!("{} - {} {}", index + 1, request.method, name))
        .unwrap_or_else(|| format!("{} - {} {}", index + 1, request.method, request.url));

    rsx! {
        details {
            class: "request-card",
            summary { class: "request-summary", "{header_text}" }
            div {
                class: "request-body",
                div { class: "request-row", strong { "Method:" } code { "{request.method}" } }
                div { class: "request-row", strong { "URL:" } code { "{request.url}" } }
                if !request.headers.is_empty() {
                    div {
                        class: "request-section",
                        strong { "Headers:" }
                        div {
                            class: "stack-list",
                            for header in request.headers {
                                code { "{header.name}: {header.value}" }
                            }
                        }
                    }
                }
                if let Some(body) = request.body {
                    div {
                        class: "request-section",
                        strong { "Body:" }
                        pre { class: "code-block request-snippet", "{body}" }
                    }
                }
                div {
                    class: "request-actions",
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| {
                            let model = app_state.peek();
                            model.start_run_single_request(index, sender.clone());
                        },
                        "▶ Run"
                    }
                    button {
                        class: "btn",
                        onclick: move |_| {
                            app_state.with_mut(|model| model.request_editor.start_editing(index));
                        },
                        "✏ Edit"
                    }
                    button {
                        class: "btn btn-danger",
                        onclick: move |_| {
                            app_state.with_mut(|model| {
                                model.request_editor.delete_request(index);
                                if let Err(error) = model.save_request_editor_changes() {
                                    eprintln!("Failed to save file: {}", error);
                                } else {
                                    model.persist();
                                }
                            });
                        },
                        "🗑 Delete"
                    }
                }
            }
        }
    }
}

fn render_request_editor_form(mut app_state: Signal<AppModel>) -> Element {
    let editable = app_state
        .read()
        .request_editor
        .get_editing_request()
        .cloned()
        .unwrap_or_default();

    rsx! {
        div {
            class: "request-editor-form",
            h3 { "Edit Request" }
            label {
                class: "field-label",
                "Name (optional):"
            }
            input {
                class: "text-input",
                value: editable.name.clone(),
                oninput: move |event| {
                    let value = event.value();
                    app_state.with_mut(|model| {
                        if let Some(editing_request) = model.request_editor.get_editing_request_mut() {
                            editing_request.name = value;
                        }
                    });
                }
            }
            label { class: "field-label", "Method:" }
            select {
                class: "select-input",
                value: editable.method.clone(),
                onchange: move |event| {
                    let value = event.value();
                    app_state.with_mut(|model| {
                        if let Some(editing_request) = model.request_editor.get_editing_request_mut() {
                            editing_request.method = value;
                        }
                    });
                },
                for method in ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"] {
                    option { value: method, "{method}" }
                }
            }
            label { class: "field-label", "URL:" }
            input {
                class: "text-input monospace-input",
                value: editable.url.clone(),
                oninput: move |event| {
                    let value = event.value();
                    app_state.with_mut(|model| {
                        if let Some(editing_request) = model.request_editor.get_editing_request_mut() {
                            editing_request.url = value;
                        }
                    });
                }
            }
            div {
                class: "request-section",
                strong { "Headers:" }
                div {
                    class: "header-grid",
                    for (header_index, (name, value)) in editable.headers.iter().cloned().enumerate() {
                        div { class: "field-label", "Name" }
                        div { class: "field-label", "Value" }
                        div {}
                        input {
                            class: "text-input monospace-input",
                            value: name,
                            oninput: move |event| {
                                let value = event.value();
                                app_state.with_mut(|model| {
                                    if let Some(editing_request) = model.request_editor.get_editing_request_mut()
                                        && let Some(header) = editing_request.headers.get_mut(header_index)
                                    {
                                        header.0 = value;
                                    }
                                });
                            }
                        }
                        input {
                            class: "text-input monospace-input",
                            value: value,
                            oninput: move |event| {
                                let value = event.value();
                                app_state.with_mut(|model| {
                                    if let Some(editing_request) = model.request_editor.get_editing_request_mut()
                                        && let Some(header) = editing_request.headers.get_mut(header_index)
                                    {
                                        header.1 = value;
                                    }
                                });
                            }
                        }
                        button {
                            class: "btn btn-danger btn-icon",
                            onclick: move |_| {
                                app_state.with_mut(|model| {
                                    if let Some(editing_request) = model.request_editor.get_editing_request_mut()
                                        && header_index < editing_request.headers.len()
                                    {
                                        editing_request.headers.remove(header_index);
                                    }
                                });
                            },
                            "🗑"
                        }
                    }
                }
                button {
                    class: "btn",
                    onclick: move |_| {
                        app_state.with_mut(|model| {
                            if let Some(editing_request) = model.request_editor.get_editing_request_mut() {
                                editing_request.headers.push((String::new(), String::new()));
                            }
                        });
                    },
                    "➕ Add Header"
                }
            }
            label { class: "field-label", "Body:" }
            textarea {
                class: "text-editor request-body-editor",
                spellcheck: false,
                value: editable.body.clone(),
                oninput: move |event| {
                    let value = event.value();
                    app_state.with_mut(|model| {
                        if let Some(editing_request) = model.request_editor.get_editing_request_mut() {
                            editing_request.body = value;
                        }
                    });
                }
            }
            details {
                class: "advanced-options",
                summary { "Advanced Options" }
                div {
                    class: "advanced-grid",
                    label { class: "field-label", "Timeout (ms):" }
                    input {
                        class: "text-input monospace-input",
                        value: editable.timeout.clone(),
                        oninput: move |event| {
                            let value = event.value();
                            app_state.with_mut(|model| {
                                if let Some(editing_request) = model.request_editor.get_editing_request_mut() {
                                    editing_request.timeout = value;
                                }
                            });
                        }
                    }
                    label { class: "field-label", "Connection Timeout (ms):" }
                    input {
                        class: "text-input monospace-input",
                        value: editable.connection_timeout.clone(),
                        oninput: move |event| {
                            let value = event.value();
                            app_state.with_mut(|model| {
                                if let Some(editing_request) = model.request_editor.get_editing_request_mut() {
                                    editing_request.connection_timeout = value;
                                }
                            });
                        }
                    }
                    label { class: "field-label", "Depends On:" }
                    input {
                        class: "text-input monospace-input",
                        value: editable.depends_on.clone(),
                        oninput: move |event| {
                            let value = event.value();
                            app_state.with_mut(|model| {
                                if let Some(editing_request) = model.request_editor.get_editing_request_mut() {
                                    editing_request.depends_on = value;
                                }
                            });
                        }
                    }
                }
            }
            div {
                class: "pane-footer",
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {
                        app_state.with_mut(|model| {
                            if model.request_editor.save_current_edit() {
                                if let Err(error) = model.save_request_editor_changes() {
                                    eprintln!("Failed to save file: {}", error);
                                } else {
                                    model.persist();
                                }
                            }
                        });
                    },
                    "💾 Save"
                }
                button {
                    class: "btn",
                    onclick: move |_| {
                        app_state.with_mut(|model| model.request_editor.cancel_editing());
                    },
                    "❌ Cancel"
                }
            }
        }
    }
}

fn render_environment_editor_pane(mut app_state: Signal<AppModel>) -> Element {
    let env_file_path = app_state.read().environment_editor.env_file_path.clone();
    let status_message = app_state.read().environment_editor.status_message.clone();
    let env_names = app_state.read().environment_editor.environment_names();
    let editing_environment = app_state
        .read()
        .environment_editor
        .editing_environment
        .clone();
    let pending_delete_env = app_state
        .read()
        .environment_editor
        .pending_delete_env
        .clone();
    let new_env_name = app_state.read().environment_editor.new_env_name.clone();
    let new_var_name = app_state.read().environment_editor.new_var_name.clone();
    let new_var_value = app_state.read().environment_editor.new_var_value.clone();
    let has_changes = app_state.read().environment_editor.has_changes();
    let variables = editing_environment
        .as_ref()
        .and_then(|environment| {
            app_state
                .read()
                .environment_editor
                .config
                .get(environment)
                .cloned()
        })
        .unwrap_or_default();
    let mut sorted_variables: Vec<(String, String)> = variables.into_iter().collect();
    sorted_variables.sort_by(|left, right| left.0.cmp(&right.0));
    let storage_hint = render_environment_storage_hint(env_file_path);

    rsx! {
        div {
            class: "environment-pane",
            h3 { "🌍 Environment Editor" }
            {storage_hint}
            if let Some(message) = status_message {
                div { class: "status-chip success", "{message}" }
            }
            div {
                class: "add-row",
                input {
                    class: "text-input",
                    placeholder: "New environment",
                    value: new_env_name,
                    oninput: move |event| {
                        let value = event.value();
                        app_state.with_mut(|model| model.environment_editor.new_env_name = value);
                    }
                }
                button {
                    class: "btn",
                    onclick: move |_| {
                        app_state.with_mut(|model| {
                            let name = model.environment_editor.new_env_name.trim().to_string();
                            if !name.is_empty() && !model.environment_editor.config.contains_key(&name) {
                                model.environment_editor.config.insert(name.clone(), Default::default());
                                model.environment_editor.editing_environment = Some(name);
                                model.environment_editor.has_changes = true;
                                model.environment_editor.status_message = None;
                                model.refresh_environment_names();
                            }
                            model.environment_editor.new_env_name.clear();
                        });
                    },
                    "➕ Add"
                }
            }
            if env_names.is_empty() {
                div {
                    class: "empty-state",
                    p { "No environments defined. Add one above." }
                }
            } else {
                div {
                    class: "tab-bar environment-tabs",
                    for env_name in env_names.clone() {
                        button {
                            class: if editing_environment.as_ref() == Some(&env_name) { "btn is-active" } else { "btn" },
                            onclick: move |_| {
                                app_state.with_mut(|model| {
                                    model.environment_editor.editing_environment = Some(env_name.clone());
                                    model.environment_editor.pending_delete_env = None;
                                });
                            },
                            "{env_name}"
                        }
                    }
                }
                if let Some(editing_environment_name) = editing_environment {
                    div {
                        class: "request-section",
                        div {
                            class: "section-header",
                            h4 { "📋 {editing_environment_name}" }
                            if pending_delete_env.as_ref() == Some(&editing_environment_name) {
                                div {
                                    class: "confirmation-row",
                                    span { class: "status-chip danger", "Delete this environment?" }
                                    button {
                                        class: "btn btn-danger",
                                        onclick: move |_| {
                                            app_state.with_mut(|model| {
                                                model.environment_editor.config.remove(&editing_environment_name);
                                                model.environment_editor.editing_environment = None;
                                                model.environment_editor.pending_delete_env = None;
                                                model.environment_editor.has_changes = true;
                                                model.refresh_environment_names();
                                            });
                                        },
                                        "Yes"
                                    }
                                    button {
                                        class: "btn",
                                        onclick: move |_| {
                                            app_state.with_mut(|model| model.environment_editor.pending_delete_env = None);
                                        },
                                        "No"
                                    }
                                }
                            } else {
                                button {
                                    class: "btn btn-danger",
                                    onclick: move |_| {
                                        app_state.with_mut(|model| {
                                            model.environment_editor.pending_delete_env = Some(editing_environment_name.clone());
                                        });
                                    },
                                    "🗑 Delete"
                                }
                            }
                        }
                        div {
                            class: "env-grid",
                            div { class: "field-label", "Variable" }
                            div { class: "field-label", "Value" }
                            div {}
                            for (name, value) in sorted_variables {
                                {render_environment_variable_row(app_state, name, value)}
                            }
                        }
                        div {
                            class: "add-row",
                            input {
                                class: "text-input monospace-input",
                                placeholder: "Variable name",
                                value: new_var_name,
                                oninput: move |event| {
                                    let value = event.value();
                                    app_state.with_mut(|model| model.environment_editor.new_var_name = value);
                                }
                            }
                            input {
                                class: "text-input monospace-input",
                                placeholder: "Variable value",
                                value: new_var_value,
                                oninput: move |event| {
                                    let value = event.value();
                                    app_state.with_mut(|model| model.environment_editor.new_var_value = value);
                                }
                            }
                            button {
                                class: "btn",
                                onclick: move |_| {
                                    app_state.with_mut(|model| {
                                        let key = model.environment_editor.new_var_name.trim().to_string();
                                        if !key.is_empty() {
                                            if let Some(environment) = model.environment_editor.editing_environment.clone()
                                                && let Some(values) = model.environment_editor.config.get_mut(&environment)
                                            {
                                                values.insert(key, model.environment_editor.new_var_value.clone());
                                                model.environment_editor.has_changes = true;
                                                model.environment_editor.status_message = None;
                                            }
                                            model.environment_editor.new_var_name.clear();
                                            model.environment_editor.new_var_value.clear();
                                        }
                                    });
                                },
                                "➕ Add Variable"
                            }
                        }
                    }
                }
            }
            div {
                class: "pane-footer",
                button {
                    class: "btn",
                    onclick: move |_| {
                        app_state.with_mut(|model| {
                            model.environment_editor.save();
                            model.refresh_environment_names();
                            model.persist();
                        });
                    },
                    "💾 Save"
                }
                if has_changes {
                    span { class: "status-chip warning", "● Unsaved changes" }
                }
            }
        }
    }
}

fn render_environment_variable_row(
    mut app_state: Signal<AppModel>,
    name: String,
    value: String,
) -> Element {
    let update_name = name.clone();
    let delete_name = name.clone();

    rsx! {
        div { class: "env-name", "{name}" }
        input {
            class: "text-input monospace-input",
            value: value,
            oninput: move |event| {
                let new_value = event.value();
                app_state.with_mut(|model| {
                    if let Some(environment) = model.environment_editor.editing_environment.clone()
                        && let Some(values) = model.environment_editor.config.get_mut(&environment)
                    {
                        values.insert(update_name.clone(), new_value);
                        model.environment_editor.has_changes = true;
                        model.environment_editor.status_message = None;
                    }
                });
            }
        }
        button {
            class: "btn btn-danger btn-icon",
            onclick: move |_| {
                app_state.with_mut(|model| {
                    if let Some(environment) = model.environment_editor.editing_environment.clone()
                        && let Some(values) = model.environment_editor.config.get_mut(&environment)
                    {
                        values.remove(&delete_name);
                        model.environment_editor.has_changes = true;
                        model.environment_editor.status_message = None;
                    }
                });
            },
            "🗑"
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn render_environment_storage_hint(env_file_path: Option<PathBuf>) -> Element {
    if let Some(path) = env_file_path {
        rsx! { p { class: "small-hint", "File: {path.display()}" } }
    } else {
        rsx! {}
    }
}

#[cfg(target_arch = "wasm32")]
fn render_environment_storage_hint(_env_file_path: Option<PathBuf>) -> Element {
    rsx! { p { class: "small-hint", "Stored in browser localStorage" } }
}

fn render_results_panel(mut app_state: Signal<AppModel>) -> Element {
    let compact_mode = app_state.read().results_compact_mode;
    let results = app_state.read().results.clone();
    let is_running = app_state.read().is_running;
    let status_message = app_state.read().status_message.clone();

    rsx! {
        div {
            class: "results-card",
            div {
                class: "card-header results-header",
                h2 { class: "card-title", "Results" }
                div { class: "toolbar-spacer" }
                button {
                    class: if compact_mode { "btn is-active" } else { "btn" },
                    onclick: move |_| {
                        app_state.with_mut(|model| {
                            model.results_compact_mode = true;
                            model.persist();
                        });
                    },
                    "📋 Compact"
                }
                button {
                    class: if compact_mode { "btn" } else { "btn is-active" },
                    onclick: move |_| {
                        app_state.with_mut(|model| {
                            model.results_compact_mode = false;
                            model.persist();
                        });
                    },
                    "📄 Verbose"
                }
            }
            div {
                class: "card-content results-content",
                if is_running {
                    div {
                        class: "status-chip info",
                        if let Some(message) = status_message {
                            "⏳ {message}"
                        } else {
                            "⏳ Running requests..."
                        }
                    }
                }
                if results.is_empty() && !is_running {
                    div {
                        class: "empty-state",
                        p { "No results yet. Select and run a request." }
                    }
                } else {
                    for (index, result) in results.iter().cloned().enumerate() {
                        if compact_mode {
                            {render_compact_result(result)}
                        } else {
                            {render_verbose_result(index, result)}
                        }
                    }
                }
            }
        }
    }
}

fn render_compact_result(result: ExecutionResult) -> Element {
    match result {
        ExecutionResult::Success {
            method,
            url,
            status,
            duration_ms,
            assertion_results,
            ..
        } => rsx! {
            div {
                class: "result-card success",
                div { class: "result-line", span { class: "result-icon", "✅" } code { "{method} {url}" } span { class: "result-meta", "{status} | {duration_ms} ms" } }
                if !assertion_results.is_empty() {
                    div {
                        class: "result-assertions",
                        for assertion_result in assertion_results {
                            {render_compact_assertion(assertion_result)}
                        }
                    }
                }
            }
        },
        ExecutionResult::Failure { method, url, error } => rsx! {
            div {
                class: "result-card failure",
                div { class: "result-line", span { class: "result-icon", "❌" } code { "{method} {url}" } }
                div { class: "result-error", "{error}" }
            }
        },
    }
}

fn render_compact_assertion(assertion_result: AssertionResult) -> Element {
    let label = assertion_label(&assertion_result);
    if assertion_result.passed {
        rsx! {
            div { class: "assertion-line success", "✅ {label}: Expected '{assertion_result.assertion.expected_value}'" }
        }
    } else {
        let error_message = assertion_result
            .error_message
            .clone()
            .unwrap_or_else(|| "Failed".to_string());
        let detail = assertion_result.actual_value.clone().map(|actual| {
            format!(
                "Expected: '{}', Actual: '{}'",
                assertion_result.assertion.expected_value, actual
            )
        });
        rsx! {
            div {
                class: "assertion-group failure",
                div { class: "assertion-line failure", "❌ {label}: {error_message}" }
                if let Some(detail) = detail {
                    div { class: "assertion-detail", "{detail}" }
                }
            }
        }
    }
}

fn render_verbose_result(index: usize, result: ExecutionResult) -> Element {
    match result {
        ExecutionResult::Success {
            method,
            url,
            status,
            duration_ms,
            request_body,
            response_body,
            assertion_results,
        } => rsx! {
            div {
                class: "result-card success",
                div { class: "result-line", span { class: "result-icon", "✅ SUCCESS" } }
                code { class: "result-url", "{method} {url}" }
                div { class: "result-meta", "Status: {status}" }
                div { class: "result-meta", "Duration: {duration_ms} ms" }
                if !assertion_results.is_empty() {
                    div {
                        class: "request-section",
                        strong { "🔍 Assertion Results:" }
                        for assertion_result in assertion_results {
                            {render_verbose_assertion(assertion_result)}
                        }
                    }
                }
                if let Some(body) = request_body
                    && !body.trim().is_empty()
                {
                    div {
                        class: "request-section",
                        strong { "Request Body:" }
                        pre { class: "code-block result-snippet", key: "request-body-{index}", "{body}" }
                    }
                }
                if !response_body.trim().is_empty() {
                    div {
                        class: "request-section",
                        strong { "Response:" }
                        pre { class: "code-block response-snippet", key: "response-body-{index}", "{response_body}" }
                    }
                }
            }
        },
        ExecutionResult::Failure { method, url, error } => rsx! {
            div {
                class: "result-card failure",
                div { class: "result-line", span { class: "result-icon", "❌ FAILED" } }
                code { class: "result-url", "{method} {url}" }
                div { class: "result-error", "{error}" }
            }
        },
    }
}

fn render_verbose_assertion(assertion_result: AssertionResult) -> Element {
    let label = assertion_label(&assertion_result);
    if assertion_result.passed {
        rsx! {
            div { class: "assertion-line success", "✅ {label}: Expected '{assertion_result.assertion.expected_value}'" }
        }
    } else {
        let error_message = assertion_result
            .error_message
            .clone()
            .unwrap_or_else(|| "Failed".to_string());
        rsx! {
            div {
                class: "assertion-group failure",
                div { class: "assertion-line failure", "❌ {label}: {error_message}" }
                if let Some(actual) = assertion_result.actual_value {
                    div { class: "assertion-detail", "Expected: '{assertion_result.assertion.expected_value}'" }
                    div { class: "assertion-detail", "Actual: '{actual}'" }
                }
            }
        }
    }
}

fn render_status_bar(app_state: Signal<AppModel>) -> Element {
    let root_directory = app_state.read().root_directory.clone();
    let selected_file = app_state.read().selected_file.clone();
    let support_key = app_state.read().support_key.clone();
    let working_directory = render_working_directory(root_directory);

    rsx! {
        div {
            class: "statusbar",
            {working_directory}
            if let Some(file) = selected_file {
                span { "Selected: {file.display()}" }
            }
            if let Some(support_key) = support_key {
                span { "Support: {support_key}" }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn render_working_directory(root_directory: PathBuf) -> Element {
    rsx! { span { "Working Directory: {root_directory.display()}" } }
}

#[cfg(target_arch = "wasm32")]
fn render_working_directory(_root_directory: PathBuf) -> Element {
    rsx! {}
}

fn editor_shortcut_hint() -> &'static str {
    #[cfg(not(target_arch = "wasm32"))]
    {
        "Ctrl+T to toggle | Ctrl+S to save | Ctrl+B to toggle file tree"
    }

    #[cfg(target_arch = "wasm32")]
    {
        "Ctrl+T to toggle | Ctrl+S to save"
    }
}

fn assertion_label(assertion_result: &AssertionResult) -> &'static str {
    match assertion_result.assertion.assertion_type {
        httprunner_core::types::AssertionType::Status => "Status Code",
        httprunner_core::types::AssertionType::Body => "Response Body",
        httprunner_core::types::AssertionType::Headers => "Response Headers",
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn handle_keyboard_shortcut(
    event: KeyboardEvent,
    mut app_state: Signal<AppModel>,
    sender: UnboundedSender<AppEvent>,
    desktop_window: DesktopContext,
) {
    if handle_shared_shortcuts(&event, app_state, sender.clone()) {
        return;
    }

    if !shortcut_modifier(&event) {
        return;
    }

    if let Key::Character(character) = event.key() {
        match character.to_lowercase().as_str() {
            "o" => {
                event.prevent_default();
                open_directory_dialog(app_state, sender);
            }
            "q" => {
                event.prevent_default();
                quit_application(app_state, desktop_window);
            }
            "b" => {
                event.prevent_default();
                app_state.with_mut(|model| {
                    model.file_tree_visible = !model.file_tree_visible;
                    model.persist();
                });
            }
            _ => {}
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn handle_keyboard_shortcut(
    event: KeyboardEvent,
    app_state: Signal<AppModel>,
    sender: UnboundedSender<AppEvent>,
) {
    let _ = handle_shared_shortcuts(&event, app_state, sender);
}

fn handle_shared_shortcuts(
    event: &KeyboardEvent,
    mut app_state: Signal<AppModel>,
    sender: UnboundedSender<AppEvent>,
) -> bool {
    if event.code() == Code::F5 {
        event.prevent_default();
        let model = app_state.peek();
        model.start_run_all(sender);
        return true;
    }

    if let Key::Character(character) = event.key()
        && shortcut_modifier(event)
    {
        return match character.to_lowercase().as_str() {
            "+" | "=" => {
                event.prevent_default();
                app_state.with_mut(|model| model.change_font_size(AppModel::FONT_SIZE_STEP));
                true
            }
            "-" => {
                event.prevent_default();
                app_state.with_mut(|model| model.change_font_size(-AppModel::FONT_SIZE_STEP));
                true
            }
            "0" => {
                event.prevent_default();
                app_state.with_mut(|model| model.reset_font_size());
                true
            }
            "r" => {
                event.prevent_default();
                let model = app_state.peek();
                model.start_run_all(sender);
                true
            }
            "e" => {
                event.prevent_default();
                app_state.with_mut(|model| model.cycle_environment());
                true
            }
            "t" => {
                event.prevent_default();
                app_state.with_mut(|model| model.cycle_view_mode());
                true
            }
            "d" => {
                event.prevent_default();
                app_state.with_mut(|model| {
                    model.results_compact_mode = !model.results_compact_mode;
                    model.persist();
                });
                true
            }
            "s" => {
                event.prevent_default();
                app_state.with_mut(|model| match model.view_mode {
                    ViewMode::TextEditor => {
                        if let Err(error) = model.text_editor.save_to_file() {
                            eprintln!("Failed to save file: {}", error);
                        } else {
                            #[cfg(target_arch = "wasm32")]
                            model.sync_request_editor_from_text();
                            model.persist();
                        }
                    }
                    ViewMode::RequestDetails => {
                        if let Err(error) = model.save_request_editor_changes() {
                            eprintln!("Failed to save file: {}", error);
                        } else {
                            model.persist();
                        }
                    }
                    ViewMode::EnvironmentEditor => {
                        model.environment_editor.save();
                        model.refresh_environment_names();
                        model.persist();
                    }
                });
                true
            }
            _ => false,
        };
    }

    false
}

fn shortcut_modifier(event: &KeyboardEvent) -> bool {
    let modifiers = event.modifiers();
    modifiers.ctrl() || modifiers.meta()
}

#[cfg(not(target_arch = "wasm32"))]
fn open_directory_dialog(mut app_state: Signal<AppModel>, sender: UnboundedSender<AppEvent>) {
    if let Some(path) = rfd::FileDialog::new().pick_folder() {
        app_state.with_mut(|model| {
            model.root_directory = path.clone();
            model.selected_file = None;
            model.text_editor = TextEditor::new();
            model.request_editor = RequestEditor::new();
            model.environment_editor = EnvironmentEditor::new();
            model.environments.clear();
            model.selected_environment = None;
            model.http_files.clear();
            model.expanded_dirs.clear();
            model.is_discovering = true;
            model.discovered_count = 0;
            model.editor_scroll_top = 0.0;
            model.editor_scroll_left = 0.0;
            model.persist();
        });
        file_tree::start_discovery(path, sender);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn create_new_http_file(mut app_state: Signal<AppModel>, sender: UnboundedSender<AppEvent>) {
    let root_directory = app_state.peek().root_directory.clone();
    if let Some(path) = rfd::FileDialog::new()
        .set_directory(&root_directory)
        .add_filter("HTTP Files", &["http"])
        .set_file_name("new.http")
        .save_file()
    {
        if let Err(error) = std::fs::write(&path, NEW_FILE_TEMPLATE) {
            eprintln!("Failed to create file: {}", error);
            return;
        }

        app_state.with_mut(|model| {
            model.select_file(path.clone());
            model.view_mode = ViewMode::TextEditor;
        });
        file_tree::start_discovery(root_directory, sender);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn quit_application(app_state: Signal<AppModel>, desktop_window: DesktopContext) {
    app_state.with(|model| model.persist());
    desktop_window.close();
}
