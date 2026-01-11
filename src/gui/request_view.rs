use std::path::PathBuf;

pub struct RequestView {
    requests: Vec<RequestInfo>,
    selected_index: Option<usize>,
}

struct RequestInfo {
    name: Option<String>,
    method: String,
    url: String,
    headers: Vec<String>,
    body: Option<String>,
}

impl RequestView {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
            selected_index: None,
        }
    }

    pub fn load_file(&mut self, path: &PathBuf) {
        self.requests.clear();
        self.selected_index = None;
        
        // Parse the .http file
        if let Some(path_str) = path.to_str() {
            let parsed_requests = httprunner::parser::parse_http_file(path_str, None)
                .unwrap_or_default();
            
            for req in parsed_requests {
                let request_info = RequestInfo {
                    name: req.name.clone(),
                    method: req.method.clone(),
                    url: req.url.clone(),
                    headers: req.headers.iter()
                        .map(|h| format!("{}: {}", h.name, h.value))
                        .collect(),
                    body: req.body.clone(),
                };
                self.requests.push(request_info);
            }
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, file: &Option<PathBuf>) -> Option<usize> {
        let mut clicked_index = None;
        
        if file.is_none() {
            ui.label("No file selected. Select a .http file from the left panel.");
            return None;
        }
        
        if self.requests.is_empty() {
            ui.label("No requests found in this file.");
            return None;
        }
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (idx, request) in self.requests.iter().enumerate() {
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
                                    ui.monospace(header);
                                }
                            });
                        }
                        
                        if let Some(body) = &request.body {
                            ui.label("Body:");
                            ui.separator();
                            egui::ScrollArea::vertical()
                                .max_height(200.0)
                                .show(ui, |ui| {
                                    ui.monospace(body);
                                });
                        }
                        
                        ui.separator();
                        if ui.button("▶ Run this request").clicked() {
                            clicked_index = Some(idx);
                            self.selected_index = Some(idx);
                        }
                    });
            }
        });
        
        clicked_index
    }
}
