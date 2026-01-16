#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_fltk;
mod file_tree;
mod request_editor;
mod request_view;
mod results_view;
mod state;

use app_fltk::HttpRunnerApp;
use fltk::{app, prelude::*, window::DoubleWindow};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Load saved state to restore window size
    let saved_state = state::AppState::load();
    let (width, height) = saved_state.window_size.unwrap_or((1200.0, 800.0));

    let fltk_app = app::App::default();
    fltk_theme::WidgetTheme::new(fltk_theme::ThemeType::Greybird).apply();

    let mut window = DoubleWindow::default()
        .with_size(width as i32, height as i32)
        .with_label("HTTP File Runner");

    // Load icon if available
    if let Ok(icon) = load_icon() {
        window.set_icon(Some(icon));
    }

    let (_s, r) = HttpRunnerApp::channel();
    let mut app_state = HttpRunnerApp::new(&mut window);

    window.end();
    window.show();

    // Main event loop
    while fltk_app.wait() {
        app_state.handle_messages(&r);
    }

    app_state.save_state_on_exit();
    Ok(())
}

fn load_icon() -> Result<fltk::image::PngImage, Box<dyn std::error::Error>> {
    let icon_bytes = include_bytes!("../../../images/icon.png");
    let mut icon = fltk::image::PngImage::from_data(icon_bytes)?;
    icon.scale(32, 32, true, true);
    Ok(icon)
}
