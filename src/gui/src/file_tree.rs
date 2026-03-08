use dioxus::prelude::*;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use walkdir::WalkDir;

#[component]
pub fn FileTree(
    root_path: Signal<PathBuf>,
    selected_file: Signal<Option<PathBuf>>,
    on_file_selected: EventHandler<PathBuf>,
) -> Element {
    let mut http_files: Signal<Vec<PathBuf>> = use_signal(Vec::new);
    let mut is_discovering: Signal<bool> = use_signal(|| true);
    let mut cancel: Signal<Arc<AtomicBool>> = use_signal(|| Arc::new(AtomicBool::new(false)));

    use_effect(move || {
        let root = root_path().clone();

        // Cancel previous discovery
        cancel().store(true, Ordering::SeqCst);
        let new_cancel = Arc::new(AtomicBool::new(false));
        cancel.set(new_cancel.clone());

        http_files.write().clear();
        is_discovering.set(true);

        #[cfg(not(target_arch = "wasm32"))]
        {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Option<PathBuf>>();
            let c = new_cancel;

            std::thread::spawn(move || {
                for entry in WalkDir::new(&root)
                    .follow_links(true)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if c.load(Ordering::SeqCst) { return; }
                    if entry.file_type().is_file()
                        && entry.path().extension().map(|e| e == "http").unwrap_or(false)
                    {
                        tx.send(Some(entry.path().to_path_buf())).ok();
                    }
                }
                tx.send(None).ok();
            });

            spawn(async move {
                while let Some(msg) = rx.recv().await {
                    match msg {
                        Some(path) => {
                            let mut w = http_files.write();
                            w.push(path);
                            w.sort();
                        }
                        None => break,
                    }
                }
                is_discovering.set(false);
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            is_discovering.set(false);
        }
    });

    let files = http_files();
    let discovering = is_discovering();
    let root = root_path().clone();

    // Group files by directory
    let mut dir_map: BTreeMap<Option<PathBuf>, Vec<PathBuf>> = BTreeMap::new();
    for f in &files {
        let parent = f.parent().map(|p| p.to_path_buf());
        dir_map.entry(parent).or_default().push(f.clone());
    }

    rsx! {
        div {
            p {
                class: "section-title",
                style: "margin-bottom: 6px;",
                "HTTP Files"
            }
            hr {}

            if discovering {
                div {
                    class: "flex items-center gap-8",
                    style: "padding: 6px 0; font-size: 12px; color: #8087a2;",
                    span { class: "spinner" }
                    span { "Discovering... ({files.len()} found)" }
                }
            }

            for (dir, dir_files) in dir_map.iter() {
                {
                    let dir_label = if let Some(dp) = &dir {
                        dp.strip_prefix(&root)
                            .unwrap_or(dp.as_path())
                            .display()
                            .to_string()
                    } else {
                        root.display().to_string()
                    };

                    rsx! {
                        div {
                            key: "{dir_label}",
                            div {
                                class: "dir-header",
                                span { "📂 {dir_label}" }
                            }
                            for f in dir_files.iter() {
                                {
                                    let fp = f.clone();
                                    let file_name = fp.file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("?")
                                        .to_string();
                                    let is_selected = selected_file().as_ref() == Some(&fp);
                                    let fc = fp.clone();
                                    rsx! {
                                        div {
                                            key: "{file_name}",
                                            class: if is_selected { "file-item selected" } else { "file-item" },
                                            onclick: move |_| on_file_selected.call(fc.clone()),
                                            "📄 {file_name}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if files.is_empty() && !discovering {
                p {
                    style: "color: #8087a2; font-size: 12px; padding: 8px 0;",
                    "No .http files found in this directory."
                }
            }
        }
    }
}
