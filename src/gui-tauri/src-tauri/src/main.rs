// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppState {
    root_directory: PathBuf,
    selected_file: Option<PathBuf>,
    selected_environment: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            root_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            selected_file: None,
            selected_environment: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HttpFile {
    path: String,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RequestInfo {
    index: usize,
    method: String,
    url: String,
    name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExecutionResult {
    success: bool,
    method: String,
    url: String,
    status: Option<u16>,
    duration_ms: Option<u64>,
    response_body: Option<String>,
    error: Option<String>,
}

// Tauri commands
#[tauri::command]
fn set_root_directory(path: String, state: State<Mutex<AppState>>) -> Result<(), String> {
    let path_buf = PathBuf::from(path);
    if path_buf.exists() {
        let mut app_state = state.lock().map_err(|e| e.to_string())?;
        app_state.root_directory = path_buf;
        Ok(())
    } else {
        Err("Path does not exist".to_string())
    }
}

#[tauri::command]
fn get_root_directory(state: State<Mutex<AppState>>) -> Result<String, String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    Ok(app_state.root_directory.to_string_lossy().to_string())
}

#[tauri::command]
fn list_http_files(state: State<Mutex<AppState>>) -> Result<Vec<HttpFile>, String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    let root = &app_state.root_directory;
    
    let mut files = Vec::new();
    
    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "http" {
                    files.push(HttpFile {
                        path: entry.path().to_string_lossy().to_string(),
                        name: entry.file_name().to_string_lossy().to_string(),
                    });
                }
            }
        }
    }
    
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

#[tauri::command]
fn read_file_content(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
fn write_file_content(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))
}

#[tauri::command]
fn select_file(path: String, state: State<Mutex<AppState>>) -> Result<(), String> {
    let mut app_state = state.lock().map_err(|e| e.to_string())?;
    app_state.selected_file = Some(PathBuf::from(path));
    Ok(())
}

#[tauri::command]
fn get_selected_file(state: State<Mutex<AppState>>) -> Result<Option<String>, String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    Ok(app_state.selected_file.as_ref().map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
fn parse_http_file(path: String) -> Result<Vec<RequestInfo>, String> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let requests = httprunner_lib::parser::parse(&content)
        .map_err(|e| format!("Failed to parse file: {}", e))?;
    
    Ok(requests.iter().enumerate().map(|(i, req)| {
        RequestInfo {
            index: i,
            method: req.method.to_string(),
            url: req.url.clone(),
            name: req.name.clone(),
        }
    }).collect())
}

#[tauri::command]
fn list_environments(path: String) -> Result<Vec<String>, String> {
    // Try to find and parse http-client.env.json
    match httprunner_lib::environment::find_environment_file(&path) {
        Ok(Some(env_file)) => {
            match httprunner_lib::environment::parse_environment_file(&env_file) {
                Ok(env_config) => {
                    let mut envs: Vec<String> = env_config.keys().cloned().collect();
                    envs.sort();
                    Ok(envs)
                }
                Err(_) => Ok(Vec::new()),
            }
        }
        _ => Ok(Vec::new()),
    }
}

#[tauri::command]
fn set_environment(env: Option<String>, state: State<Mutex<AppState>>) -> Result<(), String> {
    let mut app_state = state.lock().map_err(|e| e.to_string())?;
    app_state.selected_environment = env;
    Ok(())
}

#[tauri::command]
fn get_environment(state: State<Mutex<AppState>>) -> Result<Option<String>, String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    Ok(app_state.selected_environment.clone())
}

#[tauri::command]
fn run_single_request(path: String, index: usize, environment: Option<String>) -> Result<ExecutionResult, String> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let requests = httprunner_lib::parser::parse(&content)
        .map_err(|e| format!("Failed to parse file: {}", e))?;
    
    if index >= requests.len() {
        return Err("Request index out of bounds".to_string());
    }
    
    let request = &requests[index];
    let method = request.method.to_string();
    let url = request.url.clone();
    
    let start = std::time::Instant::now();
    match httprunner_lib::runner::execute_single_request(&path, index, environment.as_deref()) {
        Ok(result) => {
            let duration = start.elapsed();
            Ok(ExecutionResult {
                success: result.success,
                method,
                url,
                status: Some(result.status_code),
                duration_ms: Some(duration.as_millis() as u64),
                response_body: Some(result.body),
                error: None,
            })
        }
        Err(e) => Ok(ExecutionResult {
            success: false,
            method,
            url,
            status: None,
            duration_ms: Some(start.elapsed().as_millis() as u64),
            response_body: None,
            error: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
fn run_all_requests(path: String, environment: Option<String>) -> Result<Vec<ExecutionResult>, String> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let requests = httprunner_lib::parser::parse(&content)
        .map_err(|e| format!("Failed to parse file: {}", e))?;
    
    let mut results = Vec::new();
    
    for (index, request) in requests.iter().enumerate() {
        let method = request.method.to_string();
        let url = request.url.clone();
        let start = std::time::Instant::now();
        
        match httprunner_lib::runner::execute_single_request(&path, index, environment.as_deref()) {
            Ok(result) => {
                let duration = start.elapsed();
                results.push(ExecutionResult {
                    success: result.success,
                    method,
                    url,
                    status: Some(result.status_code),
                    duration_ms: Some(duration.as_millis() as u64),
                    response_body: Some(result.body),
                    error: None,
                });
            }
            Err(e) => {
                results.push(ExecutionResult {
                    success: false,
                    method,
                    url,
                    status: None,
                    duration_ms: Some(start.elapsed().as_millis() as u64),
                    response_body: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }
    
    Ok(results)
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            set_root_directory,
            get_root_directory,
            list_http_files,
            read_file_content,
            write_file_content,
            select_file,
            get_selected_file,
            parse_http_file,
            list_environments,
            set_environment,
            get_environment,
            run_single_request,
            run_all_requests,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
