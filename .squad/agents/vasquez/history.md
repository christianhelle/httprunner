# Project Context

- **Owner:** Christian Helle
- **Project:** httprunner
- **Stack:** Rust 2024 workspace, reqwest, clap, tokio, egui/eframe, ratatui/crossterm, wasm32, GitHub Actions
- **Created:** 2026-03-20T17:28:26.715Z

## Learnings

- Vasquez owns `src/gui` and `src/tui`, with special attention to WASM behavior and desktop/web differences.
- The GUI supports native and browser execution, while the TUI emphasizes keyboard-driven workflows and persisted state.
- Review work should watch for async UI bugs, stale state, and parity issues between surfaces.
