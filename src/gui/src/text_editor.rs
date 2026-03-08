use std::path::{Path, PathBuf};

/// Text editor state for editing .http files.
pub struct TextEditor {
    pub(crate) content: String,
    pub(crate) current_file: Option<PathBuf>,
    pub(crate) has_changes: bool,
}

impl TextEditor {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            current_file: None,
            has_changes: false,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_file(&mut self, path: &Path) {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                self.content = content;
                self.current_file = Some(path.to_path_buf());
                self.has_changes = false;
            }
            Err(error) => {
                eprintln!("Failed to load file {}: {}", path.display(), error);
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load_file(&mut self, _path: &Path) {
        use web_sys::window;

        if let Some(window) = window()
            && let Ok(Some(storage)) = window.local_storage()
            && let Ok(Some(saved_content)) = storage.get_item("httprunner_editor_content")
        {
            self.content = saved_content;
            self.has_changes = false;
        }
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.has_changes = true;
    }

    pub fn replace_content(&mut self, content: String) {
        self.content = content;
        self.has_changes = false;
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_to_file(&mut self) -> anyhow::Result<()> {
        if let Some(path) = &self.current_file {
            std::fs::write(path, &self.content)?;
            self.has_changes = false;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No file loaded"))
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save_to_file(&mut self) -> anyhow::Result<()> {
        use web_sys::window;

        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                storage
                    .set_item("httprunner_editor_content", &self.content)
                    .map_err(|error| {
                        anyhow::anyhow!("Failed to save to LocalStorage: {:?}", error)
                    })?;
                self.has_changes = false;
                Ok(())
            } else {
                Err(anyhow::anyhow!("LocalStorage is not available"))
            }
        } else {
            Err(anyhow::anyhow!("Window object is not available"))
        }
    }

    pub fn has_changes(&self) -> bool {
        self.has_changes
    }

    pub fn line_numbers(&self) -> String {
        let line_count = self.content.matches('\n').count() + 1;
        (1..=line_count)
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn highlighted_html(&self) -> String {
        let mut in_block_comment = false;
        let mut lines = Vec::new();

        for line in self.content.split('\n') {
            let trimmed = line.trim_start();
            let toggles_block_comment = trimmed.starts_with("###");

            let highlighted = if toggles_block_comment {
                in_block_comment = !in_block_comment;
                wrap_class("syntax-comment", escape_html(line))
            } else if in_block_comment || trimmed.starts_with('#') {
                wrap_class("syntax-comment", escape_html(line))
            } else {
                highlight_http_line(line)
            };

            lines.push(highlighted);
        }

        lines.join("\n")
    }
}

const HTTP_METHODS: &[&str] = &[
    "GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS", "CONNECT", "TRACE",
];
const REQUEST_KEYWORDS: &[&str] = &["ASSERT", "VAR"];

fn highlight_http_line(line: &str) -> String {
    let indent_width = line.len() - line.trim_start().len();
    let (indent, body) = line.split_at(indent_width);
    let escaped_indent = escape_html(indent);
    let trimmed = body.trim_start();

    if trimmed.is_empty() {
        return escaped_indent;
    }

    if let Some(request_line) = highlight_request_line(trimmed) {
        return format!("{}{}", escaped_indent, request_line);
    }

    if let Some(keyword_line) = highlight_keyword_line(trimmed) {
        return format!("{}{}", escaped_indent, keyword_line);
    }

    if let Some(header_line) = highlight_header_line(trimmed) {
        return format!("{}{}", escaped_indent, header_line);
    }

    format!("{}{}", escaped_indent, highlight_inline(trimmed))
}

fn highlight_request_line(line: &str) -> Option<String> {
    let (method, remainder) = line.split_once(char::is_whitespace)?;
    if !HTTP_METHODS.contains(&method) {
        return None;
    }

    let url = remainder.trim_start();
    let method = wrap_class("syntax-method", escape_html(method));

    if url.is_empty() {
        Some(method)
    } else {
        Some(format!(
            "{} <span class=\"syntax-url\">{}</span>",
            method,
            highlight_inline(url)
        ))
    }
}

fn highlight_keyword_line(line: &str) -> Option<String> {
    let mut parts = line.splitn(2, char::is_whitespace);
    let keyword = parts.next()?;
    if !REQUEST_KEYWORDS.contains(&keyword) {
        return None;
    }

    let highlighted_keyword = wrap_class("syntax-keyword", escape_html(keyword));
    let remainder = parts.next().unwrap_or_default().trim_start();

    if remainder.is_empty() {
        Some(highlighted_keyword)
    } else {
        Some(format!(
            "{} {}",
            highlighted_keyword,
            highlight_inline(remainder)
        ))
    }
}

fn highlight_header_line(line: &str) -> Option<String> {
    let (name, value) = line.split_once(':')?;
    let trimmed_name = name.trim();
    if trimmed_name.is_empty() || trimmed_name.contains(char::is_whitespace) {
        return None;
    }

    let highlighted_name = wrap_class("syntax-header-name", escape_html(trimmed_name));
    let highlighted_value = highlight_inline(value.trim_start());
    Some(format!(
        "{}: <span class=\"syntax-header-value\">{}</span>",
        highlighted_name, highlighted_value
    ))
}

fn highlight_inline(value: &str) -> String {
    let mut output = String::new();
    let mut remaining = value;

    while let Some(start) = remaining.find("{{") {
        let (before, after_start) = remaining.split_at(start);
        output.push_str(&escape_html(before));

        if let Some(end) = after_start[2..].find("}}") {
            let variable = &after_start[..end + 4];
            output.push_str(&wrap_class("syntax-variable", escape_html(variable)));
            remaining = &after_start[end + 4..];
        } else {
            output.push_str(&escape_html(after_start));
            return output;
        }
    }

    output.push_str(&escape_html(remaining));
    output
}

fn wrap_class(class_name: &str, content: String) -> String {
    format!("<span class=\"{}\">{}</span>", class_name, content)
}

fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }

    escaped
}

#[cfg(test)]
mod tests {
    use super::TextEditor;

    #[test]
    fn line_numbers_always_start_at_one() {
        let editor = TextEditor::new();
        assert_eq!(editor.line_numbers(), "1");
    }

    #[test]
    fn highlights_http_methods_and_variables() {
        let mut editor = TextEditor::new();
        editor.set_content("GET https://example.com/{{tenant}}".to_string());

        let html = editor.highlighted_html();

        assert!(html.contains("syntax-method"));
        assert!(html.contains("syntax-url"));
        assert!(html.contains("syntax-variable"));
    }

    #[test]
    fn escapes_html_in_editor_content() {
        let mut editor = TextEditor::new();
        editor.set_content("<script>alert('xss')</script>".to_string());

        let html = editor.highlighted_html();

        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }
}
