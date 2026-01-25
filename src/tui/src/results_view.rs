use crossterm::event::{KeyCode, KeyEvent};
use httprunner_lib::report::DiscoveryResults;

pub struct ResultsView {
    results: Option<DiscoveryResults>,
    scroll_offset: usize,
}

impl ResultsView {
    pub fn new() -> Self {
        Self {
            results: None,
            scroll_offset: 0,
        }
    }

    pub fn set_results(&mut self, results: DiscoveryResults) {
        self.results = Some(results);
        self.scroll_offset = 0;
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

    pub fn results(&self) -> Option<&DiscoveryResults> {
        self.results.as_ref()
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn passed_count(&self) -> usize {
        self.results
            .as_ref()
            .map(|r| r.total_passed)
            .unwrap_or(0)
    }

    pub fn failed_count(&self) -> usize {
        self.results
            .as_ref()
            .map(|r| r.total_failed)
            .unwrap_or(0)
    }
}
