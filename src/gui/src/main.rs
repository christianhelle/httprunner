#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod file_tree;
mod request_editor;
mod request_view;
mod results_view;
mod state;

fn main() -> iced::Result {
    env_logger::init();
    
    iced::run("HTTP File Runner", app::HttpRunnerApp::update, app::HttpRunnerApp::view)
}
