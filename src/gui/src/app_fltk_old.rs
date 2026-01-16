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
    enums::{Align, Color, Event, Font, FrameType, Key, Shortcut},
    frame::Frame,
    group::{Pack, PackType, Scroll, Tile},
    menu::{MenuBar, MenuFlag},
    prelude::*,
    text::{TextBuffer, TextDisplay, WrapMode},
    window::Window,
};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
enum Message {
    OpenDirectory,
    NewFile,
    Quit,
    SelectEnvironment,
    RunAllRequests,
    FileSelected,
    RunRequest,
    SaveFile,
}

pub struct HttpRunnerApp {
    file_tree: Rc<RefCell<FileTree>>,
    request_view: Rc<RefCell<RequestView>>,
    results_view: Rc<RefCell<ResultsView>>,
    selected_file: Rc<RefCell<Option<PathBuf>>>,
    selected_request_index: Rc<RefCell<Option<usize>>>,
    environments: Rc<RefCell<Vec<String>>>,
    selected_environment: Rc<RefCell<Option<String>>>,
    root_directory: Rc<RefCell<PathBuf>>,
    
    // UI Components that need to be updated
    file_browser: Rc<RefCell<HoldBrowser>>,
    request_display: Rc<RefCell<TextDisplay>>,
    request_buffer: Rc<RefCell<TextBuffer>>,
    results_display: Rc<RefCell<TextDisplay>>,
    results_buffer: Rc<RefCell<TextBuffer>>,
    status_buffer: Rc<RefCell<TextBuffer>>,
    env_choice: Rc<RefCell<Choice>>,
    window: Rc<RefCell<Window>>,
}

impl HttpRunnerApp {
    pub fn new(wind: &mut Window) -> Self {
        // Load saved state
        let state = AppState::load();

        // Use saved root directory or fall back to current directory
        let root_directory = state
            .root_directory
            .and_then(|p| if p.exists() { Some(p) } else { None })
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let root_directory = Rc::new(RefCell::new(root_directory.clone()));
        let selected_file = Rc::new(RefCell::new(None));
        let selected_request_index = Rc::new(RefCell::new(None));
        let environments = Rc::new(RefCell::new(Vec::new()));
        let selected_environment = Rc::new(RefCell::new(None));

        let file_tree = Rc::new(RefCell::new(FileTree::new(
            root_directory.borrow().clone(),
        )));
        let request_view = Rc::new(RefCell::new(RequestView::new()));
        let results_view = Rc::new(RefCell::new(ResultsView::new()));

        // Restore last results if available
        if let Some(last_results) = state.last_results {
            results_view.borrow_mut().restore_results(last_results);
        }

        // Restore selected file if it still exists
        if let Some(saved_file) = state.selected_file {
            if saved_file.exists() {
                *selected_file.borrow_mut() = Some(saved_file.clone());
                Self::load_environments_static(&saved_file, &environments);
                request_view.borrow_mut().load_file(&saved_file);

                // Restore selected environment if it's still valid
                if let Some(saved_env) = state.selected_environment {
                    if environments.borrow().contains(&saved_env) {
                        *selected_environment.borrow_mut() = Some(saved_env);
                    }
                }
            }
        }

        // Create UI components that will be shared
        let file_browser = Rc::new(RefCell::new(HoldBrowser::default()));
        let request_buffer = Rc::new(RefCell::new(TextBuffer::default()));
        let request_display = Rc::new(RefCell::new(TextDisplay::default()));
        let results_buffer = Rc::new(RefCell::new(TextBuffer::default()));
        let results_display = Rc::new(RefCell::new(TextDisplay::default()));
        let status_buffer = Rc::new(RefCell::new(TextBuffer::default()));
        let env_choice = Rc::new(RefCell::new(Choice::default()));
        let window = Rc::new(RefCell::new(wind.clone()));

        let mut app = Self {
            file_tree,
            request_view,
            results_view,
            selected_file,
            selected_request_index,
            environments,
            selected_environment,
            root_directory,
            file_browser,
            request_display,
            request_buffer,
            results_display,
            results_buffer,
            status_buffer,
            env_choice,
            window,
        };

        app.build_ui(wind);
        app.update_file_list();
        app.update_environment_selector();
        app.update_status();
        
        // Restore UI state
        if let Some(ref file) = *app.selected_file.borrow() {
            app.update_request_display();
        }
        if let Some(ref results) = state.last_results {
            if !results.is_empty() {
                app.update_results_display();
            }
        }

        app
    }

    fn build_ui(&mut self, wind: &mut Window) {
        let (s, r) = app::channel::<Message>();

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

        // Environment selector in menu bar (positioned to the right side)
        let mut env_frame = Frame::default()
            .with_pos(wind.w() - 320, 5)
            .with_size(80, 20)
            .with_label("Environment:");
        env_frame.set_label_size(12);

        let mut env_choice = Choice::default()
            .with_pos(wind.w() - 230, 5)
            .with_size(220, 20);
        env_choice.emit(s, Message::SelectEnvironment);
        *self.env_choice.borrow_mut() = env_choice;

        // Main layout using Tile for resizable panels
        let mut tile = Tile::new(0, 30, wind.w(), wind.h() - 60, "");
        
        // Left panel - File tree (300px initial width)
        let mut file_scroll = Scroll::new(0, 30, 300, wind.h() - 60, "");
        file_scroll.set_color(Color::from_rgb(245, 245, 245));
        
        let mut file_frame = Frame::default()
            .with_pos(5, 35)
            .with_size(290, 25)
            .with_label("HTTP Files");
        file_frame.set_align(Align::Left | Align::Inside);
        file_frame.set_label_font(Font::HelveticaBold);
        
        let mut file_browser = HoldBrowser::new(5, 65, 290, wind.h() - 100, "");
        file_browser.set_color(Color::White);
        file_browser.emit(s, Message::FileSelected);
        *self.file_browser.borrow_mut() = file_browser;
        
        file_scroll.end();
        
        // Right side - vertical tile for request and results
        let mut right_tile = Tile::new(300, 30, wind.w() - 300, wind.h() - 60, "");
        
        // Top panel - Request details
        let mut request_scroll = Scroll::new(300, 30, wind.w() - 300, (wind.h() - 60) / 2, "");
        request_scroll.set_color(Color::from_rgb(250, 250, 250));
        
        let mut request_frame = Frame::default()
            .with_pos(305, 35)
            .with_size(wind.w() - 310, 25)
            .with_label("Request Details");
        request_frame.set_align(Align::Left | Align::Inside);
        request_frame.set_label_font(Font::HelveticaBold);
        
        let mut request_display = TextDisplay::new(305, 65, wind.w() - 310, (wind.h() - 60) / 2 - 80, "");
        request_display.set_text_font(Font::Courier);
        request_display.set_text_size(12);
        request_display.wrap_mode(WrapMode::AtBounds, 0);
        
        let mut request_buffer = TextBuffer::default();
        request_buffer.set_text("No file selected. Select a .http file from the left panel.");
        request_display.set_buffer(request_buffer.clone());
        
        *self.request_display.borrow_mut() = request_display;
        *self.request_buffer.borrow_mut() = request_buffer;
        
        // Buttons row
        let mut btn_pack = Pack::new(305, (wind.h() - 60) / 2 - 35, wind.w() - 310, 30, "");
        btn_pack.set_type(PackType::Horizontal);
        btn_pack.set_spacing(5);
        
        let mut run_all_btn = Button::default()
            .with_size(140, 25)
            .with_label("▶ Run All Requests");
        run_all_btn.set_color(Color::from_rgb(76, 175, 80));
        run_all_btn.set_label_color(Color::White);
        run_all_btn.emit(s, Message::RunAllRequests);
        
        let mut save_btn = Button::default()
            .with_size(100, 25)
            .with_label("💾 Save");
        save_btn.emit(s, Message::SaveFile);
        
        btn_pack.end();
        request_scroll.end();
        
        // Bottom panel - Results
        let mut results_scroll = Scroll::new(300, 30 + (wind.h() - 60) / 2, wind.w() - 300, (wind.h() - 60) / 2, "");
        results_scroll.set_color(Color::from_rgb(245, 245, 245));
        
        let mut results_frame = Frame::default()
            .with_pos(305, 35 + (wind.h() - 60) / 2)
            .with_size(wind.w() - 310, 25)
            .with_label("Results");
        results_frame.set_align(Align::Left | Align::Inside);
        results_frame.set_label_font(Font::HelveticaBold);
        
        let mut results_display = TextDisplay::new(305, 65 + (wind.h() - 60) / 2, wind.w() - 310, (wind.h() - 60) / 2 - 35, "");
        results_display.set_text_font(Font::Courier);
        results_display.set_text_size(12);
        results_display.wrap_mode(WrapMode::AtBounds, 0);
        
        let mut results_buffer = TextBuffer::default();
        results_buffer.set_text("No results yet. Run a request to see results here.");
        results_display.set_buffer(results_buffer.clone());
        
        *self.results_display.borrow_mut() = results_display;
        *self.results_buffer.borrow_mut() = results_buffer;
        
        results_scroll.end();
        right_tile.end();
        tile.end();
        
        // Bottom status bar
        let mut status_frame = Frame::new(0, wind.h() - 30, wind.w(), 30, "");
        status_frame.set_frame(FrameType::FlatBox);
        status_frame.set_color(Color::from_rgb(220, 220, 220));
        status_frame.set_align(Align::Left | Align::Inside);
        
        let mut status_text = TextDisplay::new(5, wind.h() - 25, wind.w() - 10, 20, "");
        status_text.set_text_size(11);
        status_text.set_frame(FrameType::NoBox);
        status_text.set_color(Color::from_rgb(220, 220, 220));
        
        let mut status_buffer = TextBuffer::default();
        status_buffer.set_text(&format!("Working Directory: {}", self.root_directory.borrow().display()));
        status_text.set_buffer(status_buffer.clone());
        *self.status_buffer.borrow_mut() = status_buffer;

        // Handle keyboard shortcuts
        let wind_clone = self.window.clone();
        let sender = s.clone();
        wind.handle(move |_, ev| match ev {
            Event::KeyDown => {
                if app::event_key() == Key::F5 {
                    sender.send(Message::RunAllRequests);
                    true
                } else {
                    false
                }
            }
            _ => false,
        });

        // Set up message handler
        self.setup_message_handler(r);
    }

    fn setup_message_handler(&self, r: app::Receiver<Message>) {
        let root_dir_rc = self.root_directory.clone();
        let file_tree_rc = self.file_tree.clone();
        let request_view_rc = self.request_view.clone();
        let results_view_rc = self.results_view.clone();
        let selected_file_rc = self.selected_file.clone();
        let selected_env_rc = self.selected_environment.clone();
        let environments_rc = self.environments.clone();
        let selected_request_index_rc = self.selected_request_index.clone();
        let file_browser_rc = self.file_browser.clone();
        let request_buffer_rc = self.request_buffer.clone();
        let results_buffer_rc = self.results_buffer.clone();
        let status_buffer_rc = self.status_buffer.clone();
        let env_choice_rc = self.env_choice.clone();
        let window_rc = self.window.clone();

        let app_clone = AppClone {
            root_directory: root_dir_rc.clone(),
            file_tree: file_tree_rc.clone(),
            request_view: request_view_rc.clone(),
            results_view: results_view_rc.clone(),
            selected_file: selected_file_rc.clone(),
            selected_environment: selected_env_rc.clone(),
            environments: environments_rc.clone(),
            selected_request_index: selected_request_index_rc.clone(),
            file_browser: file_browser_rc.clone(),
            request_buffer: request_buffer_rc.clone(),
            results_buffer: results_buffer_rc.clone(),
            status_buffer: status_buffer_rc.clone(),
            env_choice: env_choice_rc.clone(),
            window: window_rc.clone(),
        };

        // Process messages in event loop
        std::thread::spawn(move || {
            while app::wait() {
                if let Some(msg) = r.recv() {
                    app_clone.handle_message(msg);
                }
            }
        });
    }

    fn update_file_list(&self) {
        let mut browser = self.file_browser.borrow_mut();
        browser.clear();
        
        let files = self.file_tree.borrow().get_files();
        for file in files {
            let display_name = file
                .strip_prefix(&*self.root_directory.borrow())
                .unwrap_or(&file)
                .display()
                .to_string();
            browser.add(&format!("📄  {}", display_name));
        }
    }

    fn update_environment_selector(&self) {
        let mut choice = self.env_choice.borrow_mut();
        choice.clear();
        choice.add("None");
        
        for env in self.environments.borrow().iter() {
            choice.add(env);
        }
        
        // Set selected value
        if let Some(ref env) = *self.selected_environment.borrow() {
            if let Some(idx) = self.environments.borrow().iter().position(|e| e == env) {
                choice.set_value((idx + 1) as i32);
            } else {
                choice.set_value(0);
            }
        } else {
            choice.set_value(0);
        }
    }

    fn update_request_display(&self) {
        let mut buffer = self.request_buffer.borrow_mut();
        
        if let Some(ref file) = *self.selected_file.borrow() {
            let requests = self.request_view.borrow().get_requests();
            
            if requests.is_empty() {
                buffer.set_text("No requests found in this file.");
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
                buffer.set_text(&text);
            }
        } else {
            buffer.set_text("No file selected. Select a .http file from the left panel.");
        }
    }

    fn update_results_display(&self) {
        let mut buffer = self.results_buffer.borrow_mut();
        let results = self.results_view.borrow().get_results();
        
        if results.is_empty() {
            buffer.set_text("No results yet. Run a request to see results here.");
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
                                let status_icon = if assertion.passed { "✓" } else { "✗" };
                                let assertion_type = match assertion.assertion.assertion_type { httprunner_lib::types::AssertionType::Status => "Status", httprunner_lib::types::AssertionType::Body => "Body", httprunner_lib::types::AssertionType::Headers => "Headers", }; let msg = if assertion.passed { assertion.assertion.expected_value.clone() } else { assertion.error_message.as_deref().unwrap_or("Failed").to_string() }; text.push_str(&format!("  {} {}: {}\n", status_icon, assertion_type, msg));
                            }
                        }
                        
                        text.push_str(&format!("\nResponse Body:\n{}\n", response_body));
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
            buffer.set_text(&text);
        }
        
        // Scroll to bottom
        if let Ok(display) = self.results_display.try_borrow() {
            display.scroll(display.count_lines(0, buffer.length(), true), 0);
        }
    }

    fn update_status(&self) {
        let mut buffer = self.status_buffer.borrow_mut();
        let mut text = format!("Working Directory: {}", self.root_directory.borrow().display());
        
        if let Some(ref file) = *self.selected_file.borrow() {
            text.push_str(&format!(" | Selected: {}", file.display()));
        }
        
        buffer.set_text(&text);
    }

    fn load_environments_static(file: &Path, environments: &Rc<RefCell<Vec<String>>>) {
        // Try to find and parse http-client.env.json
        if let Some(file_str) = file.to_str()
            && let Ok(Some(env_file)) = httprunner_lib::environment::find_environment_file(file_str)
            && let Ok(env_config) = httprunner_lib::environment::parse_environment_file(&env_file)
        {
            // Extract environment names from the config
            let mut env_names: Vec<String> = env_config.keys().cloned().collect();
            env_names.sort(); // Sort alphabetically for consistent UI
            *environments.borrow_mut() = env_names;
            return;
        }
        // No environments found or error occurred
        environments.borrow_mut().clear();
    }

    pub fn save_state(&self) {
        let state = AppState {
            root_directory: Some(self.root_directory.borrow().clone()),
            selected_file: self.selected_file.borrow().clone(),
            selected_environment: self.selected_environment.borrow().clone(),
            font_size: None,
            window_size: Some((
                self.window.borrow().w() as f32,
                self.window.borrow().h() as f32,
            )),
            last_results: Some(self.results_view.borrow().get_results()),
        };

        if let Err(e) = state.save() {
            eprintln!("Failed to save application state: {}", e);
        }
    }

    pub fn save_state_on_exit(&mut self) {
        self.save_state();
    }

    pub fn get_handle(&self) -> AppHandle {
        AppHandle {
            root_directory: self.root_directory.clone(),
            selected_file: self.selected_file.clone(),
            selected_environment: self.selected_environment.clone(),
            window: self.window.clone(),
            results_view: self.results_view.clone(),
        }
    }
}

// Helper struct for message handling
struct AppClone {
    root_directory: Rc<RefCell<PathBuf>>,
    file_tree: Rc<RefCell<FileTree>>,
    request_view: Rc<RefCell<RequestView>>,
    results_view: Rc<RefCell<ResultsView>>,
    selected_file: Rc<RefCell<Option<PathBuf>>>,
    selected_environment: Rc<RefCell<Option<String>>>,
    environments: Rc<RefCell<Vec<String>>>,
    selected_request_index: Rc<RefCell<Option<usize>>>,
    file_browser: Rc<RefCell<HoldBrowser>>,
    request_buffer: Rc<RefCell<TextBuffer>>,
    results_buffer: Rc<RefCell<TextBuffer>>,
    status_buffer: Rc<RefCell<TextBuffer>>,
    env_choice: Rc<RefCell<Choice>>,
    window: Rc<RefCell<Window>>,
}

impl AppClone {
    fn handle_message(&self, msg: Message) {
        match msg {
            Message::OpenDirectory => {
                if let Some(path) = native_dialog::FileDialog::new()
                    .set_location(&*self.root_directory.borrow())
                    .show_open_single_dir()
                    .ok()
                    .flatten()
                {
                    *self.root_directory.borrow_mut() = path.clone();
                    *self.file_tree.borrow_mut() = FileTree::new(path);
                    *self.selected_file.borrow_mut() = None;
                    *self.selected_request_index.borrow_mut() = None;
                    
                    self.update_file_list();
                    self.update_request_display();
                    self.update_status();
                    self.save_state();
                }
            }
            Message::NewFile => {
                if let Some(path) = native_dialog::FileDialog::new()
                    .set_location(&*self.root_directory.borrow())
                    .add_filter("HTTP Files", &["http"])
                    .set_filename("new.http")
                    .show_save_single_file()
                    .ok()
                    .flatten()
                {
                    if std::fs::write(&path, "### New Request\nGET https://httpbin.org/get\n").is_ok() {
                        *self.file_tree.borrow_mut() = FileTree::new(self.root_directory.borrow().clone());
                        *self.selected_file.borrow_mut() = Some(path.clone());
                        self.request_view.borrow_mut().load_file(&path);
                        
                        Self::load_environments_static(&path, &self.environments);
                        self.update_file_list();
                        self.update_environment_selector();
                        self.update_request_display();
                        self.update_status();
                        self.save_state();
                    }
                }
            }
            Message::Quit => {
                self.save_state();
                app::quit();
            }
            Message::SelectEnvironment => {
                let choice = self.env_choice.borrow();
                if let Some(text) = choice.choice() {
                    if text == "None" {
                        *self.selected_environment.borrow_mut() = None;
                    } else {
                        *self.selected_environment.borrow_mut() = Some(text);
                    }
                    self.save_state();
                }
            }
            Message::FileSelected => {
                let browser = self.file_browser.borrow();
                let idx = browser.value();
                if idx > 0 {
                    let files = self.file_tree.borrow().get_files();
                    if let Some(file) = files.get((idx - 1) as usize) {
                        *self.selected_file.borrow_mut() = Some(file.clone());
                        *self.selected_request_index.borrow_mut() = None;
                        
                        Self::load_environments_static(file, &self.environments);
                        self.request_view.borrow_mut().load_file(file);
                        
                        self.update_environment_selector();
                        self.update_request_display();
                        self.update_status();
                        self.save_state();
                    }
                }
            }
            Message::RunAllRequests => {
                if let Some(ref file) = *self.selected_file.borrow() {
                    self.results_view.borrow_mut().run_file(
                        file,
                        self.selected_environment.borrow().as_deref(),
                    );
                    
                    // Update results display after a short delay to show "Running" message
                    self.update_results_display();
                    
                    // Schedule periodic updates while running
                    let results_view = self.results_view.clone();
                    let results_buffer = self.results_buffer.clone();
                    std::thread::spawn(move || {
                        for _ in 0..100 {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            
                            let results = results_view.borrow().get_results();
                            let has_running = results.iter().any(|r| matches!(r, crate::results_view::ExecutionResult::Running { .. }));
                            
                            if !has_running {
                                // Update one last time and break
                                app::awake();
                                break;
                            }
                            app::awake();
                        }
                    });
                }
            }
            Message::RunRequest => {
                // Not implemented in this version
            }
            Message::SaveFile => {
                if let Err(e) = self.request_view.borrow_mut().save_to_file() {
                    eprintln!("Failed to save file: {}", e);
                } else {
                    *self.file_tree.borrow_mut() = FileTree::new(self.root_directory.borrow().clone());
                    self.update_file_list();
                }
            }
        }
        
        // Always update results display when awoken
        self.update_results_display();
    }

    fn update_file_list(&self) {
        let mut browser = self.file_browser.borrow_mut();
        browser.clear();
        
        let files = self.file_tree.borrow().get_files();
        for file in files {
            let display_name = file
                .strip_prefix(&*self.root_directory.borrow())
                .unwrap_or(&file)
                .display()
                .to_string();
            browser.add(&format!("📄  {}", display_name));
        }
    }

    fn update_environment_selector(&self) {
        let mut choice = self.env_choice.borrow_mut();
        choice.clear();
        choice.add("None");
        
        for env in self.environments.borrow().iter() {
            choice.add(env);
        }
        
        if let Some(ref env) = *self.selected_environment.borrow() {
            if let Some(idx) = self.environments.borrow().iter().position(|e| e == env) {
                choice.set_value((idx + 1) as i32);
            } else {
                choice.set_value(0);
            }
        } else {
            choice.set_value(0);
        }
    }

    fn update_request_display(&self) {
        let mut buffer = self.request_buffer.borrow_mut();
        
        if let Some(ref file) = *self.selected_file.borrow() {
            let requests = self.request_view.borrow().get_requests();
            
            if requests.is_empty() {
                buffer.set_text("No requests found in this file.");
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
                buffer.set_text(&text);
            }
        } else {
            buffer.set_text("No file selected. Select a .http file from the left panel.");
        }
    }

    fn update_results_display(&self) {
        let mut buffer = self.results_buffer.borrow_mut();
        let results = self.results_view.borrow().get_results();
        
        if results.is_empty() {
            buffer.set_text("No results yet. Run a request to see results here.");
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
                                let status_icon = if assertion.passed { "✓" } else { "✗" };
                                text.push_str(&format!("  {} {}: {}\n", status_icon, assertion.name, assertion.message));
                            }
                        }
                        
                        text.push_str(&format!("\nResponse Body:\n{}\n", response_body));
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
            buffer.set_text(&text);
        }
    }

    fn update_status(&self) {
        let mut buffer = self.status_buffer.borrow_mut();
        let mut text = format!("Working Directory: {}", self.root_directory.borrow().display());
        
        if let Some(ref file) = *self.selected_file.borrow() {
            text.push_str(&format!(" | Selected: {}", file.display()));
        }
        
        buffer.set_text(&text);
    }

    fn load_environments_static(file: &Path, environments: &Rc<RefCell<Vec<String>>>) {
        if let Some(file_str) = file.to_str()
            && let Ok(Some(env_file)) = httprunner_lib::environment::find_environment_file(file_str)
            && let Ok(env_config) = httprunner_lib::environment::parse_environment_file(&env_file)
        {
            let mut env_names: Vec<String> = env_config.keys().cloned().collect();
            env_names.sort();
            *environments.borrow_mut() = env_names;
            return;
        }
        environments.borrow_mut().clear();
    }

    fn save_state(&self) {
        let state = AppState {
            root_directory: Some(self.root_directory.borrow().clone()),
            selected_file: self.selected_file.borrow().clone(),
            selected_environment: self.selected_environment.borrow().clone(),
            font_size: None,
            window_size: Some((
                self.window.borrow().w() as f32,
                self.window.borrow().h() as f32,
            )),
            last_results: Some(self.results_view.borrow().get_results()),
        };

        if let Err(e) = state.save() {
            eprintln!("Failed to save application state: {}", e);
        }
    }
}

pub struct AppHandle {
    root_directory: Rc<RefCell<PathBuf>>,
    selected_file: Rc<RefCell<Option<PathBuf>>>,
    selected_environment: Rc<RefCell<Option<String>>>,
    window: Rc<RefCell<Window>>,
    results_view: Rc<RefCell<ResultsView>>,
}

impl AppHandle {
    pub fn save_state(&self) {
        let state = AppState {
            root_directory: Some(self.root_directory.borrow().clone()),
            selected_file: self.selected_file.borrow().clone(),
            selected_environment: self.selected_environment.borrow().clone(),
            font_size: None,
            window_size: Some((
                self.window.borrow().w() as f32,
                self.window.borrow().h() as f32,
            )),
            last_results: Some(self.results_view.borrow().get_results()),
        };

        if let Err(e) = state.save() {
            eprintln!("Failed to save application state: {}", e);
        }
    }
}
