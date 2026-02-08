use super::{
    environment_editor::EnvironmentEditor,
    file_tree::FileTree,
    request_view::RequestView,
    results_view::ResultsView,
    state::AppState,
    text_editor::TextEditor,
};
use httprunner_core::telemetry;
use iced::{
    widget::{button, column, container, row, text, Row}, Element, Length, Task,
};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    TextEditor,
    RequestDetails,
    EnvironmentEditor,
}

#[derive(Debug, Clone)]
pub enum Message {
    // File operations
    OpenFolder,
    FolderSelected(Option<PathBuf>),
    FileSelected(PathBuf),
    #[allow(dead_code)]
    SaveFile,
    #[allow(dead_code)]
    NewFile,
    
    // Request operations
    RunAllRequests,
    #[allow(dead_code)]
    RunRequest(usize),
    
    // Environment operations
    #[allow(dead_code)]
    EnvironmentChanged(String),
    #[allow(dead_code)]
    SwitchEnvironment,
    
    // View operations
    ToggleView,
    ToggleFileTree,
    #[allow(dead_code)]
    ToggleResultsView,
    
    // Settings
    #[allow(dead_code)]
    DelayChanged(u64),
    #[allow(dead_code)]
    ToggleTelemetry,
    #[allow(dead_code)]
    FontSizeIncrease,
    #[allow(dead_code)]
    FontSizeDecrease,
    #[allow(dead_code)]
    FontSizeReset,
    
    // Editor operations
    #[allow(dead_code)]
    TextEdited(String),
    
    // Results
    ResultsReceived(Vec<httprunner_core::HttpResult>),
    
    // Window operations
    #[allow(dead_code)]
    Quit,
    #[allow(dead_code)]
    WindowResized(f32, f32),
}

pub struct HttpRunnerApp {
    file_tree: FileTree,
    request_view: RequestView,
    text_editor: TextEditor,
    results_view: ResultsView,
    environment_editor: EnvironmentEditor,
    selected_file: Option<PathBuf>,
    #[allow(dead_code)]
    selected_request_index: Option<usize>,
    environments: Vec<String>,
    selected_environment: Option<String>,
    root_directory: PathBuf,
    font_size: f32,
    #[allow(dead_code)]
    environment_selector_open: bool,
    view_mode: ViewMode,
    file_tree_visible: bool,
    telemetry_enabled: bool,
    delay_ms: u64,
}

impl HttpRunnerApp {
    const DEFAULT_FONT_SIZE: f32 = 14.0;
    const MIN_FONT_SIZE: f32 = 8.0;
    const MAX_FONT_SIZE: f32 = 32.0;
    const FONT_SIZE_STEP: f32 = 1.0;

    pub fn new() -> (Self, Task<Message>) {
        let state = AppState::load();

        let root_directory = state
            .root_directory
            .and_then(|p| if p.exists() { Some(p) } else { None })
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let font_size = state.font_size.unwrap_or(Self::DEFAULT_FONT_SIZE);
        let file_tree_visible = state.file_tree_visible.unwrap_or(true);
        let telemetry_enabled = state.telemetry_enabled.unwrap_or(true);
        let delay_ms = state.delay_ms.unwrap_or(0);

        let mut app = Self {
            file_tree: FileTree::new(root_directory.clone()),
            request_view: RequestView::new(),
            text_editor: TextEditor::new(),
            results_view: ResultsView::new(),
            environment_editor: EnvironmentEditor::new(),
            selected_file: None,
            selected_request_index: None,
            environments: Vec::new(),
            selected_environment: None,
            root_directory,
            font_size,
            environment_selector_open: false,
            view_mode: ViewMode::TextEditor,
            file_tree_visible,
            telemetry_enabled,
            delay_ms,
        };

        if let Some(last_results) = state.last_results {
            app.results_view.restore_results(last_results);
        }

        app.results_view
            .set_compact_mode(state.results_compact_mode.unwrap_or(true));

        if let Some(saved_file) = state.selected_file
            && saved_file.exists() {
                app.selected_file = Some(saved_file.clone());
                app.load_environments(&saved_file);
                app.request_view.load_file(&saved_file);
                app.text_editor.load_file(&saved_file);

                if let Some(saved_env) = state.selected_environment
                    && app.environments.contains(&saved_env) {
                        app.selected_environment = Some(saved_env);
                    }
            }

        (app, Task::none())
    }

    fn load_environments(&mut self, file_path: &Path) {
        if let Some(file_str) = file_path.to_str()
            && let Ok(Some(env_file)) = httprunner_core::environment::find_environment_file(file_str)
                && let Ok(config) = httprunner_core::environment::parse_environment_file(&env_file) {
                    self.environments = config.keys().cloned().collect();
                }
    }

    fn save_state(&self) {
        let state = AppState {
            root_directory: Some(self.root_directory.clone()),
            selected_file: self.selected_file.clone(),
            selected_environment: self.selected_environment.clone(),
            window_size: None, // Will be updated by window resize events
            font_size: Some(self.font_size),
            file_tree_visible: Some(self.file_tree_visible),
            telemetry_enabled: Some(self.telemetry_enabled),
            delay_ms: Some(self.delay_ms),
            results_compact_mode: Some(self.results_view.is_compact_mode()),
            last_results: Some(self.results_view.get_all_results()),
        };
        let _ = state.save();
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFolder => {
                // In Iced, file dialogs need to be handled async
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .pick_folder()
                            .await
                            .map(|handle| handle.path().to_path_buf())
                    },
                    Message::FolderSelected,
                )
            }
            Message::FolderSelected(Some(path)) => {
                self.root_directory = path.clone();
                self.file_tree = FileTree::new(path);
                self.selected_file = None;
                self.selected_request_index = None;
                self.save_state();
                Task::none()
            }
            Message::FolderSelected(None) => Task::none(),
            Message::FileSelected(path) => {
                self.selected_file = Some(path.clone());
                self.load_environments(&path);
                self.request_view.load_file(&path);
                self.text_editor.load_file(&path);
                self.save_state();
                Task::none()
            }
            Message::SaveFile => {
                if let Err(e) = self.text_editor.save_to_file() {
                    eprintln!("Failed to save file: {}", e);
                } else {
                    // Reload request view after saving
                    if let Some(path) = &self.selected_file {
                        self.request_view.load_file(path);
                    }
                }
                Task::none()
            }
            Message::NewFile => {
                // TODO: Implement new file creation dialog
                Task::none()
            }
            Message::RunAllRequests => {
                if let Some(file_path) = &self.selected_file {
                    let file_path_str = file_path.to_string_lossy().to_string();
                    let env = self.selected_environment.clone();
                    let delay_ms = self.delay_ms;
                    
                    Task::perform(
                        async move {
                            let mut results = Vec::new();
                            let _ = httprunner_core::processor::process_http_file_incremental(
                                &file_path_str,
                                env.as_deref(),
                                false, // insecure
                                delay_ms,
                                |_idx, _total, processing_result| {
                                    if let httprunner_core::processor::RequestProcessingResult::Executed { result, .. } = processing_result {
                                        results.push(result);
                                    }
                                    true // Continue processing
                                },
                            );
                            results
                        },
                        Message::ResultsReceived,
                    )
                } else {
                    Task::none()
                }
            }
            Message::RunRequest(_index) => {
                // TODO: Implement single request execution
                Task::none()
            }
            Message::EnvironmentChanged(env) => {
                self.selected_environment = if env.is_empty() { None } else { Some(env) };
                self.save_state();
                Task::none()
            }
            Message::SwitchEnvironment => {
                if !self.environments.is_empty() {
                    let current_idx = self
                        .selected_environment
                        .as_ref()
                        .and_then(|env| self.environments.iter().position(|e| e == env))
                        .unwrap_or(0);
                    let next_idx = (current_idx + 1) % self.environments.len();
                    self.selected_environment = Some(self.environments[next_idx].clone());
                    self.save_state();
                }
                Task::none()
            }
            Message::ToggleView => {
                self.view_mode = match self.view_mode {
                    ViewMode::TextEditor => ViewMode::RequestDetails,
                    ViewMode::RequestDetails => ViewMode::EnvironmentEditor,
                    ViewMode::EnvironmentEditor => ViewMode::TextEditor,
                };
                Task::none()
            }
            Message::ToggleFileTree => {
                self.file_tree_visible = !self.file_tree_visible;
                self.save_state();
                Task::none()
            }
            Message::ToggleResultsView => {
                self.results_view.toggle_compact_mode();
                self.save_state();
                Task::none()
            }
            Message::DelayChanged(delay) => {
                self.delay_ms = delay;
                self.save_state();
                Task::none()
            }
            Message::ToggleTelemetry => {
                self.telemetry_enabled = !self.telemetry_enabled;
                if let Err(e) = telemetry::set_enabled(self.telemetry_enabled) {
                    eprintln!("Failed to save telemetry setting: {}", e);
                }
                self.save_state();
                Task::none()
            }
            Message::FontSizeIncrease => {
                self.font_size = (self.font_size + Self::FONT_SIZE_STEP).min(Self::MAX_FONT_SIZE);
                self.save_state();
                Task::none()
            }
            Message::FontSizeDecrease => {
                self.font_size = (self.font_size - Self::FONT_SIZE_STEP).max(Self::MIN_FONT_SIZE);
                self.save_state();
                Task::none()
            }
            Message::FontSizeReset => {
                self.font_size = Self::DEFAULT_FONT_SIZE;
                self.save_state();
                Task::none()
            }
            Message::TextEdited(content) => {
                self.text_editor.set_content(content);
                Task::none()
            }
            Message::ResultsReceived(results) => {
                self.results_view.set_results(results);
                self.save_state();
                Task::none()
            }
            Message::Quit => {
                self.save_state();
                iced::exit()
            }
            Message::WindowResized(width, height) => {
                let mut state = AppState::load();
                state.window_size = Some((width, height));
                let _ = state.save();
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let top_bar = self.view_top_bar();
        let main_content = self.view_main_content();

        column![top_bar, main_content]
            .spacing(0)
            .into()
    }

    fn view_top_bar(&self) -> Element<'_, Message> {
        let open_folder_btn = button("Open Folder").on_press(Message::OpenFolder);
        let run_btn = button("▶ Run All").on_press(Message::RunAllRequests);
        let toggle_view_btn = button("Toggle View").on_press(Message::ToggleView);
        let toggle_tree_btn = button("Toggle Tree").on_press(Message::ToggleFileTree);
        
        let current_env = self.selected_environment.clone().unwrap_or_else(|| "None".to_string());
        let env_text = format!("Environment: {}", current_env);
        
        let toolbar = row![
            open_folder_btn,
            run_btn,
            toggle_view_btn,
            toggle_tree_btn,
            text(env_text),
        ]
        .spacing(10)
        .padding(10);

        container(toolbar)
            .width(Length::Fill)
            .into()
    }

    fn view_main_content(&self) -> Element<'_, Message> {
        let mut main_row = Row::new().spacing(5).padding(5);

        // File tree (left panel)
        if self.file_tree_visible {
            let file_tree_content = self.file_tree.view();
            main_row = main_row.push(
                container(file_tree_content)
                    .width(Length::FillPortion(2))
                    .height(Length::Fill)
            );
        }

        // Center panel (editor or request view)
        let center_content = match self.view_mode {
            ViewMode::TextEditor => self.text_editor.view(),
            ViewMode::RequestDetails => self.request_view.view(),
            ViewMode::EnvironmentEditor => self.environment_editor.view(),
        };
        
        main_row = main_row.push(
            container(center_content)
                .width(if self.file_tree_visible { Length::FillPortion(5) } else { Length::FillPortion(7) })
                .height(Length::Fill)
        );

        // Results panel (right panel)
        let results_content = self.results_view.view();
        main_row = main_row.push(
            container(results_content)
                .width(Length::FillPortion(3))
                .height(Length::Fill)
        );

        container(main_row)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl Default for HttpRunnerApp {
    fn default() -> Self {
        Self::new().0
    }
}
