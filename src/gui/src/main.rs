#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod environment_editor;
mod file_tree;
mod request_view;
mod results_view;
mod state;
mod text_editor;

use app::HttpRunnerApp;
use httprunner_core::telemetry::{self, AppType};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const INSTRUMENTATION_KEY: &str = "a7a07a35-4869-4fa2-b852-03f44b35f418";

fn main() -> iced::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    // Load saved state to restore window size and check telemetry preference
    let saved_state = state::AppState::load();
    let telemetry_disabled = saved_state.telemetry_enabled == Some(false);

    // Initialize telemetry (respects stored preference)
    telemetry::init(
        AppType::GUI,
        VERSION,
        telemetry_disabled,
        INSTRUMENTATION_KEY,
    );

    let result = iced::application(
        HttpRunnerApp::new,
        HttpRunnerApp::update,
        HttpRunnerApp::view,
    )
    .run();

    // Flush telemetry before exit
    telemetry::flush();

    result
}
