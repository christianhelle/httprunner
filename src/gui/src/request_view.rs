use crate::request_editor::RequestEditor;
use std::path::Path;

pub enum RequestViewAction {
    RunRequest(usize),
    SaveFile,
    None,
}

pub struct RequestView {
    editor: RequestEditor,
}

impl RequestView {
    pub fn new() -> Self {
        Self {
            editor: RequestEditor::new(),
        }
    }

    pub fn load_file(&mut self, path: &Path) {
        self.editor.load_file(path);
    }

    pub fn get_requests(&self) -> &[httprunner_lib::HttpRequest] {
        self.editor.get_requests()
    }

    pub fn has_changes(&self) -> bool {
        self.editor.has_changes()
    }

    pub fn save_to_file(&mut self) -> anyhow::Result<()> {
        self.editor.save_to_file()
    }
}
