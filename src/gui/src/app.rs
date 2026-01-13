use super::{file_tree::FileTree, request_view::RequestView, results_view::ResultsView};
use std::path::PathBuf;

pub struct HttpRunnerApp {
    file_tree: FileTree,
    request_view: RequestView,
    results_view: ResultsView,
    selected_file: Option<PathBuf>,
    selected_request_index: Option<usize>,
    environments: Vec<String>,
    selected_environment: Option<String>,
    root_directory: PathBuf,
    font_size: f32,
}

impl HttpRunnerApp {
    const DEFAULT_FONT_SIZE: f32 = 14.0;
    const MIN_FONT_SIZE: f32 = 8.0;
    const MAX_FONT_SIZE: f32 = 32.0;
    const FONT_SIZE_STEP: f32 = 1.0;

    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Start with current directory
        let root_directory = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        Self {
            file_tree: FileTree::new(root_directory.clone()),
            request_view: RequestView::new(),
            results_view: ResultsView::new(),
            selected_file: None,
            selected_request_index: None,
            environments: Vec::new(),
            selected_environment: None,
            root_directory,
            font_size: Self::DEFAULT_FONT_SIZE,
        }
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

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        // Check for Ctrl + Plus (zoom in)
        if ctx.input(|i| {
            i.modifiers.ctrl && (i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals))
        }) {
            self.font_size = (self.font_size + Self::FONT_SIZE_STEP).min(Self::MAX_FONT_SIZE);
            self.update_font_size(ctx);
        }

        // Check for Ctrl + Minus (zoom out)
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Minus)) {
            self.font_size = (self.font_size - Self::FONT_SIZE_STEP).max(Self::MIN_FONT_SIZE);
            self.update_font_size(ctx);
        }

        // Check for Ctrl + 0 (reset to default)
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Num0)) {
            self.font_size = Self::DEFAULT_FONT_SIZE;
            self.update_font_size(ctx);
        }
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
                        }
                        ui.close();
                    }

                    ui.separator();

                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.separator();

                ui.label("Environment:");
                egui::ComboBox::from_id_salt("env_selector")
                    .selected_text(self.selected_environment.as_deref().unwrap_or("None"))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_environment, None, "None");
                        for env in &self.environments {
                            let env_clone = Some(env.clone());
                            ui.selectable_value(&mut self.selected_environment, env_clone, env);
                        }
                    });
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

    fn load_environments(&mut self, file: &PathBuf) {
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
}

impl eframe::App for HttpRunnerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keyboard_shortcuts(ctx);
        self.show_top_panel(ctx);
        self.show_bottom_panel(ctx);

        // Left panel - File tree
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

                    // Update request view
                    self.request_view.load_file(&selected);
                }
            });

        // Center panel - Split into Request Details (top, resizable) and Results (bottom)
        egui::CentralPanel::default().show(ctx, |ui| {
            // Top section - Request Details
            egui::TopBottomPanel::top("request_details_panel")
                .resizable(true)
                .default_height(400.0)
                .min_height(200.0)
                .show_inside(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.heading("Request Details");
                        ui.separator();

                        // Use available space minus the button area
                        let available_height = ui.available_height() - 40.0; // Reserve space for buttons

                        // Wrap the request view in a scroll area with fixed height
                        egui::ScrollArea::vertical()
                            .id_salt("request_details_scroll")
                            .max_height(available_height)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                if let Some(selected_idx) =
                                    self.request_view.show(ui, &self.selected_file)
                                {
                                    self.selected_request_index = Some(selected_idx);
                                }
                            });

                        ui.separator();

                        // Run buttons - always visible at bottom
                        ui.horizontal(|ui| {
                            let run_all_enabled = self.selected_file.is_some();
                            let run_one_enabled = self.selected_file.is_some()
                                && self.selected_request_index.is_some();

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

                            if ui
                                .add_enabled(
                                    run_one_enabled,
                                    egui::Button::new("▶ Run Selected Request"),
                                )
                                .clicked()
                                && let (Some(file), Some(idx)) =
                                    (&self.selected_file, self.selected_request_index)
                            {
                                self.results_view.run_single_request(
                                    file,
                                    idx,
                                    self.selected_environment.as_deref(),
                                );
                            }
                        });
                    });
                });

            // Bottom section - Results
            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Results");
                    ui.separator();

                    // Reserve all remaining space for results scroll area
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
            });
        });
    }
}
