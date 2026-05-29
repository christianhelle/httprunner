use crossterm::event::{KeyCode, KeyEvent};
use httprunner_core::processor::RequestProcessingResult;
use httprunner_core::types::{AssertionResult, ProcessorResults};
use std::sync::atomic::{AtomicBool, Ordering};
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
    Skipped {
        method: String,
        url: String,
        reason: String,
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
    /// Set by a run thread when it halts on a failure under fail-fast, so the
    /// next render can force verbose mode.
    switch_to_verbose: Arc<AtomicBool>,
}

impl ResultsView {
    pub fn new() -> Self {
        Self {
            results: None,
            incremental_results: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(Mutex::new(false)),
            scroll_offset: 0,
            compact_mode: true,
            switch_to_verbose: Arc::new(AtomicBool::new(false)),
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

    /// Shared flag used by the run thread to request a verbose switch when a run
    /// halts on a failure under fail-fast.
    pub fn switch_to_verbose_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.switch_to_verbose)
    }

    /// If a run requested a verbose switch (fail-fast halt), apply it. Should be
    /// called each frame before rendering.
    pub fn apply_pending_verbose_switch(&mut self) {
        if self.switch_to_verbose.swap(false, Ordering::SeqCst) {
            self.compact_mode = false;
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running.lock().map(|g| *g).unwrap_or(false)
    }

    pub fn try_clear_for_async_run(&mut self) -> bool {
        if let Ok(mut running) = self.is_running.lock() {
            if *running {
                return false;
            }
            *running = true;
        } else {
            return false;
        }

        self.results = None;
        if let Ok(mut inc) = self.incremental_results.lock() {
            inc.clear();
        }
        self.scroll_offset = 0;

        true
    }

    pub fn get_incremental_results(&self) -> Vec<ExecutionResult> {
        self.incremental_results
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if self.scroll_offset > 0 => {
                self.scroll_offset -= 1;
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

/// Decide whether a run should continue after processing `result`.
///
/// With fail-fast enabled, the run stops on the first failing request: an
/// executed request whose result was not successful, or a processing failure.
/// Skipped requests never trigger fail-fast.
pub fn should_continue_after(result: &RequestProcessingResult, fail_fast: bool) -> bool {
    !(fail_fast && request_result_is_failure(result))
}

fn request_result_is_failure(result: &RequestProcessingResult) -> bool {
    match result {
        RequestProcessingResult::Executed { result, .. } => !result.success,
        RequestProcessingResult::Failed { .. } => true,
        RequestProcessingResult::Skipped { .. } => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httprunner_core::types::{HttpRequest, HttpResult};

    fn sample_request() -> HttpRequest {
        HttpRequest {
            name: None,
            method: "GET".to_string(),
            url: "https://example.com".to_string(),
            headers: vec![],
            body: None,
            assertions: vec![],
            variables: vec![],
            timeout: None,
            connection_timeout: None,
            depends_on: None,
            conditions: vec![],
            pre_delay_ms: None,
            post_delay_ms: None,
        }
    }

    fn sample_result(success: bool) -> HttpResult {
        HttpResult {
            request_name: None,
            status_code: if success { 200 } else { 500 },
            success,
            error_message: None,
            duration_ms: 1,
            response_headers: None,
            response_body: None,
            assertion_results: vec![],
        }
    }

    #[test]
    fn try_clear_for_async_run_prevents_overlapping_execution() {
        let mut results_view = ResultsView::new();

        assert!(results_view.try_clear_for_async_run());
        assert!(!results_view.try_clear_for_async_run());
        assert!(results_view.is_running());
    }

    #[test]
    fn should_continue_after_stops_on_failures_only_when_fail_fast() {
        let failed_exec = RequestProcessingResult::Executed {
            request: sample_request(),
            result: sample_result(false),
        };
        let processing_failed = RequestProcessingResult::Failed {
            request: sample_request(),
            error: "boom".to_string(),
        };
        let ok = RequestProcessingResult::Executed {
            request: sample_request(),
            result: sample_result(true),
        };
        let skipped = RequestProcessingResult::Skipped {
            request: sample_request(),
            reason: "dependency".to_string(),
        };

        // Fail-fast enabled
        assert!(!should_continue_after(&failed_exec, true));
        assert!(!should_continue_after(&processing_failed, true));
        assert!(should_continue_after(&ok, true));
        assert!(should_continue_after(&skipped, true)); // skips never stop

        // Fail-fast disabled never stops
        assert!(should_continue_after(&failed_exec, false));
        assert!(should_continue_after(&processing_failed, false));
        assert!(should_continue_after(&ok, false));
        assert!(should_continue_after(&skipped, false));
    }

    #[test]
    fn apply_pending_verbose_switch_forces_verbose_once() {
        let mut results_view = ResultsView::new();
        results_view.set_compact_mode(true);

        results_view
            .switch_to_verbose_flag()
            .store(true, Ordering::SeqCst);
        results_view.apply_pending_verbose_switch();
        assert!(!results_view.is_compact_mode());

        // Flag consumed: switching back to compact stays compact.
        results_view.set_compact_mode(true);
        results_view.apply_pending_verbose_switch();
        assert!(results_view.is_compact_mode());
    }

    #[test]
    fn skipped_results_are_not_counted_as_passed_or_failed() {
        let mut results_view = ResultsView::new();
        let arc = results_view.incremental_results();
        let mut results = arc.lock().unwrap();
        results.push(ExecutionResult::Success {
            method: "GET".to_string(),
            url: "https://example.com".to_string(),
            status: 200,
            duration_ms: 10,
            request_body: None,
            response_body: "ok".to_string(),
            assertion_results: vec![],
        });
        results.push(ExecutionResult::Skipped {
            method: "POST".to_string(),
            url: "https://example.com/skip".to_string(),
            reason: "dependency not met".to_string(),
        });
        results.push(ExecutionResult::Failure {
            method: "DELETE".to_string(),
            url: "https://example.com/fail".to_string(),
            error: "404 Not Found".to_string(),
        });
        drop(results);

        assert_eq!(results_view.passed_count(), 1);
        assert_eq!(results_view.failed_count(), 1);
    }

    #[test]
    fn get_incremental_results_returns_clone() {
        let mut results_view = ResultsView::new();
        assert!(results_view.get_incremental_results().is_empty());

        results_view
            .incremental_results()
            .lock()
            .unwrap()
            .push(ExecutionResult::Skipped {
                method: "GET".to_string(),
                url: "https://example.com".to_string(),
                reason: "condition false".to_string(),
            });

        let clone = results_view.get_incremental_results();
        assert_eq!(clone.len(), 1);
        assert!(matches!(clone[0], ExecutionResult::Skipped { .. }));
    }

    #[test]
    fn skipped_variant_stores_bare_method_without_icon_prefix() {
        // Ensure ExecutionResult::Skipped holds bare method (no embedded icon),
        // because the renderer in ui.rs prepends its own icon.
        let result = ExecutionResult::Skipped {
            method: "GET".to_string(),
            url: "https://example.com".to_string(),
            reason: "condition".to_string(),
        };
        if let ExecutionResult::Skipped { method, .. } = result {
            assert!(!method.contains('⏭'), "method should not embed the skip icon");
        }
    }
}
