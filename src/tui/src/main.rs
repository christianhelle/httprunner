mod app;
mod file_tree;
mod request_view;
mod results_view;
mod state;
mod ui;

use app::App;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use httprunner_core::telemetry::{self, AppType};
use ratatui::{Terminal, backend::CrosstermBackend};
use state::AppState;
use std::io;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const INSTRUMENTATION_KEY: &str = "a7a07a35-4869-4fa2-b852-03f44b35f418";

fn main() -> anyhow::Result<()> {
    // Load saved state to check telemetry preference
    let saved_state = AppState::load();
    let telemetry_disabled = saved_state.telemetry_enabled == Some(false);

    // Initialize telemetry (respects stored preference)
    telemetry::init(
        AppType::TUI,
        VERSION,
        telemetry_disabled,
        INSTRUMENTATION_KEY,
    );

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new()?;

    // Run app
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Track error if app failed
    if let Err(ref err) = res {
        telemetry::track_error(err.as_ref());
        eprintln!("Error: {}", err);
    }

    // Flush telemetry before exit
    telemetry::flush();

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(());
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(());
                }
                _ => {
                    app.handle_key_event(key)?;
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
