# Project Context

- **Owner:** Christian Helle
- **Project:** httprunner
- **Stack:** Rust 2024 workspace, reqwest, clap, tokio, egui/eframe, ratatui/crossterm, wasm32, GitHub Actions
- **Created:** 2026-03-20T17:28:26.715Z

## Learnings

- Lambert owns testing and performance review across the workspace.
- Likely hot paths include file discovery, parsing, request execution, result rendering, and report generation.
- Current user goal is to identify issues worth turning into prioritized GitHub backlog items.
- The current serializer/editor save path does not preserve advanced request semantics: assertions, negated conditions, and per-request delays are not round-tripped safely.
- WASM single-request execution currently diverges from native and CLI behavior because it skips the incremental processor semantics used to preserve request context.
- Large-workspace hot paths include GUI/TUI file discovery and GUI state persistence, both of which currently clone or sort whole collections more often than necessary.
