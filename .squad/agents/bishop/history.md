# Project Context

- **Owner:** Christian Helle
- **Project:** httprunner
- **Stack:** Rust 2024 workspace, reqwest, clap, tokio, egui/eframe, ratatui/crossterm, wasm32, GitHub Actions
- **Created:** 2026-03-20T17:28:26.715Z

## Learnings

- Bishop owns `src/core` and `src/cli`, including parser, runner, processor, and CLI behavior.
- The repo's core responsibilities include `.http` parsing, request chaining, assertions, variables, and reporting.
- Initial review scope includes possible defects plus security and performance findings that touch execution flow.
