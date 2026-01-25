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
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // File tree
            Constraint::Percentage(40), // Request view
            Constraint::Percentage(35), // Results view
        ])
        .split(area);

    render_file_tree(f, chunks[0], app);
    render_request_view(f, chunks[1], app);
    render_results_view(f, chunks[2], app);
}

fn render_file_tree(f: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.focused_pane == FocusedPane::FileTree;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let files = app.file_tree.files();
    let items: Vec<ListItem> = files
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

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Files [Tab to switch]")
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
        let empty_msg = Paragraph::new("No requests loaded\n\nSelect a file from the left panel")
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
            .title(format!(
                "Requests ({}) [F5/Ctrl+R to run all]",
                requests.len()
            ))
            .border_style(border_style),
    );

    f.render_widget(list, area);
}

fn render_results_view(f: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.focused_pane == FocusedPane::ResultsView;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

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
        let empty_msg = Paragraph::new("No results yet\n\nPress F5 or Ctrl+R to run all requests")
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

fn render_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let status_text = vec![
        Line::from(vec![
            Span::raw("Status: "),
            Span::styled(&app.status_message, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("Shortcuts: "),
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(" = Switch Pane | "),
            Span::styled("F5/Ctrl+R", Style::default().fg(Color::Yellow)),
            Span::raw(" = Run All | "),
            Span::styled("Ctrl+E", Style::default().fg(Color::Yellow)),
            Span::raw(" = Cycle Env | "),
            Span::styled("Ctrl+Q", Style::default().fg(Color::Yellow)),
            Span::raw(" = Quit"),
        ]),
    ];

    let status = Paragraph::new(status_text).block(Block::default().borders(Borders::ALL));

    f.render_widget(status, area);
}
