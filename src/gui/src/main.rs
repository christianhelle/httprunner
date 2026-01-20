#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Modules are used by binary, not lib directly
#![allow(dead_code)]

mod app;
mod file_tree;
mod request_editor;
mod request_view;
mod results_view;
mod state;
mod text_editor;

#[cfg(target_arch = "wasm32")]
mod results_view_async;

#[cfg(not(target_arch = "wasm32"))]
use app::HttpRunnerApp;

// Native binary entry point
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    // Load saved state to restore window size
    let saved_state = state::AppState::load();
    let window_size = saved_state
        .window_size
        .map(|(w, h)| [w, h])
        .unwrap_or([1200.0, 800.0]);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(window_size)
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

#[cfg(not(target_arch = "wasm32"))]
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

// WASM entry point - empty since lib.rs handles WASM
#[cfg(target_arch = "wasm32")]
fn main() {
    // WASM entry point is in lib.rs
}
