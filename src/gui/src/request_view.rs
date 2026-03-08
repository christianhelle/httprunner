use crate::request_editor::{EditableRequest, RequestEditor};
use dioxus::prelude::*;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub enum RequestViewAction {
    RunRequest(usize),
    SaveFile,
    None,
}

#[derive(Clone, Debug, Default)]
pub struct RequestViewState {
    pub editor: RequestEditor,
    pub is_editing: bool,
}

impl RequestViewState {
    pub fn load_file(&mut self, path: &std::path::Path) {
        self.editor.load_file(path);
        self.is_editing = false;
    }

    pub fn save_to_file(&mut self) -> anyhow::Result<()> {
        self.editor.save_to_file()
    }

    pub fn has_changes(&self) -> bool {
        self.editor.has_changes()
    }
}

#[component]
pub fn RequestView(
    mut state: Signal<RequestViewState>,
    file: Signal<Option<PathBuf>>,
    on_action: EventHandler<RequestViewAction>,
) -> Element {
    let s = state();

    if file().is_none() {
        return rsx! {
            p { style: "color: #8087a2;", "No file selected. Select a .http file from the left panel." }
        };
    }

    // Show editor form if editing
    if s.is_editing {
        return rsx! { RequestEditForm { state, on_action } };
    }

    let requests = s.editor.get_requests().to_vec();

    if requests.is_empty() {
        return rsx! {
            div {
                p { style: "color: #8087a2;", "No requests found in this file." }
                hr {}
                button {
                    onclick: move |_| {
                        state.write().editor.start_new_request();
                        state.write().is_editing = true;
                    },
                    "➕ Add New Request"
                }
            }
        };
    }

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 6px;",
            for (idx, request) in requests.iter().enumerate() {
                {
                    let method = request.method.clone();
                    let url = request.url.clone();
                    let name = request.name.clone();
                    let headers = request.headers.clone();
                    let body = request.body.clone();
                    let header_text = if let Some(ref n) = name {
                        format!("{} - {} {}", idx + 1, method, n)
                    } else {
                        format!("{} - {} {}", idx + 1, method, url)
                    };
                    rsx! {
                        CollapsibleRequest {
                            key: "{idx}",
                            idx,
                            header_text,
                            method,
                            url,
                            headers,
                            body,
                            on_run: move |i: usize| on_action.call(RequestViewAction::RunRequest(i)),
                            on_edit: move |i: usize| {
                                state.write().editor.start_editing(i);
                                state.write().is_editing = true;
                            },
                            on_delete: move |i: usize| {
                                state.write().editor.delete_request(i);
                                on_action.call(RequestViewAction::SaveFile);
                            },
                        }
                    }
                }
            }
            hr {}
            button {
                onclick: move |_| {
                    state.write().editor.start_new_request();
                    state.write().is_editing = true;
                },
                "➕ Add New Request"
            }
        }
    }
}

#[component]
fn CollapsibleRequest(
    idx: usize,
    header_text: String,
    method: String,
    url: String,
    headers: Vec<httprunner_core::types::Header>,
    body: Option<String>,
    on_run: EventHandler<usize>,
    on_edit: EventHandler<usize>,
    on_delete: EventHandler<usize>,
) -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        div {
            class: "collapsible",
            // Header
            div {
                class: "collapse-header",
                onclick: move |_| { let v = open(); open.set(!v); },
                span { if open() { "▼ " } else { "▶ " } }
                span { "{header_text}" }
            }
            // Content
            if open() {
                div {
                    class: "collapse-content",
                    style: "background: #1e2030; border-radius: 4px; padding: 8px; margin-top: 4px;",
                    div { class: "form-row",
                        label { "Method:" }
                        span { class: "mono", "{method}" }
                    }
                    div { class: "form-row",
                        label { "URL:" }
                        span { class: "mono", style: "word-break: break-all;", "{url}" }
                    }
                    if !headers.is_empty() {
                        div {
                            p { class: "section-title", style: "margin-bottom: 4px;", "Headers:" }
                            for h in headers.iter() {
                                p { class: "mono", style: "font-size: 12px;", "{h.name}: {h.value}" }
                            }
                        }
                    }
                    if let Some(ref b) = body {
                        if !b.trim().is_empty() {
                            div {
                                p { class: "section-title", style: "margin: 4px 0;", "Body:" }
                                div { class: "code-block", pre { "{b}" } }
                            }
                        }
                    }
                    hr {}
                    div { class: "flex items-center gap-8",
                        button {
                            style: "background: #a6da95; color: #24273a;",
                            onclick: move |_| on_run.call(idx),
                            "▶ Run"
                        }
                        button {
                            onclick: move |_| on_edit.call(idx),
                            "✏ Edit"
                        }
                        button {
                            style: "background: #ed8796; color: #24273a;",
                            onclick: move |_| on_delete.call(idx),
                            "🗑 Delete"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RequestEditForm(
    mut state: Signal<RequestViewState>,
    on_action: EventHandler<RequestViewAction>,
) -> Element {
    // Local copies of fields for controlled inputs
    let req = state().editor.get_editing_request().cloned();
    if req.is_none() {
        return rsx! { p { "No request being edited." } };
    }

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 8px;",
            h2 { "Edit Request" }
            hr {}

            div { class: "form-row",
                label { "Name:" }
                input {
                    r#type: "text",
                    style: "flex: 1;",
                    placeholder: "Optional name",
                    value: "{state().editor.get_editing_request().map(|r| r.name.as_str()).unwrap_or(\"\")}",
                    oninput: move |e| {
                        if let Some(req) = state.write().editor.get_editing_request_mut() {
                            req.name = e.value();
                        }
                    },
                }
            }

            div { class: "form-row",
                label { "Method:" }
                select {
                    style: "flex: 1;",
                    value: "{state().editor.get_editing_request().map(|r| r.method.as_str()).unwrap_or(\"GET\")}",
                    onchange: move |e| {
                        if let Some(req) = state.write().editor.get_editing_request_mut() {
                            req.method = e.value();
                        }
                    },
                    for m in ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"] {
                        option { value: "{m}", "{m}" }
                    }
                }
            }

            div { class: "form-row",
                label { "URL:" }
                input {
                    r#type: "text",
                    style: "flex: 1;",
                    placeholder: "https://example.com/api",
                    value: "{state().editor.get_editing_request().map(|r| r.url.as_str()).unwrap_or(\"\")}",
                    oninput: move |e| {
                        if let Some(req) = state.write().editor.get_editing_request_mut() {
                            req.url = e.value();
                        }
                    },
                }
            }

            hr {}
            p { class: "section-title", "Headers:" }

            {
                let header_count = state().editor.get_editing_request().map(|r| r.headers.len()).unwrap_or(0);
                rsx! {
                    for i in 0..header_count {
                        div {
                            key: "{i}",
                            class: "flex items-center gap-8",
                            style: "margin-bottom: 4px;",
                            input {
                                r#type: "text",
                                style: "flex: 1;",
                                placeholder: "Header name",
                                value: "{state().editor.get_editing_request().and_then(|r| r.headers.get(i)).map(|(n, _)| n.as_str()).unwrap_or(\"\")}",
                                oninput: move |e| {
                                    if let Some(req) = state.write().editor.get_editing_request_mut() {
                                        if let Some(h) = req.headers.get_mut(i) {
                                            h.0 = e.value();
                                        }
                                    }
                                },
                            }
                            input {
                                r#type: "text",
                                style: "flex: 1;",
                                placeholder: "Header value",
                                value: "{state().editor.get_editing_request().and_then(|r| r.headers.get(i)).map(|(_, v)| v.as_str()).unwrap_or(\"\")}",
                                oninput: move |e| {
                                    if let Some(req) = state.write().editor.get_editing_request_mut() {
                                        if let Some(h) = req.headers.get_mut(i) {
                                            h.1 = e.value();
                                        }
                                    }
                                },
                            }
                            button {
                                onclick: move |_| {
                                    if let Some(req) = state.write().editor.get_editing_request_mut() {
                                        req.headers.remove(i);
                                    }
                                },
                                "🗑"
                            }
                        }
                    }
                }
            }

            button {
                onclick: move |_| {
                    if let Some(req) = state.write().editor.get_editing_request_mut() {
                        req.headers.push((String::new(), String::new()));
                    }
                },
                "➕ Add Header"
            }

            hr {}
            p { class: "section-title", "Body:" }
            textarea {
                style: "height: 150px; width: 100%;",
                value: "{state().editor.get_editing_request().map(|r| r.body.as_str()).unwrap_or(\"\")}",
                oninput: move |e| {
                    if let Some(req) = state.write().editor.get_editing_request_mut() {
                        req.body = e.value();
                    }
                },
            }

            hr {}
            div { class: "flex items-center gap-8",
                button {
                    style: "background: #a6da95; color: #24273a;",
                    onclick: move |_| {
                        if state.write().editor.save_current_edit() {
                            state.write().is_editing = false;
                            on_action.call(RequestViewAction::SaveFile);
                        }
                    },
                    "💾 Save"
                }
                button {
                    onclick: move |_| {
                        state.write().editor.cancel_editing();
                        state.write().is_editing = false;
                    },
                    "❌ Cancel"
                }
            }
        }
    }
}
