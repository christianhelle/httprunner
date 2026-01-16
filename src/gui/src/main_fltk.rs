mod app;
mod file_tree;
mod request_editor;
mod request_view;
mod results_view;
mod state;

use app::HttpRunnerApp;
use fltk::{app as fltk_app, prelude::*, window::Window};

fn main() {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    // Load saved state to restore window size
    let saved_state = state::AppState::load();
    let window_size = saved_state
        .window_size
        .map(|(w, h)| (w as i32, h as i32))
        .unwrap_or((1200, 800));

    let app = fltk_app::App::default();
    
    // Apply a modern theme
    let widget_theme = fltk_theme::WidgetTheme::new(fltk_theme::ThemeType::Aero);
    widget_theme.apply();
    
    // Create main window
    let mut wind = Window::default()
        .with_size(window_size.0, window_size.1)
        .with_label("HTTP File Runner");
    
    // Center the window on screen
    wind.make_resizable(true);
    
    // Create and setup the application
    let mut http_app = HttpRunnerApp::new(&mut wind);
    
    wind.end();
    wind.show();
    
    // Run the application
    app.run().unwrap();
    
    // Save state on exit
    http_app.save_state_on_exit();
}
