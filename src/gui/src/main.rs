#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod file_tree;
mod request_editor;
mod request_view;
mod results_view;
mod state;

use app::HttpRunnerApp;

fn main() -> iced::Result {
    env_logger::init();
    
    iced::application(
        HttpRunnerApp::new,
        HttpRunnerApp::update,
        HttpRunnerApp::view,
    )
    .subscription(HttpRunnerApp::subscription)
    .theme(HttpRunnerApp::theme)
    .window_size((1200.0, 800.0))
    .run()
}
