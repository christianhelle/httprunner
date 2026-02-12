use crate::app::{App, FocusedPane};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title bar
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(size);

    render_title(f, chunks[0], app);
    render_main_content(f, chunks[1], app);
    render_status_bar(f, chunks[2], app);
}

fn render_title(f: &mut Frame, area: Rect, app: &App) {
    let title_text = vec![
        Line::from(vec![
            Span::styled("HTTP ", Style::default().fg(Color::Cyan)),
            Span::styled("File ", Style::default().fg(Color::Green)),
            Span::styled("Runner ", Style::default().fg(Color::Yellow)),
            Span::styled("- TUI", Style::default().fg(Color::Magenta)),
        ]),
        Line::from(vec![
            Span::raw("Environment: "),
            Span::styled(
                app.selected_environment.as_deref().unwrap_or("None"),
                Style::default().fg(Color::Yellow),
            ),
        ]),
    ];

    let title = Paragraph::new(title_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

    f.render_widget(title, area);
}

fn render_main_content(f: &mut Frame, area: Rect, app: &App) {
    if app.file_tree_visible {
        let h_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // File tree
                Constraint::Percentage(75), // Request + Environment + Results
            ])
            .split(area);

        render_file_tree(f, h_chunks[0], app);

        let v_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // Request view + Environment editor
                Constraint::Percentage(50), // Results
            ])
            .split(h_chunks[1]);

        let top_h_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Request view
                Constraint::Percentage(50), // Environment editor
            ])
            .split(v_chunks[0]);

        render_request_view(f, top_h_chunks[0], app);
        render_environment_editor(f, top_h_chunks[1], app);
        render_results_view(f, v_chunks[1], app);
    } else {
        let v_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // Request view + Environment editor
                Constraint::Percentage(50), // Results
            ])
            .split(area);

        let top_h_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Request view
                Constraint::Percentage(50), // Environment editor
            ])
            .split(v_chunks[0]);

        render_request_view(f, top_h_chunks[0], app);
        render_environment_editor(f, top_h_chunks[1], app);
        render_results_view(f, v_chunks[1], app);
    }
}

fn render_file_tree(f: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.focused_pane == FocusedPane::FileTree;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let is_discovering = app.file_tree.is_discovering();
    let discovered_count = app.file_tree.discovered_count();

    let title = "Files [↑/↓/j/k to navigate]".to_string();

    let files = app.file_tree.files();
    let mut items: Vec<ListItem> = files
        .iter()
        .enumerate()
        .map(|(i, path)| {
            let display_name = path
                .strip_prefix(app.file_tree.root())
                .unwrap_or(path)
                .display()
                .to_string();

            let style = if i == app.file_tree.selected_index() {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(display_name).style(style)
        })
        .collect();

    // Add discovering indicator at the top if still scanning
    if is_discovering {
        items.insert(
            0,
            ListItem::new(Line::from(vec![
                Span::styled("⏳ ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("Scanning... ({})", discovered_count),
                    Style::default().fg(Color::Gray),
                ),
            ])),
        );
    }

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style),
    );

    f.render_widget(list, area);
}

fn render_request_view(f: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.focused_pane == FocusedPane::RequestView;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let requests = app.request_view.requests();

    if requests.is_empty() {
        let hint = if app.file_tree_visible {
            "No requests loaded\n\nSelect a file from the left panel"
        } else {
            "No requests loaded\n\nPress Ctrl+B to show file list"
        };
        let empty_msg = Paragraph::new(hint)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Requests [Enter to run]")
                    .border_style(border_style),
            )
            .alignment(Alignment::Center);
        f.render_widget(empty_msg, area);
        return;
    }

    let items: Vec<ListItem> = requests
        .iter()
        .enumerate()
        .map(|(i, req)| {
            let name = req.name.as_deref().unwrap_or("Unnamed Request");
            let method_color = match req.method.as_str() {
                "GET" => Color::Green,
                "POST" => Color::Yellow,
                "PUT" => Color::Blue,
                "DELETE" => Color::Red,
                "PATCH" => Color::Magenta,
                _ => Color::White,
            };

            let content = vec![
                Line::from(vec![
                    Span::styled(&req.method, Style::default().fg(method_color)),
                    Span::raw(" "),
                    Span::raw(name),
                ]),
                Line::from(Span::styled(&req.url, Style::default().fg(Color::Gray))),
            ];

            let style = if i == app.request_view.selected_index() {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Requests ({}) [R/F5 to run all]", requests.len()))
            .border_style(border_style),
    );

    f.render_widget(list, area);
}

fn render_results_view(f: &mut Frame, area: Rect, app: &App) {
    use crate::results_view::ExecutionResult;
    use httprunner_core::types::AssertionType;

    let is_focused = app.focused_pane == FocusedPane::ResultsView;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let is_running = app.results_view.is_running();
    let incremental_results = app.results_view.get_incremental_results();
    let compact_mode = app.results_view.is_compact_mode();

    // Check if we have incremental results (from async execution)
    if !incremental_results.is_empty() || is_running {
        let mut lines = Vec::new();

        // Show mode indicator
        lines.push(Line::from(vec![
            if compact_mode {
                Span::styled(
                    "📋 Compact",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    "📄 Verbose",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            },
            Span::styled(" (Ctrl+D to toggle)", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(""));

        // Show running indicator
        if is_running {
            lines.push(Line::from(vec![
                Span::styled("⏳ ", Style::default().fg(Color::Yellow)),
                Span::styled("Executing requests...", Style::default().fg(Color::Cyan)),
            ]));
            lines.push(Line::from(""));
        }

        // Show incremental results
        for result in &incremental_results {
            match result {
                ExecutionResult::Success {
                    method,
                    url,
                    status,
                    duration_ms,
                    request_body,
                    response_body,
                    assertion_results,
                } => {
                    // Main result line (same for compact and verbose)
                    lines.push(Line::from(vec![
                        Span::styled("✓ ", Style::default().fg(Color::Green)),
                        Span::raw(format!("{} {} ", method, url)),
                        Span::styled(
                            format!("| {} | {}ms", status, duration_ms),
                            Style::default().fg(Color::Gray),
                        ),
                    ]));

                    // In Verbose mode, show in order: Assertion Results -> Request Body -> Response Body

                    // 1. Show assertion results
                    if !assertion_results.is_empty() {
                        for assertion in assertion_results {
                            let assertion_type_str = match assertion.assertion.assertion_type {
                                AssertionType::Status => "Status",
                                AssertionType::Body => "Body",
                                AssertionType::Headers => "Headers",
                            };

                            if assertion.passed {
                                lines.push(Line::from(vec![
                                    Span::raw("  "),
                                    Span::styled("✓ ", Style::default().fg(Color::Green)),
                                    Span::raw(format!(
                                        "{}: Expected '{}'",
                                        assertion_type_str, assertion.assertion.expected_value
                                    )),
                                ]));
                            } else {
                                lines.push(Line::from(vec![
                                    Span::raw("  "),
                                    Span::styled("✗ ", Style::default().fg(Color::Red)),
                                    Span::styled(
                                        format!(
                                            "{}: {}",
                                            assertion_type_str,
                                            assertion
                                                .error_message
                                                .as_ref()
                                                .unwrap_or(&"Failed".to_string())
                                        ),
                                        Style::default().fg(Color::Red),
                                    ),
                                ]));

                                if let Some(ref actual) = assertion.actual_value {
                                    lines.push(Line::from(vec![
                                        Span::raw("    "),
                                        Span::styled(
                                            format!(
                                                "Expected: '{}', Actual: '{}'",
                                                assertion.assertion.expected_value, actual
                                            ),
                                            Style::default().fg(Color::Yellow),
                                        ),
                                    ]));
                                }
                            }
                        }
                    }

                    // 2. Show request body in verbose mode (skip if empty or whitespace only)
                    if !compact_mode
                        && let Some(req_body) = request_body
                        && !req_body.trim().is_empty()
                    {
                        lines.push(Line::from(""));
                        lines.push(Line::from(vec![Span::styled(
                            "  Request Body:",
                            Style::default().add_modifier(Modifier::BOLD),
                        )]));
                        // Show first few lines of request body (truncate for TUI)
                        let line_count = req_body.lines().count();
                        for (i, line) in req_body.lines().take(5).enumerate() {
                            lines.push(Line::from(vec![
                                Span::raw("    "),
                                Span::styled(line.to_string(), Style::default().fg(Color::Cyan)),
                            ]));
                            if i == 4 && line_count > 5 {
                                lines.push(Line::from(vec![
                                    Span::raw("    "),
                                    Span::styled(
                                        "... (truncated)",
                                        Style::default().fg(Color::DarkGray),
                                    ),
                                ]));
                            }
                        }
                    }

                    // 3. Show response body in verbose mode (skip if empty or whitespace only)
                    if !compact_mode && !response_body.trim().is_empty() {
                        lines.push(Line::from(""));
                        lines.push(Line::from(vec![Span::styled(
                            "  Response:",
                            Style::default().add_modifier(Modifier::BOLD),
                        )]));
                        // Show first few lines of response (truncate for TUI)
                        let line_count = response_body.lines().count();
                        for (i, line) in response_body.lines().take(10).enumerate() {
                            lines.push(Line::from(vec![
                                Span::raw("    "),
                                Span::styled(line.to_string(), Style::default().fg(Color::Gray)),
                            ]));
                            if i == 9 && line_count > 10 {
                                lines.push(Line::from(vec![
                                    Span::raw("    "),
                                    Span::styled(
                                        "... (truncated)",
                                        Style::default().fg(Color::DarkGray),
                                    ),
                                ]));
                            }
                        }
                    }
                    lines.push(Line::from(""));
                }
                ExecutionResult::Failure { method, url, error } => {
                    lines.push(Line::from(vec![
                        Span::styled("✗ ", Style::default().fg(Color::Red)),
                        Span::raw(format!("{} {}", method, url)),
                    ]));
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(error, Style::default().fg(Color::Red)),
                    ]));
                    lines.push(Line::from(""));
                }
            }
        }

        // Summary at the bottom
        if !is_running && !incremental_results.is_empty() {
            lines.push(Line::from(""));
            let passed = incremental_results
                .iter()
                .filter(|r| matches!(r, ExecutionResult::Success { .. }))
                .count();
            let failed = incremental_results
                .iter()
                .filter(|r| matches!(r, ExecutionResult::Failure { .. }))
                .count();
            lines.push(Line::from(vec![
                Span::styled("Passed: ", Style::default().fg(Color::Green)),
                Span::raw(format!("{} | ", passed)),
                Span::styled("Failed: ", Style::default().fg(Color::Red)),
                Span::raw(format!("{}", failed)),
            ]));
        }

        let title = if is_running {
            "Results [Running...]"
        } else {
            "Results"
        };

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(border_style),
            )
            .wrap(Wrap { trim: true })
            .scroll((app.results_view.scroll_offset() as u16, 0));

        f.render_widget(paragraph, area);
        return;
    }

    let results = app.results_view.results();

    if let Some(results) = results {
        let mut lines = Vec::new();

        // Calculate totals
        let total_files = results.files.len();
        let total_passed: u32 = results.files.iter().map(|f| f.success_count).sum();
        let total_failed: u32 = results.files.iter().map(|f| f.failed_count).sum();
        let total_skipped: u32 = results.files.iter().map(|f| f.skipped_count).sum();

        // Summary
        lines.push(Line::from(vec![
            Span::styled(
                "Total Files: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{} | ", total_files)),
            Span::styled("Passed: ", Style::default().fg(Color::Green)),
            Span::raw(format!("{} | ", total_passed)),
            Span::styled("Failed: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{} | ", total_failed)),
            Span::styled("Skipped: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", total_skipped)),
        ]));
        lines.push(Line::from(""));

        // File results
        for file_result in &results.files {
            let status_color = if file_result.failed_count == 0 {
                Color::Green
            } else {
                Color::Red
            };

            lines.push(Line::from(vec![
                Span::styled(
                    if file_result.failed_count == 0 {
                        "✓"
                    } else {
                        "✗"
                    },
                    Style::default().fg(status_color),
                ),
                Span::raw(" "),
                Span::raw(&file_result.filename),
            ]));

            for context in &file_result.result_contexts {
                if let Some(result) = &context.result {
                    let result_color = if result.success {
                        Color::Green
                    } else {
                        Color::Red
                    };

                    let method = &context.request.method;
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            if result.success { "✓" } else { "✗" },
                            Style::default().fg(result_color),
                        ),
                        Span::raw(" "),
                        Span::raw(format!("{} {}", method, &context.name)),
                    ]));

                    if let Some(error) = &result.error_message {
                        lines.push(Line::from(vec![
                            Span::raw("    "),
                            Span::styled(error, Style::default().fg(Color::Red)),
                        ]));
                    }
                } else {
                    // Skipped request
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled("-", Style::default().fg(Color::Yellow)),
                        Span::raw(" "),
                        Span::styled(&context.name, Style::default().fg(Color::Gray)),
                        Span::raw(" (skipped)"),
                    ]));
                }
            }
            lines.push(Line::from(""));
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Results")
                    .border_style(border_style),
            )
            .wrap(Wrap { trim: true })
            .scroll((app.results_view.scroll_offset() as u16, 0));

        f.render_widget(paragraph, area);
    } else {
        let empty_msg = Paragraph::new("No results yet\n\nPress R or F5 to run all requests")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Results")
                    .border_style(border_style),
            )
            .alignment(Alignment::Center);
        f.render_widget(empty_msg, area);
    }
}

fn render_environment_editor(f: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.focused_pane == FocusedPane::EnvironmentEditor;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    let editor = &app.environment_editor;

    let mut lines = Vec::new();

    // Help text
    lines.push(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(" Switch | "),
        Span::styled("n", Style::default().fg(Color::Yellow)),
        Span::raw(" New Env | "),
        Span::styled("a", Style::default().fg(Color::Yellow)),
        Span::raw(" Add Var | "),
        Span::styled("e/Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" Edit | "),
        Span::styled("d", Style::default().fg(Color::Yellow)),
        Span::raw(" Delete | "),
        Span::styled("Ctrl+S", Style::default().fg(Color::Yellow)),
        Span::raw(" Save"),
    ]));
    lines.push(Line::from(""));

    let env_names = editor.env_names();

    if env_names.is_empty() {
        lines.push(Line::from(Span::styled(
            "No environments defined. Press 'n' to add one.",
            Style::default().fg(Color::Gray),
        )));
    } else {
        // Show environments list
        lines.push(Line::from(Span::styled(
            "Environments:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));

        for (i, env_name) in env_names.iter().enumerate() {
            let is_selected = i == editor.selected_env_index();
            let marker = if is_selected && editor.is_env_list_focused() {
                "▸ "
            } else if is_selected {
                "› "
            } else {
                "  "
            };
            let style = if is_selected && editor.is_env_list_focused() {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Gray)
            };
            lines.push(Line::from(vec![
                Span::raw(marker),
                Span::styled(env_name, style),
            ]));
        }

        lines.push(Line::from(""));

        // Show variables for selected environment
        if let Some(selected_env) = editor.selected_env_name() {
            lines.push(Line::from(vec![Span::styled(
                format!("Variables for '{}':", selected_env),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]));

            let var_names = editor.var_names();
            if var_names.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  No variables. Press 'a' to add one.",
                    Style::default().fg(Color::Gray),
                )));
            } else {
                for (i, var_name) in var_names.iter().enumerate() {
                    let is_selected = i == editor.selected_var_index();
                    let marker = if is_selected && editor.is_var_list_focused() {
                        "▸ "
                    } else if is_selected {
                        "› "
                    } else {
                        "  "
                    };
                    let style = if is_selected && editor.is_var_list_focused() {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    let value = editor.get_var_value(selected_env, var_name).unwrap_or("");

                    lines.push(Line::from(vec![
                        Span::raw(marker),
                        Span::styled(format!("{}: ", var_name), style),
                        Span::styled(value, Style::default().fg(Color::Cyan)),
                    ]));
                }
            }
        }
    }

    // Show input prompt if in input mode
    if editor.is_in_input_mode() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(editor.input_prompt(), Style::default().fg(Color::Yellow)),
            Span::styled(editor.input_buffer(), Style::default().fg(Color::White)),
            Span::styled("▌", Style::default().fg(Color::White)),
        ]));
    }

    // Show unsaved changes indicator
    if editor.has_changes() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "● Unsaved changes (Ctrl+S to save)",
            Style::default().fg(Color::Yellow),
        )));
    }

    let title = if editor.has_changes() {
        "🌍 Environment Editor [modified]"
    } else {
        "🌍 Environment Editor"
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_status_bar(f: &mut Frame, area: Rect, app: &App) {
    // Get support key
    let support_key_text = match httprunner_core::logging::get_support_key() {
        Ok(key) => format!("Support: {}", key.short_key),
        Err(_) => String::new(),
    };

    // Telemetry status indicator
    let telemetry_indicator = if app.telemetry_enabled {
        Span::styled("📊", Style::default().fg(Color::Green))
    } else {
        Span::styled("📊", Style::default().fg(Color::DarkGray))
    };

    let status_text = vec![
        Line::from(vec![
            Span::raw("Status: "),
            Span::styled(&app.status_message, Style::default().fg(Color::Cyan)),
            if !support_key_text.is_empty() {
                Span::raw(" | ")
            } else {
                Span::raw("")
            },
            Span::styled(support_key_text, Style::default().fg(Color::Blue)),
            Span::raw(" | "),
            telemetry_indicator,
            Span::raw(" | "),
            Span::styled(
                format!("Delay: {}ms", app.delay_ms),
                Style::default().fg(Color::Magenta),
            ),
        ]),
        Line::from(vec![
            Span::styled("R/F5", Style::default().fg(Color::Yellow)),
            Span::raw(" Run | "),
            Span::styled("Q", Style::default().fg(Color::Yellow)),
            Span::raw(" Quit | "),
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(" Pane | "),
            Span::styled("Ctrl+B", Style::default().fg(Color::Yellow)),
            Span::raw(" Files | "),
            Span::styled("Ctrl+D", Style::default().fg(Color::Yellow)),
            Span::raw(" View | "),
            Span::styled("Ctrl+E", Style::default().fg(Color::Yellow)),
            Span::raw(" Env | "),
            Span::styled("Ctrl+T", Style::default().fg(Color::Yellow)),
            Span::raw(" Telemetry | "),
            Span::styled("[/]", Style::default().fg(Color::Yellow)),
            Span::raw(" Delay"),
        ]),
    ];

    let status = Paragraph::new(status_text).block(Block::default().borders(Borders::ALL));

    f.render_widget(status, area);
}
