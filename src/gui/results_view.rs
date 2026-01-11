use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
pub enum ExecutionResult {
    Success {
        method: String,
        url: String,
        status: u16,
        duration_ms: u64,
        response_body: String,
    },
    Failure {
        method: String,
        url: String,
        error: String,
    },
    Running {
        message: String,
    },
}

pub struct ResultsView {
    results: Arc<Mutex<Vec<ExecutionResult>>>,
    is_running: Arc<Mutex<bool>>,
}

impl ResultsView {
    pub fn new() -> Self {
        Self {
            results: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn run_file(&mut self, path: &PathBuf, environment: Option<&str>) {
        let path = path.clone();
        let env = environment.map(|s| s.to_string());
        let results = Arc::clone(&self.results);
        let is_running = Arc::clone(&self.is_running);
        
        // Clear previous results
        if let Ok(mut r) = results.lock() {
            r.clear();
            r.push(ExecutionResult::Running {
                message: format!("Running all requests from {}...", path.display()),
            });
        }
        
        if let Ok(mut running) = is_running.lock() {
            *running = true;
        }
        
        thread::spawn(move || {
            // Parse the file
            if let Some(path_str) = path.to_str() {
                if let Ok(requests) = httprunner::parser::parse_http_file(path_str, env.as_deref()) {
                    // Execute requests
                    for request in requests {
                        let result = execute_request(request);
                        if let Ok(mut r) = results.lock() {
                            r.push(result);
                        }
                    }
                } else if let Ok(mut r) = results.lock() {
                    r.clear();
                    r.push(ExecutionResult::Failure {
                        method: "PARSE".to_string(),
                        url: path.display().to_string(),
                        error: "Failed to parse .http file".to_string(),
                    });
                }
            } else if let Ok(mut r) = results.lock() {
                r.clear();
                r.push(ExecutionResult::Failure {
                    method: "READ".to_string(),
                    url: path.display().to_string(),
                    error: "Failed to convert path to string".to_string(),
                });
            }
            
            if let Ok(mut running) = is_running.lock() {
                *running = false;
            }
        });
    }

    pub fn run_single_request(&mut self, path: &PathBuf, index: usize, environment: Option<&str>) {
        let path = path.clone();
        let env = environment.map(|s| s.to_string());
        let results = Arc::clone(&self.results);
        let is_running = Arc::clone(&self.is_running);
        
        // Clear previous results
        if let Ok(mut r) = results.lock() {
            r.clear();
            r.push(ExecutionResult::Running {
                message: format!("Running request {} from {}...", index + 1, path.display()),
            });
        }
        
        if let Ok(mut running) = is_running.lock() {
            *running = true;
        }
        
        thread::spawn(move || {
            // Parse the file
            if let Some(path_str) = path.to_str() {
                if let Ok(requests) = httprunner::parser::parse_http_file(path_str, env.as_deref()) {
                    if let Some(request) = requests.get(index) {
                        let result = execute_request(request.clone());
                        if let Ok(mut r) = results.lock() {
                            r.clear();
                            r.push(result);
                        }
                    } else if let Ok(mut r) = results.lock() {
                        r.clear();
                        r.push(ExecutionResult::Failure {
                            method: "INDEX".to_string(),
                            url: path.display().to_string(),
                            error: format!("Request index {} not found", index),
                        });
                    }
                } else if let Ok(mut r) = results.lock() {
                    r.clear();
                    r.push(ExecutionResult::Failure {
                        method: "PARSE".to_string(),
                        url: path.display().to_string(),
                        error: "Failed to parse .http file".to_string(),
                    });
                }
            } else if let Ok(mut r) = results.lock() {
                r.clear();
                r.push(ExecutionResult::Failure {
                    method: "PATH".to_string(),
                    url: path.display().to_string(),
                    error: "Invalid file path".to_string(),
                });
            }
            
            if let Ok(mut running) = is_running.lock() {
                *running = false;
            }
        });
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        if let Ok(is_running) = self.is_running.lock() {
            if *is_running {
                ui.spinner();
            }
        }
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Ok(results) = self.results.lock() {
                if results.is_empty() {
                    ui.label("No results yet. Select and run a request.");
                    return;
                }
                
                for result in results.iter() {
                    match result {
                        ExecutionResult::Success { method, url, status, duration_ms, response_body } => {
                            ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "✅ SUCCESS");
                            ui.monospace(format!("{} {}", method, url));
                            ui.label(format!("Status: {}", status));
                            ui.label(format!("Duration: {} ms", duration_ms));
                            
                            ui.separator();
                            ui.label("Response:");
                            egui::ScrollArea::vertical()
                                .max_height(300.0)
                                .show(ui, |ui| {
                                    ui.monospace(response_body);
                                });
                            ui.separator();
                        }
                        ExecutionResult::Failure { method, url, error } => {
                            ui.colored_label(egui::Color32::from_rgb(200, 0, 0), "❌ FAILED");
                            ui.monospace(format!("{} {}", method, url));
                            ui.colored_label(egui::Color32::from_rgb(200, 0, 0), error);
                            ui.separator();
                        }
                        ExecutionResult::Running { message } => {
                            ui.colored_label(egui::Color32::from_rgb(0, 100, 200), "⏳ RUNNING");
                            ui.label(message);
                            ui.separator();
                        }
                    }
                }
            }
        });
    }
}

fn execute_request(
    request: httprunner::HttpRequest,
) -> ExecutionResult {
    use std::time::Instant;
    
    let start = Instant::now();
    
    // Execute the request using the runner
    match httprunner::runner::execute_http_request(&request, false, false) {
        Ok(result) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            
            if result.success {
                ExecutionResult::Success {
                    method: request.method,
                    url: request.url,
                    status: result.status_code,
                    duration_ms,
                    response_body: result.response_body.unwrap_or_default(),
                }
            } else {
                ExecutionResult::Failure {
                    method: request.method,
                    url: request.url,
                    error: result.error_message.unwrap_or_else(|| "Unknown error".to_string()),
                }
            }
        }
        Err(e) => ExecutionResult::Failure {
            method: request.method,
            url: request.url,
            error: e.to_string(),
        },
    }
}
