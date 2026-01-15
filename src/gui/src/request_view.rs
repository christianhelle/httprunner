use crate::request_editor::RequestEditor;
use iced::widget::{button, column, row, text, Column};
use iced::Element;
use std::path::{Path, PathBuf};

pub enum RequestViewAction {
    RunRequest(usize),
    SaveFile,
    None,
}

#[derive(Debug, Clone)]
pub enum Message {
    RunRequest(usize),
    EditRequest(usize),
    DeleteRequest(usize),
    AddNewRequest,
    CancelEdit,
    SaveEdit,
}

pub struct RequestView {
    editor: RequestEditor,
}

impl RequestView {
    pub fn new() -> Self {
        Self {
            editor: RequestEditor::new(),
        }
    }

    pub fn load_file(&mut self, path: &Path) {
        self.editor.load_file(path);
    }

    pub fn update(&mut self, message: Message) -> RequestViewAction {
        match message {
            Message::RunRequest(idx) => RequestViewAction::RunRequest(idx),
            Message::EditRequest(idx) => {
                self.editor.start_editing(idx);
                RequestViewAction::None
            }
            Message::DeleteRequest(idx) => {
                self.editor.delete_request(idx);
                RequestViewAction::SaveFile
            }
            Message::AddNewRequest => {
                self.editor.start_new_request();
                RequestViewAction::None
            }
            Message::CancelEdit => {
                self.editor.cancel_editing();
                RequestViewAction::None
            }
            Message::SaveEdit => {
                if self.editor.save_current_edit() {
                    RequestViewAction::SaveFile
                } else {
                    RequestViewAction::None
                }
            }
        }
    }

    pub fn view(&self, file: &Option<PathBuf>) -> Element<Message> {
        let mut content: Column<Message> = Column::new().spacing(10);

        if file.is_none() {
            content = content.push(text("No file selected. Select a .http file from the left panel."));
            return content.into();
        }

        // Show editor if we're editing
        if self.editor.is_editing() {
            return self.show_editor();
        }

        // Clone requests to avoid borrowing issues
        let requests: Vec<_> = self.editor.get_requests().to_vec();

        if requests.is_empty() {
            content = content
                .push(text("No requests found in this file."))
                .push(button("➕ Add New Request").on_press(Message::AddNewRequest));
            return content.into();
        }

        // Show list of requests
        for (idx, request) in requests.iter().enumerate() {
            let header_text = if let Some(name) = &request.name {
                format!("{} - {} {}", idx + 1, request.method, name)
            } else {
                format!("{} - {} {}", idx + 1, request.method, request.url)
            };

            let mut request_section = column![
                text(header_text).size(16),
                row![
                    text("Method: "),
                    text(&request.method),
                ],
                row![
                    text("URL: "),
                    text(&request.url),
                ],
            ]
            .spacing(5);

            if !request.headers.is_empty() {
                request_section = request_section.push(text("Headers:"));
                for header in &request.headers {
                    request_section =
                        request_section.push(text(format!("  {}: {}", header.name, header.value)));
                }
            }

            if let Some(body) = &request.body {
                request_section = request_section.push(text("Body:"));
                request_section = request_section.push(text(body));
            }

            let button_row = row![
                button("▶ Run").on_press(Message::RunRequest(idx)),
                button("✏ Edit").on_press(Message::EditRequest(idx)),
                button("🗑 Delete").on_press(Message::DeleteRequest(idx)),
            ]
            .spacing(10);

            request_section = request_section.push(button_row);

            content = content.push(request_section);
        }

        content = content.push(button("➕ Add New Request").on_press(Message::AddNewRequest));

        content.into()
    }

    fn show_editor(&self) -> Element<Message> {
        let mut content: Column<Message> = column![
            text("Edit Request").size(18),
            text("Editor functionality coming soon..."),
        ]
        .spacing(10);

        let button_row = row![
            button("💾 Save").on_press(Message::SaveEdit),
            button("❌ Cancel").on_press(Message::CancelEdit),
        ]
        .spacing(10);

        content = content.push(button_row);

        content.into()
    }

    pub fn save_to_file(&mut self) -> anyhow::Result<()> {
        self.editor.save_to_file()
    }

    pub fn has_changes(&self) -> bool {
        self.editor.has_changes()
    }
}
