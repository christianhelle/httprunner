#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod file_tree;
mod request_editor;
mod request_view;
mod results_view;
mod state;

use app::HttpRunnerApp;

fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "HTTP File Runner",
        native_options,
        Box::new(|cc| Ok(Box::new(HttpRunnerApp::new(cc)))),
    )
}

fn load_icon() -> std::sync::Arc<egui::IconData> {
    let icon_bytes = include_bytes!("../../../images/icon.png");
    match eframe::icon_data::from_png_bytes(icon_bytes) {
        Ok(icon_data) => std::sync::Arc::new(icon_data),
        Err(e) => {
            eprintln!("Warning: Failed to load application icon: {}", e);
            // Return default icon data (empty) which will use the egui default
            std::sync::Arc::new(egui::IconData::default())
        }
    }
}
