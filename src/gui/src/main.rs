#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

mod app;
mod environment_editor;
mod file_tree;
mod request_editor;
mod request_view;
mod results_view;
mod state;
mod text_editor;

use httprunner_core::telemetry::{self, AppType};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const INSTRUMENTATION_KEY: &str = "a7a07a35-4869-4fa2-b852-03f44b35f418";

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

    let (w, h) = saved_state.window_size.unwrap_or((1200.0, 800.0));

    let cfg = dioxus::desktop::Config::default().with_window(
        dioxus::desktop::WindowBuilder::new()
            .with_title("HTTP File Runner")
            .with_inner_size(dioxus::desktop::LogicalSize::new(w as f64, h as f64))
            .with_min_inner_size(dioxus::desktop::LogicalSize::new(800.0_f64, 600.0_f64)),
    );

    dioxus::LaunchBuilder::desktop().with_cfg(cfg).launch(app::App);

    telemetry::flush();
}
