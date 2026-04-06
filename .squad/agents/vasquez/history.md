# Project Context

- **Owner:** Christian Helle
- **Project:** httprunner
- **Stack:** Rust 2024 workspace, reqwest, clap, tokio, egui/eframe, ratatui/crossterm, wasm32, GitHub Actions
- **Created:** 2026-03-20T17:28:26.715Z

## Learnings

- Vasquez owns `src/gui` and `src/tui`, with special attention to WASM behavior and desktop/web differences.
- The GUI supports native and browser execution, while the TUI emphasizes keyboard-driven workflows and persisted state.
- Review work should watch for async UI bugs, stale state, and parity issues between surfaces.
- In this repo, the highest-signal UI audits come from comparing README promises and shortcut hints against the actual action gates in `app.rs`; WASM and keyboard paths drift first.
- GUI request editing must preserve the full `HttpRequest` shape, including assertions, variables, conditions, and delay fields, or the UI silently corrupts files on save.
- 📌 **Team update (2026-04-06T10:40:50Z):** Pest migration approved by squad; Vasquez responsible for WASM parity verification post-migration. Parser contract baseline locked. GUI/TUI consumers use stable `parse_http_file`/`parse_http_content` API. Decisions merged and inbox cleared. Ready for Phase 1. — decided by Ripley, Bishop, Lambert, Hicks
