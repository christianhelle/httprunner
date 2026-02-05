use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use httprunner_lib::telemetry;
use std::path::PathBuf;

use crate::file_tree::FileTree;
use crate::request_view::RequestView;
use crate::results_view::ResultsView;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPane {
    FileTree,
    RequestView,
    ResultsView,
}

pub struct App {
    pub should_quit: bool,
    pub file_tree: FileTree,
    pub request_view: RequestView,
    pub results_view: ResultsView,
    pub focused_pane: FocusedPane,
    pub root_directory: PathBuf,
    pub selected_file: Option<PathBuf>,
    pub environments: Vec<String>,
    pub selected_environment: Option<String>,
    pub status_message: String,
    pub file_tree_visible: bool,
    pub telemetry_enabled: bool,
    pub delay_ms: u64,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let state = AppState::load();

        let root_directory = state
            .root_directory
            .and_then(|p| if p.exists() { Some(p) } else { None })
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let file_tree_visible = state.file_tree_visible.unwrap_or(true);
        let results_compact_mode = state.results_compact_mode.unwrap_or(true);
        let telemetry_enabled = state.telemetry_enabled.unwrap_or(true);
        let delay_ms = state.delay_ms.unwrap_or(0);

        let mut results_view = ResultsView::new();
        results_view.set_compact_mode(results_compact_mode);

        let mut app = Self {
            should_quit: false,
            file_tree: FileTree::new(root_directory.clone()),
            request_view: RequestView::new(),
            results_view,
            focused_pane: FocusedPane::FileTree,
            root_directory,
            selected_file: None,
            environments: Vec::new(),
            selected_environment: None,
            status_message: String::from("Ready"),
            file_tree_visible,
            telemetry_enabled,
            delay_ms,
        };

        if let Some(saved_file) = state.selected_file
            && saved_file.exists()
        {
            app.select_file(saved_file);
        }

        Ok(app)
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        // Global shortcuts
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL)
            | (KeyCode::Char('q'), KeyModifiers::NONE)
            | (KeyCode::Char('Q'), KeyModifiers::SHIFT) => {
                self.should_quit = true;
                self.save_state();
                return Ok(());
            }
            (KeyCode::BackTab, _) => {
                self.cycle_focus_reverse();
                return Ok(());
            }
            (KeyCode::Tab, _) => {
                self.cycle_focus();
                return Ok(());
            }
            (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                self.cycle_environment();
                return Ok(());
            }
            (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
                self.file_tree_visible = !self.file_tree_visible;
                // If hiding file tree and it was focused, switch to request view
                if !self.file_tree_visible && self.focused_pane == FocusedPane::FileTree {
                    self.focused_pane = FocusedPane::RequestView;
                }
                return Ok(());
            }
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                self.results_view.toggle_compact_mode();
                return Ok(());
            }
            (KeyCode::Char('t'), KeyModifiers::CONTROL) => {
                self.toggle_telemetry();
                return Ok(());
            }
            (KeyCode::Char(']'), KeyModifiers::NONE) => {
                self.increase_delay();
                return Ok(());
            }
            (KeyCode::Char('['), KeyModifiers::NONE) => {
                self.decrease_delay();
                return Ok(());
            }
            (KeyCode::F(5), _)
            | (KeyCode::Char('r'), KeyModifiers::CONTROL)
            | (KeyCode::Char('r'), KeyModifiers::NONE)
            | (KeyCode::Char('R'), KeyModifiers::SHIFT) => {
                self.run_all_requests();
                return Ok(());
            }
            _ => {}
        }

        // Pane-specific handling
        match self.focused_pane {
            FocusedPane::FileTree => {
                self.file_tree.handle_key_event(key);
                if let Some(selected) = self.file_tree.get_selected_file() {
                    self.select_file(selected);
                }
            }
            FocusedPane::RequestView => {
                self.request_view.handle_key_event(key);
                if self.request_view.should_run_request() {
                    self.run_selected_request();
                }
            }
            FocusedPane::ResultsView => {
                self.results_view.handle_key_event(key);
            }
        }

        Ok(())
    }

    fn cycle_focus(&mut self) {
        self.focused_pane = match self.focused_pane {
            FocusedPane::FileTree => FocusedPane::RequestView,
            FocusedPane::RequestView => FocusedPane::ResultsView,
            FocusedPane::ResultsView => {
                if self.file_tree_visible {
                    FocusedPane::FileTree
                } else {
                    FocusedPane::RequestView
                }
            }
        };
    }

    fn cycle_focus_reverse(&mut self) {
        if !self.file_tree_visible {
            // When the file tree is hidden, always normalize focus to RequestView.
            self.focused_pane = FocusedPane::RequestView;
            return;
        }

        self.focused_pane = match self.focused_pane {
            FocusedPane::FileTree => FocusedPane::ResultsView,
            FocusedPane::RequestView => FocusedPane::FileTree,
            FocusedPane::ResultsView => FocusedPane::RequestView,
        };
    }

    fn cycle_environment(&mut self) {
        if self.environments.is_empty() {
            return;
        }

        let current_index = self
            .selected_environment
            .as_ref()
            .and_then(|env| self.environments.iter().position(|e| e == env));

        let next_index = match current_index {
            Some(i) => (i + 1) % self.environments.len(),
            None => 0,
        };

        self.selected_environment = Some(self.environments[next_index].clone());
        self.status_message = format!("Environment: {}", self.environments[next_index]);
        self.save_state();
    }

    fn select_file(&mut self, path: PathBuf) {
        self.selected_file = Some(path.clone());
        self.load_environments(&path);
        self.request_view.load_file(&path);

        if let Some(error) = self.request_view.error_message() {
            self.status_message = error.clone();
        } else {
            self.status_message = format!("Loaded: {}", path.display());
        }

        self.save_state();
    }

    fn load_environments(&mut self, file: &std::path::Path) {
        if let Some(file_str) = file.to_str()
            && let Ok(Some(env_file)) = httprunner_lib::environment::find_environment_file(file_str)
        {
            if let Ok(env_config) = httprunner_lib::environment::parse_environment_file(&env_file) {
                self.environments = env_config.keys().cloned().collect();
                self.environments.sort();
                self.status_message = format!("Loaded {} environments", self.environments.len());
                return;
            }
            self.status_message = "Warning: Failed to parse environment file".to_string();
        }
        self.environments.clear();
        self.selected_environment = None;
    }

    fn run_all_requests(&mut self) {
        if let Some(file) = &self.selected_file {
            self.status_message = format!("Running requests from {}", file.display());

            // Track feature usage
            telemetry::track_feature_usage("run_all_requests");

            let file_path = file.clone();
            let env = self.selected_environment.clone();
            let incremental_results = self.results_view.incremental_results();
            let is_running = self.results_view.is_running_arc();
            let delay_ms = self.delay_ms; // Copy the delay value

            // Clear for async run
            self.results_view.clear_for_async_run();

            // Spawn background thread for async execution
            std::thread::spawn(move || {
                let path_str = file_path.to_string_lossy().to_string();
                let execution_start = std::time::Instant::now();

                let mut success_count = 0usize;
                let mut failed_count = 0usize;
                let mut skipped_count = 0usize;
                let mut total_count = 0usize;

                // Use the incremental processor which handles all features
                let result = httprunner_lib::processor::process_http_file_incremental(
                    &path_str,
                    env.as_deref(),
                    false, // insecure
                    delay_ms,
                    |_idx, total, process_result| {
                        total_count = total;

                        use httprunner_lib::processor::RequestProcessingResult;
                        match process_result {
                            RequestProcessingResult::Skipped { request, reason } => {
                                skipped_count += 1;
                                if let Ok(mut results) = incremental_results.lock() {
                                    results.push(crate::results_view::ExecutionResult::Failure {
                                        method: format!("⏭️ {}", request.method),
                                        url: request.url,
                                        error: format!("Skipped: {}", reason),
                                    });
                                }
                            }
                            RequestProcessingResult::Executed { request, result } => {
                                let request_body = request.body.clone();
                                if result.success {
                                    success_count += 1;
                                    if let Ok(mut results) = incremental_results.lock() {
                                        results.push(
                                            crate::results_view::ExecutionResult::Success {
                                                method: request.method,
                                                url: request.url,
                                                status: result.status_code,
                                                duration_ms: result.duration_ms,
                                                request_body,
                                                response_body: result
                                                    .response_body
                                                    .unwrap_or_default(),
                                                assertion_results: result.assertion_results,
                                            },
                                        );
                                    }
                                } else {
                                    failed_count += 1;
                                    if let Ok(mut results) = incremental_results.lock() {
                                        results.push(
                                            crate::results_view::ExecutionResult::Failure {
                                                method: request.method,
                                                url: request.url,
                                                error: result
                                                    .error_message
                                                    .unwrap_or_else(|| "Unknown error".to_string()),
                                            },
                                        );
                                    }
                                }
                            }
                            RequestProcessingResult::Failed { request, error } => {
                                failed_count += 1;
                                if let Ok(mut results) = incremental_results.lock() {
                                    results.push(crate::results_view::ExecutionResult::Failure {
                                        method: request.method,
                                        url: request.url,
                                        error,
                                    });
                                }
                            }
                        }
                        // Continue processing all requests
                        true
                    },
                );

                if let Err(e) = result {
                    // Track parse error
                    telemetry::track_error_message(&format!("Parse error: {}", e));

                    if let Ok(mut results) = incremental_results.lock() {
                        results.push(crate::results_view::ExecutionResult::Failure {
                            method: "PARSE".to_string(),
                            url: path_str,
                            error: format!("Failed to parse file: {}", e),
                        });
                    }
                } else {
                    // Track execution completion
                    let total_duration = execution_start.elapsed().as_millis() as u64;

                    // Track parse metrics (approximate, since parsing is now integrated)
                    telemetry::track_parse_complete(total_count, 0);

                    telemetry::track_execution_complete(
                        success_count,
                        failed_count,
                        skipped_count,
                        total_duration,
                    );
                }

                // Mark as complete
                if let Ok(mut running) = is_running.lock() {
                    *running = false;
                }
            });
        }
    }

    fn run_selected_request(&mut self) {
        // Note: Running individual requests requires library support for single-request execution
        // Currently, the library's process_http_files function processes all requests in a file
        // This would need to be enhanced in httprunner-lib to support executing a single request by index
        if let Some(file) = &self.selected_file
            && self.request_view.get_selected_index().is_some()
        {
            self.status_message = format!(
                "Running individual requests not yet supported. Use F5/Ctrl+R to run all requests in {}",
                file.file_name().unwrap_or_default().to_string_lossy()
            );
        }
    }

    fn toggle_telemetry(&mut self) {
        self.telemetry_enabled = !self.telemetry_enabled;

        // Update the global telemetry state and persist
        if let Err(e) = telemetry::set_enabled(self.telemetry_enabled) {
            self.status_message = format!("Failed to save telemetry setting: {}", e);
        } else {
            self.status_message = if self.telemetry_enabled {
                "Telemetry enabled".to_string()
            } else {
                "Telemetry disabled".to_string()
            };
        }

        self.save_state();
    }

    fn increase_delay(&mut self) {
        self.delay_ms = (self.delay_ms + 100).min(10000); // Max 10 seconds
        self.status_message = format!("Delay: {}ms", self.delay_ms);
        self.save_state();
    }

    fn decrease_delay(&mut self) {
        self.delay_ms = self.delay_ms.saturating_sub(100);
        self.status_message = format!("Delay: {}ms", self.delay_ms);
        self.save_state();
    }

    fn save_state(&self) {
        let state = AppState {
            root_directory: Some(self.root_directory.clone()),
            selected_file: self.selected_file.clone(),
            selected_environment: self.selected_environment.clone(),
            window_size: None,
            font_size: None,
            file_tree_visible: Some(self.file_tree_visible),
            results_compact_mode: Some(self.results_view.is_compact_mode()),
            last_results: None,
            telemetry_enabled: Some(self.telemetry_enabled),
            delay_ms: Some(self.delay_ms),
        };
        state.save();
    }
}
