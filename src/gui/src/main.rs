#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod state;

use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::path::PathBuf;
use std::rc::Rc;
use walkdir::WalkDir;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    env_logger::init();

    // Load saved state
    let saved_state = state::AppState::load();

    // Create the main window
    let ui = MainWindow::new()?;

    // Initialize state
    let root_directory = saved_state
        .root_directory
        .and_then(|p| if p.exists() { Some(p) } else { None })
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Set working directory
    ui.set_working_directory(root_directory.display().to_string().into());

    // Load HTTP files
    let http_files = discover_http_files(&root_directory);
    let files_model = Rc::new(VecModel::from(http_files));
    ui.set_http_files(ModelRc::from(files_model.clone()));

    // Restore selected file if available
    if let Some(saved_file) = saved_state.selected_file {
        if saved_file.exists() {
            ui.set_selected_file(saved_file.display().to_string().into());
            load_file_requests(&ui, &saved_file);
            load_environments(&ui, &saved_file);

            // Restore selected environment
            if let Some(saved_env) = saved_state.selected_environment {
                let envs = ui.get_environments();
                for i in 0..envs.row_count() {
                    if envs.row_data(i) == Some(saved_env.clone().into()) {
                        ui.set_selected_environment_index(i as i32);
                        break;
                    }
                }
            }
        }
    }

    // Restore last results if available
    if let Some(last_results) = saved_state.last_results {
        restore_results(&ui, last_results);
    }

    // Setup callbacks
    setup_callbacks(&ui, root_directory);

    ui.run()
}

fn discover_http_files(root_path: &PathBuf) -> Vec<HttpFile> {
    let mut http_files = Vec::new();

    for entry in WalkDir::new(root_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "http" {
                    let path = entry.path().to_path_buf();
                    let name = entry
                        .file_name()
                        .to_str()
                        .unwrap_or("unknown")
                        .to_string();

                    http_files.push(HttpFile {
                        path: path.display().to_string().into(),
                        name: name.into(),
                    });
                }
            }
        }
    }

    http_files.sort_by(|a, b| a.path.cmp(&b.path));
    http_files
}

fn load_file_requests(ui: &MainWindow, file_path: &PathBuf) {
    let requests = match httprunner_lib::parser::parse_http_file(file_path.to_str().unwrap()) {
        Ok(parsed_requests) => {
            parsed_requests
                .into_iter()
                .enumerate()
                .map(|(idx, req)| {
                    let headers = req
                        .headers
                        .iter()
                        .map(|h| format!("{}: {}", h.name, h.value))
                        .collect::<Vec<_>>()
                        .join("\n");

                    HttpRequest {
                        index: (idx + 1) as i32,
                        name: req.name.unwrap_or_default().into(),
                        method: req.method.into(),
                        url: req.url.into(),
                        headers: headers.into(),
                        body: req.body.unwrap_or_default().into(),
                    }
                })
                .collect()
        }
        Err(e) => {
            eprintln!("Failed to parse file: {}", e);
            Vec::new()
        }
    };

    let requests_model = Rc::new(VecModel::from(requests));
    ui.set_requests(ModelRc::from(requests_model));
}

fn load_environments(ui: &MainWindow, file_path: &PathBuf) {
    let mut environments = vec!["None".to_string()];

    if let Some(file_str) = file_path.to_str() {
        if let Ok(Some(env_file)) = httprunner_lib::environment::find_environment_file(file_str) {
            if let Ok(env_config) = httprunner_lib::environment::parse_environment_file(&env_file) {
                let mut env_names: Vec<String> = env_config.keys().cloned().collect();
                env_names.sort();
                environments.extend(env_names);
            }
        }
    }

    let env_model = Rc::new(VecModel::from(
        environments
            .into_iter()
            .map(|s| s.into())
            .collect::<Vec<slint::SharedString>>(),
    ));
    ui.set_environments(ModelRc::from(env_model));
    ui.set_selected_environment_index(0); // Default to "None"
}

fn restore_results(ui: &MainWindow, saved_results: Vec<state::ExecutionResult>) {
    let results: Vec<ExecutionResult> = saved_results
        .into_iter()
        .map(|r| match r {
            state::ExecutionResult::Success {
                method,
                url,
                status,
                duration_ms,
                response_body,
                ..
            } => ExecutionResult {
                method: method.into(),
                url: url.into(),
                status: status.to_string().into(),
                duration: format!("{}ms", duration_ms).into(),
                response: response_body.into(),
                is_success: true,
                is_running: false,
            },
            state::ExecutionResult::Failure { method, url, error } => ExecutionResult {
                method: method.into(),
                url: url.into(),
                status: "Error".to_string().into(),
                duration: "".into(),
                response: error.into(),
                is_success: false,
                is_running: false,
            },
            state::ExecutionResult::Running { message } => ExecutionResult {
                method: "".into(),
                url: message.into(),
                status: "Running".to_string().into(),
                duration: "".into(),
                response: "".into(),
                is_success: false,
                is_running: true,
            },
        })
        .collect();

    let results_model = Rc::new(VecModel::from(results));
    ui.set_results(ModelRc::from(results_model));
}

fn setup_callbacks(ui: &MainWindow, initial_root: PathBuf) {
    let ui_weak = ui.as_weak();
    let root_dir = Rc::new(std::cell::RefCell::new(initial_root));

    // Open Directory callback
    {
        let ui_weak = ui_weak.clone();
        let root_dir = root_dir.clone();
        ui.on_open_directory(move || {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                *root_dir.borrow_mut() = path.clone();
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_working_directory(path.display().to_string().into());
                    let files = discover_http_files(&path);
                    let files_model = Rc::new(VecModel::from(files));
                    ui.set_http_files(ModelRc::from(files_model));
                    ui.set_selected_file("".into());
                    ui.set_requests(ModelRc::default());
                    save_state(&ui, &root_dir.borrow());
                }
            }
        });
    }

    // New HTTP File callback
    {
        let ui_weak = ui_weak.clone();
        let root_dir = root_dir.clone();
        ui.on_new_http_file(move || {
            let root = root_dir.borrow();
            if let Some(path) = rfd::FileDialog::new()
                .set_directory(&*root)
                .add_filter("HTTP Files", &["http"])
                .set_file_name("new.http")
                .save_file()
            {
                if std::fs::write(&path, "### New Request\nGET https://httpbin.org/get\n").is_ok() {
                    if let Some(ui) = ui_weak.upgrade() {
                        let files = discover_http_files(&*root);
                        let files_model = Rc::new(VecModel::from(files));
                        ui.set_http_files(ModelRc::from(files_model));
                        ui.set_selected_file(path.display().to_string().into());
                        load_file_requests(&ui, &path);
                        load_environments(&ui, &path);
                        save_state(&ui, &*root);
                    }
                }
            }
        });
    }

    // Select File callback
    {
        let ui_weak = ui_weak.clone();
        let root_dir = root_dir.clone();
        ui.on_select_file(move |file_path| {
            if let Some(ui) = ui_weak.upgrade() {
                let path = PathBuf::from(file_path.as_str());
                ui.set_selected_file(file_path);
                load_file_requests(&ui, &path);
                load_environments(&ui, &path);
                save_state(&ui, &root_dir.borrow());
            }
        });
    }

    // Select Environment callback
    {
        let ui_weak = ui_weak.clone();
        let root_dir = root_dir.clone();
        ui.on_select_environment(move |index| {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_selected_environment_index(index);
                save_state(&ui, &root_dir.borrow());
            }
        });
    }

    // Run All Requests callback
    {
        let ui_weak = ui_weak.clone();
        let root_dir = root_dir.clone();
        ui.on_run_all_requests(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let file_path = ui.get_selected_file();
                if file_path.is_empty() {
                    return;
                }

                let env_idx = ui.get_selected_environment_index();
                let environment = if env_idx > 0 {
                    let envs = ui.get_environments();
                    envs.row_data(env_idx as usize).map(|s| s.to_string())
                } else {
                    None
                };

                run_all_requests(&ui, &PathBuf::from(file_path.as_str()), environment);
                save_state(&ui, &root_dir.borrow());
            }
        });
    }

    // Run Single Request callback
    {
        let ui_weak = ui_weak.clone();
        let root_dir = root_dir.clone();
        ui.on_run_single_request(move |request_index| {
            if let Some(ui) = ui_weak.upgrade() {
                let file_path = ui.get_selected_file();
                if file_path.is_empty() {
                    return;
                }

                let env_idx = ui.get_selected_environment_index();
                let environment = if env_idx > 0 {
                    let envs = ui.get_environments();
                    envs.row_data(env_idx as usize).map(|s| s.to_string())
                } else {
                    None
                };

                run_single_request(
                    &ui,
                    &PathBuf::from(file_path.as_str()),
                    request_index as usize - 1,
                    environment,
                );
                save_state(&ui, &root_dir.borrow());
            }
        });
    }

    // Quit callback
    {
        let ui_weak = ui_weak.clone();
        let root_dir = root_dir.clone();
        ui.on_quit(move || {
            if let Some(ui) = ui_weak.upgrade() {
                save_state(&ui, &root_dir.borrow());
                ui.hide().ok();
            }
        });
    }
}

fn run_all_requests(ui: &MainWindow, file_path: &PathBuf, environment: Option<String>) {
    // Clear previous results and show running indicator
    let running_result = ExecutionResult {
        method: "".into(),
        url: format!("Running all requests from {}...", file_path.display()).into(),
        status: "Running".into(),
        duration: "".into(),
        response: "".into(),
        is_success: false,
        is_running: true,
    };
    let results_model = Rc::new(VecModel::from(vec![running_result]));
    ui.set_results(ModelRc::from(results_model.clone()));

    // Run requests
    if let Some(path_str) = file_path.to_str() {
        let files = vec![path_str.to_string()];
        match httprunner_lib::processor::process_http_files(
            &files,
            false,
            None,
            environment.as_deref(),
            false,
            false,
        ) {
            Ok(processor_results) => {
                results_model.set_vec(Vec::new());
                for file_result in processor_results.files {
                    for request_context in file_result.result_contexts {
                        if let Some(http_result) = request_context.result {
                            let result = if http_result.success {
                                ExecutionResult {
                                    method: request_context.request.method.into(),
                                    url: request_context.request.url.into(),
                                    status: http_result.status_code.to_string().into(),
                                    duration: format!("{}ms", http_result.duration_ms).into(),
                                    response: http_result.body.into(),
                                    is_success: true,
                                    is_running: false,
                                }
                            } else {
                                ExecutionResult {
                                    method: request_context.request.method.into(),
                                    url: request_context.request.url.into(),
                                    status: http_result.status_code.to_string().into(),
                                    duration: format!("{}ms", http_result.duration_ms).into(),
                                    response: http_result.body.into(),
                                    is_success: false,
                                    is_running: false,
                                }
                            };
                            results_model.push(result);
                        } else {
                            let result = ExecutionResult {
                                method: request_context.request.method.into(),
                                url: request_context.request.url.into(),
                                status: "Error".into(),
                                duration: "".into(),
                                response: "Request failed".into(),
                                is_success: false,
                                is_running: false,
                            };
                            results_model.push(result);
                        }
                    }
                }
            }
            Err(e) => {
                results_model.set_vec(vec![ExecutionResult {
                    method: "ERROR".into(),
                    url: "".into(),
                    status: "Error".into(),
                    duration: "".into(),
                    response: format!("Failed to run requests: {}", e).into(),
                    is_success: false,
                    is_running: false,
                }]);
            }
        }
    }
}

fn run_single_request(
    ui: &MainWindow,
    file_path: &PathBuf,
    request_index: usize,
    environment: Option<String>,
) {
    // Parse the file to get the specific request
    if let Ok(requests) = httprunner_lib::parser::parse_http_file(file_path.to_str().unwrap()) {
        if request_index < requests.len() {
            let request = &requests[request_index];

            // Show running indicator
            let running_result = ExecutionResult {
                method: request.method.clone().into(),
                url: request.url.clone().into(),
                status: "Running".into(),
                duration: "".into(),
                response: "".into(),
                is_success: false,
                is_running: true,
            };

            // Get current results and add running indicator
            let current_results = ui.get_results();
            let mut results_vec: Vec<ExecutionResult> = (0..current_results.row_count())
                .filter_map(|i| current_results.row_data(i))
                .collect();
            results_vec.push(running_result);

            let results_model = Rc::new(VecModel::from(results_vec));
            ui.set_results(ModelRc::from(results_model.clone()));

            // Execute the request
            match httprunner_lib::executor::execute_request(
                request,
                file_path.to_str().unwrap(),
                environment.as_deref(),
                false,
            ) {
                Ok(http_result) => {
                    // Remove running indicator
                    let mut results_vec: Vec<ExecutionResult> = (0..results_model.row_count())
                        .filter_map(|i| results_model.row_data(i))
                        .filter(|r| !r.is_running)
                        .collect();

                    let result = if http_result.success {
                        ExecutionResult {
                            method: request.method.clone().into(),
                            url: request.url.clone().into(),
                            status: http_result.status_code.to_string().into(),
                            duration: format!("{}ms", http_result.duration_ms).into(),
                            response: http_result.body.into(),
                            is_success: true,
                            is_running: false,
                        }
                    } else {
                        ExecutionResult {
                            method: request.method.clone().into(),
                            url: request.url.clone().into(),
                            status: http_result.status_code.to_string().into(),
                            duration: format!("{}ms", http_result.duration_ms).into(),
                            response: http_result.body.into(),
                            is_success: false,
                            is_running: false,
                        }
                    };
                    results_vec.push(result);
                    results_model.set_vec(results_vec);
                }
                Err(e) => {
                    // Remove running indicator
                    let mut results_vec: Vec<ExecutionResult> = (0..results_model.row_count())
                        .filter_map(|i| results_model.row_data(i))
                        .filter(|r| !r.is_running)
                        .collect();

                    let result = ExecutionResult {
                        method: request.method.clone().into(),
                        url: request.url.clone().into(),
                        status: "Error".into(),
                        duration: "".into(),
                        response: format!("Failed to execute request: {}", e).into(),
                        is_success: false,
                        is_running: false,
                    };
                    results_vec.push(result);
                    results_model.set_vec(results_vec);
                }
            }
        }
    }
}

fn save_state(ui: &MainWindow, root_directory: &PathBuf) {
    // Get environment name if selected
    let selected_environment = {
        let env_idx = ui.get_selected_environment_index();
        if env_idx > 0 {
            let envs = ui.get_environments();
            envs.row_data(env_idx as usize).map(|s| s.to_string())
        } else {
            None
        }
    };

    // Get selected file
    let selected_file = {
        let file_path = ui.get_selected_file();
        if !file_path.is_empty() {
            Some(PathBuf::from(file_path.as_str()))
        } else {
            None
        }
    };

    // Get current results (filter out running ones)
    let last_results = {
        let results = ui.get_results();
        (0..results.row_count())
            .filter_map(|i| results.row_data(i))
            .filter(|r| !r.is_running)
            .map(|r| {
                if r.is_success {
                    state::ExecutionResult::Success {
                        method: r.method.to_string(),
                        url: r.url.to_string(),
                        status: r.status.parse().unwrap_or(200),
                        duration_ms: r
                            .duration
                            .to_string()
                            .trim_end_matches("ms")
                            .parse()
                            .unwrap_or(0),
                        response_body: r.response.to_string(),
                        assertion_results: Vec::new(),
                    }
                } else {
                    state::ExecutionResult::Failure {
                        method: r.method.to_string(),
                        url: r.url.to_string(),
                        error: r.response.to_string(),
                    }
                }
            })
            .collect()
    };

    let state = state::AppState {
        root_directory: Some(root_directory.clone()),
        selected_file,
        selected_environment,
        font_size: None, // Not applicable in Slint version
        window_size: None, // Slint handles this automatically
        last_results: Some(last_results),
    };

    if let Err(e) = state.save() {
        eprintln!("Failed to save application state: {}", e);
    }
}
