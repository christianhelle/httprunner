use super::{
    file_tree::FileTree,
    request_view::{RequestView, RequestViewAction},
    results_view::ResultsView,
    state::AppState,
    text_editor::{TextEditor, TextEditorAction},
};
use std::path::{Path, PathBuf};

enum KeyboardAction {
    None,
    RunAllRequests,
    OpenFolder,
    Quit,
    SwitchEnvironment,
    ToggleView,
    ToggleFileTree,
    SaveFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    TextEditor,
    RequestDetails,
}

pub struct HttpRunnerApp {
    file_tree: FileTree,
    request_view: RequestView,
    text_editor: TextEditor,
    results_view: ResultsView,
    selected_file: Option<PathBuf>,
    selected_request_index: Option<usize>,
    environments: Vec<String>,
    selected_environment: Option<String>,
    root_directory: PathBuf,
    font_size: f32,
    environment_selector_open: bool,
    last_saved_window_size: Option<(f32, f32)>,
    view_mode: ViewMode,
    file_tree_visible: bool,
}

impl HttpRunnerApp {
    const DEFAULT_FONT_SIZE: f32 = 14.0;
    const MIN_FONT_SIZE: f32 = 8.0;
    const MAX_FONT_SIZE: f32 = 32.0;
    const FONT_SIZE_STEP: f32 = 1.0;

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load saved state
        let state = AppState::load();

        // Use saved root directory or fall back to current directory
        let root_directory = state
            .root_directory
            .and_then(|p| if p.exists() { Some(p) } else { None })
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        // Use saved font size or default
        let font_size = state.font_size.unwrap_or(Self::DEFAULT_FONT_SIZE);

        let mut app = Self {
            file_tree: FileTree::new(root_directory.clone()),
            request_view: RequestView::new(),
            text_editor: TextEditor::new(),
            results_view: ResultsView::new(),
            selected_file: None,
            selected_request_index: None,
            environments: Vec::new(),
            selected_environment: None,
            root_directory,
            font_size,
            environment_selector_open: false,
            last_saved_window_size: state.window_size,
            view_mode: ViewMode::TextEditor, // Default to text editor for new files
            file_tree_visible: true,
        };

        // Apply the loaded font size to the UI context
        app.update_font_size(&cc.egui_ctx);

        // Restore last results if available
        if let Some(last_results) = state.last_results {
            app.results_view.restore_results(last_results);
        }

        // Restore selected file if it still exists
        if let Some(saved_file) = state.selected_file {
            if saved_file.exists() {
                app.selected_file = Some(saved_file.clone());
                app.load_environments(&saved_file);
                app.request_view.load_file(&saved_file);
                app.text_editor.load_file(&saved_file); // Load into text editor too

                // Restore selected environment if it's still valid
                if let Some(saved_env) = state.selected_environment {
                    if app.environments.contains(&saved_env) {
                        app.selected_environment = Some(saved_env);
                    }
                }
            }
        }

        app
    }

    fn update_font_size(&mut self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        // Update all text styles with the new font size
        let base_size = self.font_size;
        style.text_styles = [
            (
                egui::TextStyle::Small,
                egui::FontId::proportional(base_size * 0.857),
            ),
            (egui::TextStyle::Body, egui::FontId::proportional(base_size)),
            (
                egui::TextStyle::Button,
                egui::FontId::proportional(base_size),
            ),
            (
                egui::TextStyle::Heading,
                egui::FontId::proportional(base_size * 1.286),
            ),
            (
                egui::TextStyle::Monospace,
                egui::FontId::monospace(base_size),
            ),
        ]
        .into();

        ctx.set_style(style);
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) -> KeyboardAction {
        // Check for Ctrl + Plus (zoom in)
        if ctx.input(|i| {
            i.modifiers.ctrl && (i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals))
        }) {
            self.font_size = (self.font_size + Self::FONT_SIZE_STEP).min(Self::MAX_FONT_SIZE);
            self.update_font_size(ctx);
            self.save_state();
        }

        // Check for Ctrl + Minus (zoom out)
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Minus)) {
            self.font_size = (self.font_size - Self::FONT_SIZE_STEP).max(Self::MIN_FONT_SIZE);
            self.update_font_size(ctx);
            self.save_state();
        }

        // Check for Ctrl + 0 (reset to default)
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Num0)) {
            self.font_size = Self::DEFAULT_FONT_SIZE;
            self.update_font_size(ctx);
            self.save_state();
        }

        // Check for F5 (run all requests for selected file)
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            return KeyboardAction::RunAllRequests;
        }

        // Check for Ctrl+R (run all requests for selected file)
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::R)) {
            return KeyboardAction::RunAllRequests;
        }

        // Check for Ctrl+O (open folder)
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::O)) {
            return KeyboardAction::OpenFolder;
        }

        // Check for Ctrl+Q (quit)
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Q)) {
            return KeyboardAction::Quit;
        }

        // Check for Ctrl+E (switch environment)
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::E)) {
            return KeyboardAction::SwitchEnvironment;
        }

        // Check for Ctrl+T (toggle between Text Editor and Request Details)
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::T)) {
            return KeyboardAction::ToggleView;
        }

        // Check for Ctrl+B (toggle file tree visibility)
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::B)) {
            return KeyboardAction::ToggleFileTree;
        }

        // Check for Ctrl+S (save file)
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            return KeyboardAction::SaveFile;
        }

        KeyboardAction::None
    }

    fn show_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Directory...").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.root_directory = path.clone();
                            self.file_tree = FileTree::new(path);
                            self.selected_file = None;
                            self.selected_request_index = None;
                            self.save_state();
                        }
                        ui.close();
                    }

                    if ui.button("New .http File...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_directory(&self.root_directory)
                            .add_filter("HTTP Files", &["http"])
                            .set_file_name("new.http")
                            .save_file()
                        {
                            // Create an empty .http file
                            if let Err(e) = std::fs::write(
                                &path,
                                "### New Request\nGET https://httpbin.org/get\n",
                            ) {
                                eprintln!("Failed to create file: {}", e);
                            } else {
                                // Refresh file tree and select the new file
                                self.file_tree = FileTree::new(self.root_directory.clone());
                                self.selected_file = Some(path.clone());
                                self.request_view.load_file(&path);
                                self.text_editor.load_file(&path);
                                // Switch to text editor view for new files
                                self.view_mode = ViewMode::TextEditor;
                                self.save_state();
                            }
                        }
                        ui.close();
                    }

                    ui.separator();

                    if ui.button("Quit").clicked() {
                        self.save_state();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.separator();

                ui.label("Environment:");
                let combo = egui::ComboBox::from_id_salt("env_selector")
                    .selected_text(self.selected_environment.as_deref().unwrap_or("None"));

                let previous_env = self.selected_environment.clone();
                let response = combo.show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_environment, None, "None");
                    for env in &self.environments {
                        let env_clone = Some(env.clone());
                        ui.selectable_value(&mut self.selected_environment, env_clone, env);
                    }
                });

                // Save state if environment changed
                if previous_env != self.selected_environment {
                    self.save_state();
                }

                // Track whether the combo box is open by checking if the popup is actually open
                self.environment_selector_open = response.response.has_focus()
                    || egui::containers::Popup::is_id_open(ui.ctx(), response.response.id);
            });
        });
    }

    fn show_bottom_panel(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Working Directory: {}",
                    self.root_directory.display()
                ));
                ui.separator();
                if let Some(file) = &self.selected_file {
                    ui.label(format!("Selected: {}", file.display()));
                }
            });
        });
    }

    fn load_environments(&mut self, file: &Path) {
        // Try to find and parse http-client.env.json
        if let Some(file_str) = file.to_str()
            && let Ok(Some(env_file)) = httprunner_lib::environment::find_environment_file(file_str)
            && let Ok(env_config) = httprunner_lib::environment::parse_environment_file(&env_file)
        {
            // Extract environment names from the config
            self.environments = env_config.keys().cloned().collect();
            self.environments.sort(); // Sort alphabetically for consistent UI
            return;
        }
        // No environments found or error occurred
        self.environments = Vec::new();
    }

    fn save_state(&self) {
        self.save_state_internal(None);
    }

    fn save_state_with_window(&self, ctx: &egui::Context) {
        // Get viewport size from context
        let window_size = ctx.input(|i| i.viewport().inner_rect.map(|r| r.size()).unwrap_or(egui::vec2(1200.0, 800.0)));
        self.save_state_internal(Some((window_size.x, window_size.y)));
    }

    fn save_state_internal(&self, window_size: Option<(f32, f32)>) {
        let state = AppState {
            root_directory: Some(self.root_directory.clone()),
            selected_file: self.selected_file.clone(),
            selected_environment: self.selected_environment.clone(),
            font_size: Some(self.font_size),
            window_size,
            last_results: Some(self.results_view.get_results()),
        };

        if let Err(e) = state.save() {
            eprintln!("Failed to save application state: {}", e);
        }
    }
}

impl eframe::App for HttpRunnerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let keyboard_action = self.handle_keyboard_shortcuts(ctx);

        // Process keyboard actions
        match keyboard_action {
            KeyboardAction::RunAllRequests => {
                if !self.request_view.has_changes()
                    && let Some(file) = &self.selected_file
                {
                    self.results_view
                        .run_file(file, self.selected_environment.as_deref());
                }
            }
            KeyboardAction::OpenFolder => {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.root_directory = path.clone();
                    self.file_tree = FileTree::new(path);
                    self.selected_file = None;
                    self.selected_request_index = None;
                    self.save_state();
                }
            }
            KeyboardAction::Quit => {
                self.save_state_with_window(ctx);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            KeyboardAction::SwitchEnvironment => {
                if !self.environment_selector_open {
                    // Cycle through environments
                    if self.environments.is_empty() {
                        // No environments available
                    } else if let Some(ref current_env) = self.selected_environment {
                        // Find current index and move to next
                        if let Some(idx) = self.environments.iter().position(|e| e == current_env) {
                            let next_idx = (idx + 1) % (self.environments.len() + 1);
                            self.selected_environment = if next_idx == self.environments.len() {
                                None
                            } else {
                                Some(self.environments[next_idx].clone())
                            };
                        } else {
                            // Current environment not found; reset to first environment
                            self.selected_environment = self.environments.first().cloned();
                        }
                    } else {
                        // Currently "None", switch to first environment
                        self.selected_environment = self.environments.first().cloned();
                    }
                    self.save_state();
                }
            }
            KeyboardAction::ToggleView => {
                // Toggle between text editor and request details view
                self.view_mode = match self.view_mode {
                    ViewMode::TextEditor => ViewMode::RequestDetails,
                    ViewMode::RequestDetails => ViewMode::TextEditor,
                };
            }
            KeyboardAction::ToggleFileTree => {
                // Toggle file tree visibility
                self.file_tree_visible = !self.file_tree_visible;
            }
            KeyboardAction::SaveFile => {
                // Save file based on current view mode
                match self.view_mode {
                    ViewMode::TextEditor => {
                        if let Err(e) = self.text_editor.save_to_file() {
                            eprintln!("Failed to save file: {}", e);
                        }
                    }
                    ViewMode::RequestDetails => {
                        if let Err(e) = self.request_view.save_to_file() {
                            eprintln!("Failed to save file: {}", e);
                        } else {
                            // Refresh the file tree to show any new files
                            self.file_tree = FileTree::new(self.root_directory.clone());
                            // Reload text editor with updated content
                            if let Some(file) = &self.selected_file {
                                self.text_editor.load_file(file);
                            }
                        }
                    }
                }
            }
            KeyboardAction::None => {}
        }

        self.show_top_panel(ctx);
        self.show_bottom_panel(ctx);

        // Left panel - File tree (only show if visible)
        if self.file_tree_visible {
            egui::SidePanel::left("file_tree_panel")
                .resizable(true)
                .default_width(300.0)
                .show(ctx, |ui| {
                    ui.heading("HTTP Files");
                    ui.separator();

                    if let Some(selected) = self.file_tree.show(ui) {
                        self.selected_file = Some(selected.clone());
                        self.selected_request_index = None;

                        // Load environments for this file
                        self.load_environments(&selected);

                        // Update both request view and text editor
                        self.request_view.load_file(&selected);
                        self.text_editor.load_file(&selected);

                        // Save state after file selection
                        self.save_state();
                    }
                });
        }

        // Right panel - Results (from main branch)
        egui::SidePanel::right("results_panel")
            .resizable(true)
            .default_width(500.0)
            .min_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Results");
                ui.separator();

                // Use all available height for results scroll area
                let available_height = ui.available_height();

                // Wrap results in a scroll area with explicit height
                egui::ScrollArea::vertical()
                    .id_salt("results_scroll")
                    .max_height(available_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.results_view.show(ui);
                    });
            });

        // Center panel - Text Editor or Request Details (from this branch)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                // Tab selection
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.view_mode, ViewMode::TextEditor, "📝 Text Editor");
                    ui.selectable_value(&mut self.view_mode, ViewMode::RequestDetails, "📋 Request Details");
                    ui.label("(Ctrl+T to toggle | Ctrl+S to save | Ctrl+B to toggle sidebar)");
                });
                ui.separator();

                // Use available space minus the button area
                let available_height = ui.available_height() - 40.0; // Reserve space for buttons

                // Show either text editor or request details based on view mode
                match self.view_mode {
                    ViewMode::TextEditor => {
                        // Wrap the text editor in a scroll area with fixed height
                        egui::ScrollArea::vertical()
                            .id_salt("text_editor_scroll")
                            .max_height(available_height)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                match self.text_editor.show(ui, &self.selected_file) {
                                    TextEditorAction::RunRequest(idx) => {
                                        self.selected_request_index = Some(idx);
                                        // When run request is clicked, run it immediately
                                        if let Some(file) = &self.selected_file {
                                            self.results_view.run_single_request(
                                                file,
                                                idx,
                                                self.selected_environment.as_deref(),
                                            );
                                        }
                                    }
                                    TextEditorAction::None => {}
                                }
                            });

                        ui.separator();

                        // Additional buttons for text editor view
                        ui.horizontal(|ui| {
                            let run_all_enabled =
                                self.selected_file.is_some() && !self.text_editor.has_changes();

                            if ui
                                .add_enabled(
                                    run_all_enabled,
                                    egui::Button::new("▶ Run All Requests"),
                                )
                                .clicked()
                                && let Some(file) = &self.selected_file
                            {
                                self.results_view
                                    .run_file(file, self.selected_environment.as_deref());
                            }

                            // Show save indicator if there are unsaved changes
                            if self.text_editor.has_changes() {
                                ui.colored_label(
                                    egui::Color32::from_rgb(255, 165, 0),
                                    "● Unsaved changes",
                                );
                            }
                        });
                    }
                    ViewMode::RequestDetails => {
                        // Wrap the request view in a scroll area with fixed height
                        egui::ScrollArea::vertical()
                            .id_salt("request_details_scroll")
                            .max_height(available_height)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                match self.request_view.show(ui, &self.selected_file) {
                                    RequestViewAction::RunRequest(idx) => {
                                        self.selected_request_index = Some(idx);
                                        // When a request button is clicked, run it immediately
                                        if let Some(file) = &self.selected_file {
                                            self.results_view.run_single_request(
                                                file,
                                                idx,
                                                self.selected_environment.as_deref(),
                                            );
                                        }
                                    }
                                    RequestViewAction::SaveFile => {
                                        // Save the file and reload both views
                                        if let Err(e) = self.request_view.save_to_file() {
                                            eprintln!("Failed to save file: {}", e);
                                        } else {
                                            // Refresh the file tree to show any new files
                                            self.file_tree =
                                                FileTree::new(self.root_directory.clone());
                                            // Reload text editor with updated content
                                            if let Some(file) = &self.selected_file {
                                                self.text_editor.load_file(file);
                                            }
                                        }
                                    }
                                    RequestViewAction::None => {}
                                }
                            });

                        ui.separator();

                        // Run buttons - always visible at bottom
                        ui.horizontal(|ui| {
                            let run_all_enabled =
                                self.selected_file.is_some() && !self.request_view.has_changes();

                            if ui
                                .add_enabled(
                                    run_all_enabled,
                                    egui::Button::new("▶ Run All Requests"),
                                )
                                .clicked()
                                && let Some(file) = &self.selected_file
                            {
                                self.results_view
                                    .run_file(file, self.selected_environment.as_deref());
                            }

                            // Show save indicator if there are unsaved changes
                            if self.request_view.has_changes() {
                                ui.colored_label(
                                    egui::Color32::from_rgb(255, 165, 0),
                                    "● Unsaved changes",
                                );
                            }
                        });
                    }
                }
            });
        });

        // Save window size if it changed (to avoid unnecessary file writes)
        let current_window_size = ctx.input(|i| i.viewport().inner_rect.map(|r| r.size()).unwrap_or(egui::vec2(1200.0, 800.0)));
        let current_size = (current_window_size.x, current_window_size.y);
        
        let should_save_window_size = match self.last_saved_window_size {
            None => true,
            Some(last_size) => last_size != current_size,
        };
        
        if should_save_window_size {
            self.last_saved_window_size = Some(current_size);
            self.save_state_with_window(ctx);
        }
    }
}
