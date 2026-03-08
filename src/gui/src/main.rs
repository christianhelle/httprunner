#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod environment_editor;
mod file_tree;
mod request_editor;
mod results_view;
mod state;
mod text_editor;

#[cfg(target_arch = "wasm32")]
mod results_view_async;

#[cfg(not(target_arch = "wasm32"))]
use dioxus::LaunchBuilder;
#[cfg(not(target_arch = "wasm32"))]
use dioxus_desktop::{Config, LogicalSize, WindowBuilder};
#[cfg(not(target_arch = "wasm32"))]
use httprunner_core::telemetry::{self, AppType};

#[cfg(not(target_arch = "wasm32"))]
const VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg(not(target_arch = "wasm32"))]
const INSTRUMENTATION_KEY: &str = "a7a07a35-4869-4fa2-b852-03f44b35f418";

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    env_logger::init();

    let saved_state = state::AppState::load();
    let telemetry_disabled = saved_state.telemetry_enabled == Some(false);
    telemetry::init(
        AppType::GUI,
        VERSION,
        telemetry_disabled,
        INSTRUMENTATION_KEY,
    );

    let window_size = saved_state.window_size.unwrap_or((1200.0, 800.0));
    let data_directory = dirs::data_local_dir()
        .or_else(dirs::config_dir)
        .unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        })
        .join("httprunner");

    let mut config = Config::new()
        .with_data_directory(data_directory)
        .with_window(
            WindowBuilder::new()
                .with_title("HTTP File Runner")
                .with_inner_size(LogicalSize::new(window_size.0 as f64, window_size.1 as f64))
                .with_min_inner_size(LogicalSize::new(800.0, 600.0)),
        )
        .with_background_color((17, 24, 39, 255));

    if let Some(icon) = load_icon() {
        config = config.with_icon(icon);
    }

    LaunchBuilder::new().with_cfg(config).launch(app::app);

    telemetry::flush();
}

#[cfg(not(target_arch = "wasm32"))]
fn load_icon() -> Option<dioxus_desktop::tao::window::Icon> {
    let icon_bytes = include_bytes!("../../../images/icon.png");
    let image = match image::load_from_memory(icon_bytes) {
        Ok(image) => image.to_rgba8(),
        Err(error) => {
            eprintln!("Warning: Failed to load application icon: {}", error);
            return None;
        }
    };

    let (width, height) = image.dimensions();
    dioxus_desktop::tao::window::Icon::from_rgba(image.into_raw(), width, height).ok()
}

#[cfg(target_arch = "wasm32")]
fn main() {
    panic!(
        "This binary is not meant to be run directly for wasm32 targets.\n\
         The WebAssembly entry point is defined in lib.rs and should be loaded via \
         a WASM bundler (e.g. `trunk serve` or `trunk build`), not via `cargo run`."
    );
}
