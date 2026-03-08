use dioxus::prelude::*;
use std::path::PathBuf;

/// Highlight HTTP content and return HTML string with colored spans.
pub fn highlight_http(content: &str) -> String {
    let mut html = String::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        let highlighted = if trimmed.starts_with('#') || trimmed.starts_with("//") {
            format!(
                "<span style='color:#6e738d'>{}</span>",
                html_escape(line)
            )
        } else if trimmed.starts_with("GET ")
            || trimmed.starts_with("POST ")
            || trimmed.starts_with("PUT ")
            || trimmed.starts_with("DELETE ")
            || trimmed.starts_with("PATCH ")
            || trimmed.starts_with("HEAD ")
            || trimmed.starts_with("OPTIONS ")
        {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                format!(
                    "<span style='color:#c6a0f6;font-weight:bold'>{}</span> <span style='color:#8aadf4'>{}</span>",
                    html_escape(parts[0]),
                    html_escape(parts[1])
                )
            } else {
                html_escape(line).to_string()
            }
        } else if line.contains(':') && !trimmed.starts_with('{') && !trimmed.starts_with('[') {
            let pos = line.find(':').unwrap();
            format!(
                "<span style='color:#8bd5ca'>{}</span><span style='color:#cad3f5'>:{}</span>",
                html_escape(&line[..pos]),
                html_escape(&line[pos + 1..])
            )
        } else {
            html_escape(line).to_string()
        };
        html.push_str(&highlighted);
        html.push('\n');
    }
    html
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[component]
pub fn TextEditor(
    file: Signal<Option<PathBuf>>,
    mut content: Signal<String>,
    mut has_changes: Signal<bool>,
    on_save: EventHandler<()>,
) -> Element {
    // Load file when selection changes
    use_effect(move || {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = file().clone() {
            match std::fs::read_to_string(&path) {
                Ok(text) => {
                    content.set(text);
                    has_changes.set(false);
                }
                Err(e) => eprintln!("Failed to load file {}: {}", path.display(), e),
            }
        }
        #[cfg(target_arch = "wasm32")]
        if file().is_some() {
            if let Some(window) = web_sys::window()
                && let Ok(Some(storage)) = window.local_storage()
                && let Ok(Some(saved)) = storage.get_item("httprunner_editor_content")
            {
                content.set(saved);
                has_changes.set(false);
            }
        }
    });

    let highlighted = highlight_http(&content());
    let show_no_file = file().is_none();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%; gap: 6px;",

            if show_no_file {
                p { style: "color: #8087a2;", "No file selected. Select a .http file from the left panel." }
            }

            // Editor wrapper with syntax highlight overlay
            div {
                class: "editor-wrapper",
                style: "flex: 1; position: relative;",

                // Syntax highlight layer (behind)
                pre {
                    style: "
                        position: absolute; top: 0; left: 0; right: 0; bottom: 0;
                        padding: 8px; margin: 0;
                        font-family: 'Consolas', 'JetBrains Mono', monospace;
                        font-size: 13px; line-height: 1.6;
                        white-space: pre-wrap; word-break: break-all;
                        pointer-events: none;
                        background: #1e2030;
                        border: 1px solid #363a4f;
                        border-radius: 4px;
                        overflow: hidden;
                        color: transparent;
                    ",
                    dangerous_inner_html: "{highlighted}",
                }

                // Input layer (on top, transparent text)
                textarea {
                    style: "
                        position: absolute; top: 0; left: 0; right: 0; bottom: 0;
                        padding: 8px;
                        font-family: 'Consolas', 'JetBrains Mono', monospace;
                        font-size: 13px; line-height: 1.6;
                        white-space: pre-wrap; word-break: break-all;
                        background: transparent;
                        color: transparent;
                        caret-color: #cad3f5;
                        border: 1px solid transparent;
                        border-radius: 4px;
                        resize: none;
                        outline: none;
                        width: 100%; height: 100%;
                        overflow: auto;
                        tab-size: 2;
                        spellcheck: false;
                    ",
                    value: "{content()}",
                    oninput: move |e| {
                        content.set(e.value());
                        has_changes.set(true);
                    },
                    spellcheck: "false",
                }
            }

            // Save bar
            div {
                class: "flex items-center gap-8",
                button {
                    onclick: move |_| on_save.call(()),
                    "💾 Save"
                }
                if has_changes() {
                    span { class: "warning", "● Unsaved changes" }
                }
            }
        }
    }
}
