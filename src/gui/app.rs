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
}

impl HttpRunnerApp {
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
        }
    }

    fn show_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Directory...").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.root_directory = path.clone();
                            self.file_tree = FileTree::new(path);
                            self.selected_file = None;
                            self.selected_request_index = None;
                        }
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.separator();

                ui.label("Environment:");
                egui::ComboBox::from_id_salt("env_selector")
                    .selected_text(
                        self.selected_environment
                            .as_ref()
                            .map(|s| s.as_str())
                            .unwrap_or("None"),
                    )
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
        if let Some(file_str) = file.to_str() {
            if let Ok(Some(env_file)) = httprunner::environment::find_environment_file(file_str) {
                if let Ok(env_config) = httprunner::environment::parse_environment_file(&env_file) {
                    // Extract environment names from the config
                    self.environments = env_config.keys().cloned().collect();
                    self.environments.sort(); // Sort alphabetically for consistent UI
                    return;
                }
            }
        }
        // No environments found or error occurred
        self.environments = Vec::new();
    }
}

impl eframe::App for HttpRunnerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

        // Center panel - Request details
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Request Details");
            ui.separator();

            if let Some(selected_idx) = self.request_view.show(ui, &self.selected_file) {
                self.selected_request_index = Some(selected_idx);
            }

            ui.separator();

            // Run button
            ui.horizontal(|ui| {
                let run_all_enabled = self.selected_file.is_some();
                let run_one_enabled =
                    self.selected_file.is_some() && self.selected_request_index.is_some();

                if ui
                    .add_enabled(run_all_enabled, egui::Button::new("▶ Run All Requests"))
                    .clicked()
                {
                    if let Some(file) = &self.selected_file {
                        self.results_view
                            .run_file(file, self.selected_environment.as_deref());
                    }
                }

                if ui
                    .add_enabled(run_one_enabled, egui::Button::new("▶ Run Selected Request"))
                    .clicked()
                {
                    if let (Some(file), Some(idx)) =
                        (&self.selected_file, self.selected_request_index)
                    {
                        self.results_view.run_single_request(
                            file,
                            idx,
                            self.selected_environment.as_deref(),
                        );
                    }
                }
            });
        });

        // Right panel - Results
        egui::SidePanel::right("results_panel")
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Results");
                ui.separator();
                self.results_view.show(ui);
            });
    }
}
