mod app;
mod file_tree;
mod request_view;
mod results_view;

use app::HttpRunnerApp;

fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    // Load the application icon
    let icon = load_icon();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(icon),
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
    let icon_data = eframe::icon_data::from_png_bytes(icon_bytes)
        .expect("Failed to parse PNG icon data from embedded icon.png bytes");
    std::sync::Arc::new(icon_data)
}
