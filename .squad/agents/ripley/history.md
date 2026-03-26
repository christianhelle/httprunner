# Project Context

- **Owner:** Christian Helle
- **Project:** httprunner
- **Stack:** Rust 2024 workspace, reqwest, clap, tokio, egui/eframe, ratatui/crossterm, wasm32, GitHub Actions
- **Created:** 2026-03-20T17:28:26.715Z

## Learnings

- Initial squad setup assigned Ripley to lead architecture, review routing, and issue triage.
- The repository spans shared core logic plus CLI, GUI, TUI, and release surfaces.
- Christian Helle wants prioritized findings turned into GitHub issues assigned to him.
- `executor_async.rs` is only compiled for `wasm32` (gated in `runner/mod.rs:4`). The `_request` vs `request` variable name mismatch on line 77 is a real bug but only affects WASM builds.
- `reqwest::blocking::Client` is rebuilt per-request in `executor.rs:85-99` — no client reuse or connection pooling. This is the single biggest performance bottleneck (#213).
- All regex patterns (37+ `Regex::new()` calls, 17 per `substitute_functions()` invocation) are compiled at call time, never cached (#215).
- Verbose/log mode in `processor/executor.rs` logs all headers and bodies in cleartext — no redaction of auth headers (#217).
- GUI and TUI duplicate the same raw `thread::spawn` pattern with no panic handling or cancellation (#219). Core lacks an execution lifecycle abstraction.
- Available GitHub labels: bug, enhancement, Core, CLI, GUI, TUI, WASM, documentation, etc. No custom priority or performance labels exist.
- Feature flags: core defaults to `telemetry = ["appinsights", "tokio"]`. CLI explicitly enables it. GUI/TUI get it via default features but never call telemetry init — telemetry is compiled in but inert for GUI/TUI.
