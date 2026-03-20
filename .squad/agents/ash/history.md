# Project Context

- **Owner:** Christian Helle
- **Project:** httprunner
- **Stack:** Rust 2024 workspace, reqwest, clap, tokio, egui/eframe, ratatui/crossterm, wasm32, GitHub Actions
- **Created:** 2026-03-20T17:28:26.715Z

## Learnings

- Ash owns security and privacy review across CLI, GUI, TUI, reports, exports, telemetry, and TLS handling.
- The project supports `--insecure`, log/export/report generation, environment files, and request chaining, all of which are likely review surfaces.
- Findings should be framed in terms of exposure path, affected surfaces, and practical mitigation.
