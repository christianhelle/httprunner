use crate::{
    file_tree::FileTree,
    request_view::RequestView,
    results_view::ResultsView,
    state::AppState,
};
use fltk::{
    app,
    browser::HoldBrowser,
    button::Button,
    menu::Choice,
    enums::{Align, Color, Font, FrameType, Shortcut},
    frame::Frame,
    menu::{MenuBar, MenuFlag},
    prelude::*,
    text::{TextBuffer, TextDisplay, WrapMode},
    window::DoubleWindow,
};
use std::path::{Path, PathBuf};

pub struct HttpRunnerApp {
    file_tree: FileTree,
    request_view: RequestView,
    results_view: ResultsView,
    selected_file: Option<PathBuf>,
    environments: Vec<String>,
    selected_environment: Option<String>,
    root_directory: PathBuf,
    
    // UI handles
    file_browser: HoldBrowser,
    request_buffer: TextBuffer,
    results_buffer: TextBuffer,
    status_buffer: TextBuffer,
    env_choice: Choice,
    window_w: i32,
    window_h: i32,
}

impl HttpRunnerApp {
    pub fn new(wind: &mut DoubleWindow) -> Self {
        // Load saved state
        let state = AppState::load();

        // Use saved root directory or fall back to current directory
        let root_directory = state
            .root_directory
            .and_then(|p| if p.exists() { Some(p) } else { None })
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let file_tree = FileTree::new(root_directory.clone());
        let mut request_view = RequestView::new();
        let mut results_view = ResultsView::new();

        // Restore last results if available
        if let Some(last_results) = &state.last_results {
            results_view.restore_results(last_results.clone());
        }

        let selected_file = if let Some(saved_file) = &state.selected_file {
            if saved_file.exists() {
                request_view.load_file(saved_file);
                Some(saved_file.clone())
            } else {
                None
            }
        } else {
            None
        };

        let mut environments = Vec::new();
        let mut selected_environment = None;
        
        if let Some(ref file) = selected_file {
            Self::load_environments_for_file(file, &mut environments);
            if let Some(ref saved_env) = state.selected_environment {
                if environments.contains(saved_env) {
                    selected_environment = Some(saved_env.clone());
                }
            }
        }

        let window_w = wind.w();
        let window_h = wind.h();

        // Create UI
        let (file_browser, request_buffer, results_buffer, status_buffer, env_choice) =
            Self::build_ui(wind, &file_tree, &root_directory, &environments);

        let mut app = Self {
            file_tree,
            request_view,
            results_view,
            selected_file,
            environments,
            selected_environment,
            root_directory,
            file_browser,
            request_buffer,
            results_buffer,
            status_buffer,
            env_choice,
            window_w,
            window_h,
        };

        app.update_all_ui();
        app
    }

    fn build_ui(
        wind: &mut DoubleWindow,
        _file_tree: &FileTree,
        root_directory: &Path,
        environments: &[String],
    ) -> (HoldBrowser, TextBuffer, TextBuffer, TextBuffer, Choice) {
        let (s, _r) = app::channel::<Message>();

        // Menu bar
        let mut menu = MenuBar::default().with_size(wind.w(), 30);
        menu.add_emit(
            "&File/Open Directory...\t",
            Shortcut::Ctrl | 'o',
            MenuFlag::Normal,
            s,
            Message::OpenDirectory,
        );
        menu.add_emit(
            "&File/New .http File...\t",
            Shortcut::None,
            MenuFlag::Normal,
            s,
            Message::NewFile,
        );
        menu.add_emit(
            "&File/Quit\t",
            Shortcut::Ctrl | 'q',
            MenuFlag::Normal,
            s,
            Message::Quit,
        );

        // Environment choice
        let mut env_frame = Frame::default()
            .with_pos(wind.w() - 320, 5)
            .with_size(80, 20)
            .with_label("Environment:");
        env_frame.set_label_size(11);

        let mut env_choice = Choice::default()
            .with_pos(wind.w() - 230, 5)
            .with_size(220, 20);
        env_choice.add_choice("None");
        for env in environments {
            env_choice.add_choice(env);
        }
        env_choice.set_value(0);
        env_choice.emit(s, Message::SelectEnvironment);

        // File browser (left panel)
        let mut file_browser = HoldBrowser::new(5, 60, 295, wind.h() - 95, "");
        file_browser.set_color(Color::White);
        file_browser.emit(s, Message::FileSelected);

        // Request display (top right)
        let mut request_display = TextDisplay::new(305, 60, wind.w() - 310, (wind.h() - 95) / 2 - 15, "");
        request_display.set_text_font(Font::Courier);
        request_display.set_text_size(11);
        request_display.wrap_mode(WrapMode::AtBounds, 0);
        let mut request_buffer = TextBuffer::default();
        request_buffer.set_text("No file selected");
        request_display.set_buffer(request_buffer.clone());

        // Run button
        let mut run_btn = Button::new(305, 60 + (wind.h() - 95) / 2 - 10, 120, 25, "▶ Run All");
        run_btn.set_color(Color::from_rgb(76, 175, 80));
        run_btn.set_label_color(Color::White);
        run_btn.emit(s, Message::RunAllRequests);

        // Results display (bottom right)
        let mut results_display = TextDisplay::new(305, 60 + (wind.h() - 95) / 2 + 20, wind.w() - 310, (wind.h() - 95) / 2 - 20, "");
        results_display.set_text_font(Font::Courier);
        results_display.set_text_size(11);
        results_display.wrap_mode(WrapMode::AtBounds, 0);
        let mut results_buffer = TextBuffer::default();
        results_buffer.set_text("No results yet");
        results_display.set_buffer(results_buffer.clone());

        // Status bar
        let mut status_frame = Frame::new(0, wind.h() - 30, wind.w(), 30, "");
        status_frame.set_frame(FrameType::FlatBox);
        status_frame.set_color(Color::from_rgb(220, 220, 220));
        status_frame.set_align(Align::Left | Align::Inside);

        let mut status_display = TextDisplay::new(5, wind.h() - 25, wind.w() - 10, 20, "");
        status_display.set_text_size(10);
        status_display.set_frame(FrameType::NoBox);
        status_display.set_color(Color::from_rgb(220, 220, 220));
        let mut status_buffer = TextBuffer::default();
        status_buffer.set_text(&format!("Working Directory: {}", root_directory.display()));
        status_display.set_buffer(status_buffer.clone());

        (file_browser, request_buffer, results_buffer, status_buffer, env_choice)
    }

    fn update_all_ui(&mut self) {
        self.update_file_list();
        self.update_environment_selector();
        self.update_request_display();
        self.update_results_display();
        self.update_status();
    }

    fn update_file_list(&mut self) {
        self.file_browser.clear();
        let files = self.file_tree.get_files();
        for file in files {
            let display_name = file
                .strip_prefix(&self.root_directory)
                .unwrap_or(&file)
                .display()
                .to_string();
            self.file_browser.add(&format!("📄  {}", display_name));
        }
    }

    fn update_environment_selector(&mut self) {
        self.env_choice.clear();
        self.env_choice.add_choice("None");
        for env in &self.environments {
            self.env_choice.add_choice(env);
        }
        if let Some(ref env) = self.selected_environment {
            if let Some(idx) = self.environments.iter().position(|e| e == env) {
                self.env_choice.set_value((idx + 1) as i32);
            }
        } else {
            self.env_choice.set_value(0);
        }
    }

    fn update_request_display(&mut self) {
        if let Some(ref _file) = self.selected_file {
            let requests = self.request_view.get_requests();
            if requests.is_empty() {
                self.request_buffer.set_text("No requests found in file");
            } else {
                let mut text = String::new();
                for (idx, req) in requests.iter().enumerate() {
                    if idx > 0 {
                        text.push_str("\n---\n\n");
                    }
                    text.push_str(&format!("Request #{}\n", idx + 1));
                    if let Some(ref name) = req.name {
                        text.push_str(&format!("Name: {}\n", name));
                    }
                    text.push_str(&format!("Method: {}\n", req.method));
                    text.push_str(&format!("URL: {}\n", req.url));
                    if !req.headers.is_empty() {
                        text.push_str("\nHeaders:\n");
                        for header in &req.headers {
                            text.push_str(&format!("  {}: {}\n", header.name, header.value));
                        }
                    }
                    if let Some(ref body) = req.body {
                        text.push_str(&format!("\nBody:\n{}\n", body));
                    }
                }
                self.request_buffer.set_text(&text);
            }
        } else {
            self.request_buffer.set_text("No file selected");
        }
    }

    fn update_results_display(&mut self) {
        let results = self.results_view.get_results();
        if results.is_empty() {
            self.results_buffer.set_text("No results yet");
        } else {
            let mut text = String::new();
            for (idx, result) in results.iter().enumerate() {
                if idx > 0 {
                    text.push_str("\n========================================\n\n");
                }
                match result {
                    crate::results_view::ExecutionResult::Success {
                        method,
                        url,
                        status,
                        duration_ms,
                        response_body,
                        assertion_results,
                    } => {
                        text.push_str(&format!("✓ SUCCESS - {} {}\n", method, url));
                        text.push_str(&format!("Status: {}\n", status));
                        text.push_str(&format!("Duration: {}ms\n", duration_ms));
                        if !assertion_results.is_empty() {
                            text.push_str("\nAssertions:\n");
                            for assertion in assertion_results {
                                let icon = if assertion.passed { "✓" } else { "✗" };
                                let assertion_type = match assertion.assertion.assertion_type {
                                    httprunner_lib::types::AssertionType::Status => "Status",
                                    httprunner_lib::types::AssertionType::Body => "Body",
                                    httprunner_lib::types::AssertionType::Headers => "Headers",
                                };
                                let msg = if assertion.passed {
                                    assertion.assertion.expected_value.clone()
                                } else {
                                    assertion.error_message.as_deref().unwrap_or("Failed").to_string()
                                };
                                text.push_str(&format!("  {} {}: {}\n", icon, assertion_type, msg));
                            }
                        }
                        text.push_str(&format!("\nResponse:\n{}\n", response_body));
                    }
                    crate::results_view::ExecutionResult::Failure { method, url, error } => {
                        text.push_str(&format!("✗ FAILURE - {} {}\n", method, url));
                        text.push_str(&format!("Error: {}\n", error));
                    }
                    crate::results_view::ExecutionResult::Running { message } => {
                        text.push_str(&format!("⏳ {}\n", message));
                    }
                }
            }
            self.results_buffer.set_text(&text);
        }
    }

    fn update_status(&mut self) {
        let mut text = format!("Working Directory: {}", self.root_directory.display());
        if let Some(ref file) = self.selected_file {
            text.push_str(&format!(" | Selected: {}", file.display()));
        }
        self.status_buffer.set_text(&text);
    }

    fn load_environments_for_file(file: &Path, environments: &mut Vec<String>) {
        // Try to find and parse http-client.env.json
        if let Some(file_str) = file.to_str() {
            if let Ok(Some(env_file)) = httprunner_lib::environment::find_environment_file(file_str) {
                if let Ok(env_config) = httprunner_lib::environment::parse_environment_file(&env_file) {
                    let mut env_names: Vec<String> = env_config.keys().cloned().collect();
                    env_names.sort();
                    *environments = env_names;
                    return;
                }
            }
        }
        environments.clear();
    }

    pub fn save_state(&self) {
        let state = AppState {
            root_directory: Some(self.root_directory.clone()),
            selected_file: self.selected_file.clone(),
            selected_environment: self.selected_environment.clone(),
            font_size: None,
            window_size: Some((self.window_w as f32, self.window_h as f32)),
            last_results: Some(self.results_view.get_results()),
        };
        let _ = state.save();
    }

    pub fn save_state_on_exit(&mut self) {
        self.save_state();
    }

    pub fn handle_messages(&mut self, r: &app::Receiver<Message>) {
        if let Some(msg) = r.recv() {
            match msg {
                Message::OpenDirectory => {
                    if let Some(path) = native_dialog::FileDialog::new()
                        .set_location(&self.root_directory)
                        .show_open_single_dir()
                        .ok()
                        .flatten()
                    {
                        self.root_directory = path.clone();
                        self.file_tree = FileTree::new(path);
                        self.selected_file = None;
                        self.update_all_ui();
                        self.save_state();
                    }
                }
                Message::NewFile => {
                    if let Some(path) = native_dialog::FileDialog::new()
                        .set_location(&self.root_directory)
                        .add_filter("HTTP Files", &["http"])
                        .set_filename("new.http")
                        .show_save_single_file()
                        .ok()
                        .flatten()
                    {
                        if std::fs::write(&path, "### New Request\nGET https://httpbin.org/get\n").is_ok() {
                            self.file_tree = FileTree::new(self.root_directory.clone());
                            self.selected_file = Some(path.clone());
                            self.request_view.load_file(&path);
                            Self::load_environments_for_file(&path, &mut self.environments);
                            self.update_all_ui();
                            self.save_state();
                        }
                    }
                }
                Message::Quit => {
                    self.save_state();
                    app::quit();
                }
                Message::SelectEnvironment => {
                    if let Some(text) = self.env_choice.choice() {
                        self.selected_environment = if text == "None" {
                            None
                        } else {
                            Some(text)
                        };
                        self.save_state();
                    }
                }
                Message::FileSelected => {
                    let idx = self.file_browser.value();
                    if idx > 0 {
                        let files = self.file_tree.get_files();
                        if let Some(file) = files.get((idx - 1) as usize) {
                            self.selected_file = Some(file.clone());
                            self.request_view.load_file(file);
                            Self::load_environments_for_file(file, &mut self.environments);
                            self.update_all_ui();
                            self.save_state();
                        }
                    }
                }
                Message::RunAllRequests => {
                    if let Some(ref file) = self.selected_file {
                        self.results_view.run_file(file, self.selected_environment.as_deref());
                        // Update results display to show initial "Running" message
                        self.update_results_display();
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    OpenDirectory,
    NewFile,
    Quit,
    SelectEnvironment,
    RunAllRequests,
    FileSelected,
}

impl HttpRunnerApp {
    pub fn channel() -> (app::Sender<Message>, app::Receiver<Message>) {
        app::channel::<Message>()
    }
}
