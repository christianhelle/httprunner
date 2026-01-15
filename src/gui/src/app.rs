use crate::{
    file_tree::{self, FileTree},
    request_view::{self, RequestView, RequestViewAction},
    results_view::ResultsView,
    state::AppState,
};
use iced::widget::{button, column, container, row, scrollable, text, Row};
use iced::{executor, keyboard, window, Element, Length, Subscription, Task, Theme};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum Message {
    // File operations
    FileSelected(PathBuf),
    OpenDirectoryClicked,
    DirectoryOpened(Option<PathBuf>),
    NewFileClicked,
    NewFileCreated(Option<PathBuf>),
    Quit,
    
    // Environment selection
    EnvironmentSelected(Option<String>),
    
    // Request operations
    RunRequest(usize),
    RunAllRequests,
    SaveFile,
    
    // File tree messages
    FileTreeMessage(file_tree::Message),
    
    // Request view messages
    RequestViewMessage(request_view::Message),
    
    // Font/UI controls
    IncreaseFontSize,
    DecreaseFontSize,
    ResetFontSize,
    
    // Keyboard shortcuts
    KeyPressed(keyboard::Key, keyboard::Modifiers),
    
    // Window events
    WindowResized(window::Id, iced::Size),
}

pub struct HttpRunnerApp {
    file_tree: FileTree,
    request_view: RequestView,
    results_view: ResultsView,
    selected_file: Option<PathBuf>,
    selected_request_index: Option<usize>,
    environments: Vec<String>,
    selected_environment: Option<String>,
    root_directory: PathBuf,
    font_size: f32,
    last_saved_window_size: Option<(f32, f32)>,
}

impl HttpRunnerApp {
    const DEFAULT_FONT_SIZE: f32 = 14.0;
    const MIN_FONT_SIZE: f32 = 8.0;
    const MAX_FONT_SIZE: f32 = 32.0;
    const FONT_SIZE_STEP: f32 = 1.0;

    pub fn new() -> (Self, Task<Message>) {
        // Load saved state
        let state = AppState::load();

        // Use saved root directory or fall back to current directory
        let root_directory = state
            .root_directory
            .and_then(|p| if p.exists() { Some(p) } else { None })
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        // Use saved font size or default
        let font_size = state.font_size.unwrap_or(Self::DEFAULT_FONT_SIZE);

        let mut app = Self {
            file_tree: FileTree::new(root_directory.clone()),
            request_view: RequestView::new(),
            results_view: ResultsView::new(),
            selected_file: None,
            selected_request_index: None,
            environments: Vec::new(),
            selected_environment: None,
            root_directory,
            font_size,
            last_saved_window_size: state.window_size,
        };

        // Restore last results if available
        if let Some(last_results) = state.last_results {
            app.results_view.restore_results(last_results);
        }

        // Restore selected file if it still exists
        if let Some(saved_file) = state.selected_file {
            if saved_file.exists() {
                app.selected_file = Some(saved_file.clone());
                app.load_environments(&saved_file);
                app.request_view.load_file(&saved_file);

                // Restore selected environment if it's still valid
                if let Some(saved_env) = state.selected_environment {
                    if app.environments.contains(&saved_env) {
                        app.selected_environment = Some(saved_env);
                    }
                }
            }
        }

        (app, Task::none())
    }

    fn load_environments(&mut self, file: &Path) {
        // Try to find and parse http-client.env.json
        if let Some(file_str) = file.to_str()
            && let Ok(Some(env_file)) = httprunner_lib::environment::find_environment_file(file_str)
            && let Ok(env_config) = httprunner_lib::environment::parse_environment_file(&env_file)
        {
            // Extract environment names from the config
            self.environments = env_config.keys().cloned().collect();
            self.environments.sort(); // Sort alphabetically for consistent UI
            return;
        }
        // No environments found or error occurred
        self.environments = Vec::new();
    }

    fn save_state(&self) {
        self.save_state_internal(None);
    }

    fn save_state_with_window(&self, window_size: (f32, f32)) {
        self.save_state_internal(Some(window_size));
    }

    fn save_state_internal(&self, window_size: Option<(f32, f32)>) {
        let state = AppState {
            root_directory: Some(self.root_directory.clone()),
            selected_file: self.selected_file.clone(),
            selected_environment: self.selected_environment.clone(),
            font_size: Some(self.font_size),
            window_size,
            last_results: Some(self.results_view.get_results()),
        };

        if let Err(e) = state.save() {
            eprintln!("Failed to save application state: {}", e);
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FileSelected(path) => {
                self.selected_file = Some(path.clone());
                self.selected_request_index = None;
                self.load_environments(&path);
                self.request_view.load_file(&path);
                self.save_state();
                Task::none()
            }
            Message::OpenDirectoryClicked => {
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .pick_folder()
                            .await
                            .map(|handle| handle.path().to_path_buf())
                    },
                    Message::DirectoryOpened,
                )
            }
            Message::DirectoryOpened(Some(path)) => {
                self.root_directory = path.clone();
                self.file_tree = FileTree::new(path);
                self.selected_file = None;
                self.selected_request_index = None;
                self.save_state();
                Task::none()
            }
            Message::DirectoryOpened(None) => Task::none(),
            Message::NewFileClicked => {
                let root = self.root_directory.clone();
                Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .set_directory(&root)
                            .add_filter("HTTP Files", &["http"])
                            .set_file_name("new.http")
                            .save_file()
                            .await
                            .map(|handle| handle.path().to_path_buf())
                    },
                    Message::NewFileCreated,
                )
            }
            Message::NewFileCreated(Some(path)) => {
                // Create an empty .http file
                if let Err(e) = std::fs::write(&path, "### New Request\nGET https://httpbin.org/get\n") {
                    eprintln!("Failed to create file: {}", e);
                } else {
                    // Refresh file tree and select the new file
                    self.file_tree = FileTree::new(self.root_directory.clone());
                    self.selected_file = Some(path.clone());
                    self.request_view.load_file(&path);
                    self.save_state();
                }
                Task::none()
            }
            Message::NewFileCreated(None) => Task::none(),
            Message::Quit => {
                self.save_state();
                iced::exit()
            }
            Message::EnvironmentSelected(env) => {
                self.selected_environment = env;
                self.save_state();
                Task::none()
            }
            Message::RunRequest(idx) => {
                self.selected_request_index = Some(idx);
                if let Some(file) = &self.selected_file {
                    self.results_view.run_single_request(
                        file,
                        idx,
                        self.selected_environment.as_deref(),
                    );
                }
                Task::none()
            }
            Message::RunAllRequests => {
                if !self.request_view.has_changes() {
                    if let Some(file) = &self.selected_file {
                        self.results_view
                            .run_file(file, self.selected_environment.as_deref());
                    }
                }
                Task::none()
            }
            Message::SaveFile => {
                if let Err(e) = self.request_view.save_to_file() {
                    eprintln!("Failed to save file: {}", e);
                } else {
                    // Refresh the file tree to show any new files
                    self.file_tree = FileTree::new(self.root_directory.clone());
                }
                Task::none()
            }
            Message::FileTreeMessage(msg) => {
                if let Some(file) = self.file_tree.update(msg) {
                    return self.update(Message::FileSelected(file));
                }
                Task::none()
            }
            Message::RequestViewMessage(msg) => {
                match self.request_view.update(msg) {
                    RequestViewAction::RunRequest(idx) => {
                        self.update(Message::RunRequest(idx))
                    }
                    RequestViewAction::SaveFile => {
                        self.update(Message::SaveFile)
                    }
                    RequestViewAction::None => Task::none(),
                }
            }
            Message::IncreaseFontSize => {
                self.font_size = (self.font_size + Self::FONT_SIZE_STEP).min(Self::MAX_FONT_SIZE);
                self.save_state();
                Task::none()
            }
            Message::DecreaseFontSize => {
                self.font_size = (self.font_size - Self::FONT_SIZE_STEP).max(Self::MIN_FONT_SIZE);
                self.save_state();
                Task::none()
            }
            Message::ResetFontSize => {
                self.font_size = Self::DEFAULT_FONT_SIZE;
                self.save_state();
                Task::none()
            }
            Message::KeyPressed(key, modifiers) => {
                self.handle_keyboard_shortcut(key, modifiers)
            }
            Message::WindowResized(_id, size) => {
                let current_size = (size.width, size.height);
                let should_save = match self.last_saved_window_size {
                    None => true,
                    Some(last_size) => last_size != current_size,
                };
                
                if should_save {
                    self.last_saved_window_size = Some(current_size);
                    self.save_state_with_window(current_size);
                }
                Task::none()
            }
        }
    }

    fn handle_keyboard_shortcut(&mut self, key: keyboard::Key, modifiers: keyboard::Modifiers) -> Task<Message> {
        use keyboard::Key;
        
        match key {
            Key::Character(ref c) if modifiers.control() => {
                match c.as_str() {
                    "o" | "O" => return self.update(Message::OpenDirectoryClicked),
                    "q" | "Q" => return self.update(Message::Quit),
                    "e" | "E" => {
                        // Cycle through environments
                        if !self.environments.is_empty() {
                            let new_env = if let Some(ref current_env) = self.selected_environment {
                                if let Some(idx) = self.environments.iter().position(|e| e == current_env) {
                                    let next_idx = (idx + 1) % (self.environments.len() + 1);
                                    if next_idx == self.environments.len() {
                                        None
                                    } else {
                                        Some(self.environments[next_idx].clone())
                                    }
                                } else {
                                    self.environments.first().cloned()
                                }
                            } else {
                                self.environments.first().cloned()
                            };
                            return self.update(Message::EnvironmentSelected(new_env));
                        }
                    }
                    "=" | "+" => return self.update(Message::IncreaseFontSize),
                    "-" => return self.update(Message::DecreaseFontSize),
                    "0" => return self.update(Message::ResetFontSize),
                    _ => {}
                }
            }
            Key::Named(keyboard::key::Named::F5) => {
                return self.update(Message::RunAllRequests);
            }
            _ => {}
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        let content = column![
            self.top_menu(),
            row![
                self.file_tree_panel(),
                self.center_panel(),
            ]
            .spacing(10)
            .padding(10),
            self.bottom_panel(),
        ]
        .spacing(0);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn top_menu(&self) -> Element<Message> {
        let open_btn = button("Open Directory...").on_press(Message::OpenDirectoryClicked);
        let new_btn = button("New .http File...").on_press(Message::NewFileClicked);
        let quit_btn = button("Quit").on_press(Message::Quit);

        let env_text = self.selected_environment.as_deref().unwrap_or("None");
        let env_label = text(format!("Environment: {}", env_text));

        let menu = row![
            open_btn,
            new_btn,
            quit_btn,
        ]
        .spacing(10)
        .padding(10);

        let menu_with_env = row![
            menu,
            env_label,
        ]
        .spacing(10);

        container(menu_with_env)
            .width(Length::Fill)
            .style(container::bordered_box)
            .into()
    }

    fn file_tree_panel(&self) -> Element<Message> {
        let content = column![
            text("HTTP Files").size(18),
            scrollable(
                self.file_tree
                    .view()
                    .map(Message::FileTreeMessage)
            )
            .height(Length::Fill),
        ]
        .spacing(10)
        .padding(10)
        .width(Length::Fixed(300.0));

        container(content)
            .height(Length::Fill)
            .style(container::bordered_box)
            .into()
    }

    fn center_panel(&self) -> Element<Message> {
        column![
            self.request_details_panel(),
            self.results_panel(),
        ]
        .spacing(10)
        .width(Length::Fill)
        .into()
    }

    fn request_details_panel(&self) -> Element<Message> {
        let mut content = column![text("Request Details").size(18)].spacing(10);

        let request_content = scrollable(
            self.request_view
                .view(&self.selected_file)
                .map(Message::RequestViewMessage)
        )
        .height(Length::Fixed(400.0));

        content = content.push(request_content);

        // Run buttons
        let run_all_enabled = self.selected_file.is_some() && !self.request_view.has_changes();
        let run_all_btn = if run_all_enabled {
            button("▶ Run All Requests").on_press(Message::RunAllRequests)
        } else {
            button("▶ Run All Requests")
        };

        let mut button_row = Row::new().spacing(10).push(run_all_btn);

        if self.request_view.has_changes() {
            button_row = button_row.push(text("● Unsaved changes").style(text::warning));
        }

        content = content.push(button_row);

        container(content)
            .padding(10)
            .style(container::bordered_box)
            .into()
    }

    fn results_panel(&self) -> Element<Message> {
        let content = column![
            text("Results").size(18),
            scrollable(self.results_view.view()).height(Length::Fill),
        ]
        .spacing(10);

        container(content)
            .padding(10)
            .height(Length::Fill)
            .style(container::bordered_box)
            .into()
    }

    fn bottom_panel(&self) -> Element<Message> {
        let status = row![
            text(format!("Working Directory: {}", self.root_directory.display())),
            text(" | "),
            text(if let Some(file) = &self.selected_file {
                format!("Selected: {}", file.display())
            } else {
                String::new()
            }),
        ]
        .spacing(5)
        .padding(5);

        container(status)
            .width(Length::Fill)
            .style(container::bordered_box)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        use iced::event::{self, Event};
        use iced::keyboard;

        let keyboard_events = event::listen_with(|event, _status, _id| {
            if let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event {
                Some(Message::KeyPressed(key, modifiers))
            } else {
                None
            }
        });

        let window_events = event::listen_with(|event, _status, id| {
            if let Event::Window(window::Event::Resized(size)) = event {
                Some(Message::WindowResized(id, size))
            } else {
                None
            }
        });

        Subscription::batch(vec![keyboard_events, window_events])
    }

    pub fn theme(&self) -> Theme {
        Theme::CatppuccinMocha
    }
}
