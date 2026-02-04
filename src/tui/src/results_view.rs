use crossterm::event::{KeyCode, KeyEvent};
use httprunner_lib::types::{AssertionResult, ProcessorResults};
use std::sync::{Arc, Mutex};

/// Individual execution result for incremental display
#[derive(Clone)]
pub enum ExecutionResult {
    Success {
        method: String,
        url: String,
        status: u16,
        duration_ms: u64,
        request_body: Option<String>,
        response_body: String,
        assertion_results: Vec<AssertionResult>,
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
    #[allow(dead_code)]
    results: Option<ProcessorResults>,
    /// Incremental results for async execution
    incremental_results: Arc<Mutex<Vec<ExecutionResult>>>,
    /// Whether execution is in progress
    is_running: Arc<Mutex<bool>>,
    scroll_offset: usize,
    /// Compact mode (true) or Verbose mode (false)
    compact_mode: bool,
}

impl ResultsView {
    pub fn new() -> Self {
        Self {
            results: None,
            incremental_results: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(Mutex::new(false)),
            scroll_offset: 0,
            compact_mode: true,
        }
    }

    pub fn toggle_compact_mode(&mut self) {
        self.compact_mode = !self.compact_mode;
    }

    pub fn set_compact_mode(&mut self, compact: bool) {
        self.compact_mode = compact;
    }

    pub fn is_compact_mode(&self) -> bool {
        self.compact_mode
    }

    #[allow(dead_code)]
    pub fn set_results(&mut self, results: ProcessorResults) {
        self.results = Some(results);
        // Clear incremental results when setting batch results
        if let Ok(mut inc) = self.incremental_results.lock() {
            inc.clear();
        }
        self.scroll_offset = 0;
    }

    pub fn incremental_results(&self) -> Arc<Mutex<Vec<ExecutionResult>>> {
        Arc::clone(&self.incremental_results)
    }

    pub fn is_running_arc(&self) -> Arc<Mutex<bool>> {
        Arc::clone(&self.is_running)
    }

    pub fn is_running(&self) -> bool {
        self.is_running.lock().map(|g| *g).unwrap_or(false)
    }

    pub fn clear_for_async_run(&mut self) {
        self.results = None;
        if let Ok(mut inc) = self.incremental_results.lock() {
            inc.clear();
        }
        if let Ok(mut running) = self.is_running.lock() {
            *running = true;
        }
        self.scroll_offset = 0;
    }

    pub fn get_incremental_results(&self) -> Vec<ExecutionResult> {
        self.incremental_results
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll_offset += 1;
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.scroll_offset += 10;
            }
            KeyCode::Home => {
                self.scroll_offset = 0;
            }
            _ => {}
        }
    }

    pub fn results(&self) -> Option<&ProcessorResults> {
        self.results.as_ref()
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    #[allow(dead_code)]
    pub fn passed_count(&self) -> usize {
        // First check incremental results
        let inc_passed = self
            .incremental_results
            .lock()
            .map(|g| {
                g.iter()
                    .filter(|r| matches!(r, ExecutionResult::Success { .. }))
                    .count()
            })
            .unwrap_or(0);

        if inc_passed > 0 {
            return inc_passed;
        }

        self.results
            .as_ref()
            .map(|r| r.files.iter().map(|f| f.success_count as usize).sum())
            .unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn failed_count(&self) -> usize {
        // First check incremental results
        let inc_failed = self
            .incremental_results
            .lock()
            .map(|g| {
                g.iter()
                    .filter(|r| matches!(r, ExecutionResult::Failure { .. }))
                    .count()
            })
            .unwrap_or(0);

        if inc_failed > 0 || self.is_running() {
            return inc_failed;
        }

        self.results
            .as_ref()
            .map(|r| r.files.iter().map(|f| f.failed_count as usize).sum())
            .unwrap_or(0)
    }
}
