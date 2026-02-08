use crate::request_editor::RequestEditor;
use std::path::{Path, PathBuf};

pub enum RequestViewAction {
    RunRequest(usize),
    SaveFile,
    None,
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

    pub fn show(&mut self, ui: &mut egui::Ui, file: &Option<PathBuf>) -> RequestViewAction {
        let mut action = RequestViewAction::None;

        if file.is_none() {
            ui.label("No file selected. Select a .http file from the left panel.");
            return action;
        }

        // Show editor if we're editing
        if self.editor.is_editing() {
            action = self.show_editor(ui);
            return action;
        }

        let requests: Vec<_> = self.editor.get_requests().to_vec();

        if requests.is_empty() {
            ui.label("No requests found in this file.");
            ui.separator();
            if ui.button("➕ Add New Request").clicked() {
                self.editor.start_new_request();
            }
            return action;
        }

        // Show list of requests
        for (idx, request) in requests.iter().enumerate() {
            let header_text = if let Some(name) = &request.name {
                format!("{} - {} {}", idx + 1, request.method, name)
            } else {
                format!("{} - {} {}", idx + 1, request.method, request.url)
            };

            egui::CollapsingHeader::new(header_text)
                .default_open(false)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Method:");
                        ui.monospace(&request.method);
                    });

                    ui.horizontal(|ui| {
                        ui.label("URL:");
                        ui.monospace(&request.url);
                    });

                    if !request.headers.is_empty() {
                        ui.label("Headers:");
                        ui.indent("headers", |ui| {
                            for header in &request.headers {
                                ui.monospace(format!("{}: {}", header.name, header.value));
                            }
                        });
                    }

                    if let Some(body) = &request.body {
                        ui.label("Body:");
                        ui.separator();
                        egui::ScrollArea::vertical()
                            .id_salt(format!("request_body_{}", idx))
                            .max_height(200.0)
                            .show(ui, |ui| {
                                ui.monospace(body);
                            });
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("▶ Run").clicked() {
                            action = RequestViewAction::RunRequest(idx);
                        }
                        if ui.button("✏ Edit").clicked() {
                            self.editor.start_editing(idx);
                        }
                        if ui.button("🗑 Delete").clicked() {
                            self.editor.delete_request(idx);
                            action = RequestViewAction::SaveFile;
                        }
                    });
                });
        }

        ui.separator();
        if ui.button("➕ Add New Request").clicked() {
            self.editor.start_new_request();
        }

        action
    }

    fn show_editor(&mut self, ui: &mut egui::Ui) -> RequestViewAction {
        let mut action = RequestViewAction::None;

        ui.heading("Edit Request");
        ui.separator();

        if let Some(editable) = self.editor.get_editing_request_mut() {
            ui.horizontal(|ui| {
                ui.label("Name (optional):");
                ui.text_edit_singleline(&mut editable.name);
            });

            ui.horizontal(|ui| {
                ui.label("Method:");
                egui::ComboBox::from_id_salt("method_combo")
                    .selected_text(&editable.method)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut editable.method, "GET".to_string(), "GET");
                        ui.selectable_value(&mut editable.method, "POST".to_string(), "POST");
                        ui.selectable_value(&mut editable.method, "PUT".to_string(), "PUT");
                        ui.selectable_value(&mut editable.method, "DELETE".to_string(), "DELETE");
                        ui.selectable_value(&mut editable.method, "PATCH".to_string(), "PATCH");
                        ui.selectable_value(&mut editable.method, "HEAD".to_string(), "HEAD");
                        ui.selectable_value(&mut editable.method, "OPTIONS".to_string(), "OPTIONS");
                    });
            });

            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.text_edit_singleline(&mut editable.url);
            });

            ui.separator();
            ui.label("Headers:");

            let mut headers_to_remove = Vec::new();
            for (i, (name, value)) in editable.headers.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(name);
                    ui.label("Value:");
                    ui.text_edit_singleline(value);
                    if ui.button("🗑").clicked() {
                        headers_to_remove.push(i);
                    }
                });
            }

            // Remove headers in reverse order to maintain indices
            for i in headers_to_remove.iter().rev() {
                editable.headers.remove(*i);
            }

            if ui.button("➕ Add Header").clicked() {
                editable.headers.push((String::new(), String::new()));
            }

            ui.separator();
            ui.label("Body:");
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    ui.text_edit_multiline(&mut editable.body);
                });

            ui.separator();
            ui.collapsing("Advanced Options", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Timeout (ms):");
                    ui.text_edit_singleline(&mut editable.timeout);
                });

                ui.horizontal(|ui| {
                    ui.label("Connection Timeout (ms):");
                    ui.text_edit_singleline(&mut editable.connection_timeout);
                });

                ui.horizontal(|ui| {
                    ui.label("Depends On:");
                    ui.text_edit_singleline(&mut editable.depends_on);
                });
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("💾 Save").clicked() && self.editor.save_current_edit() {
                    action = RequestViewAction::SaveFile;
                }
                if ui.button("❌ Cancel").clicked() {
                    self.editor.cancel_editing();
                }
            });
        }

        action
    }

    pub fn save_to_file(&mut self) -> anyhow::Result<()> {
        self.editor.save_to_file()
    }

    pub fn has_changes(&self) -> bool {
        self.editor.has_changes()
    }
}
