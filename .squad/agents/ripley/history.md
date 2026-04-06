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
- **pest migration scoping (2025-07-22):** The handwritten parser in `file_parser.rs` (~470 lines, state-machine design) is the only component that needs replacement. The public API (`parse_http_file`, `parse_http_content`) and all output types (`HttpRequest`, `Header`, `Assertion`, `Condition`, `Variable`) remain stable. Substitution runs post-parse and is decoupled. The serializer emits text, not AST, so it's unaffected. Consumers: CLI/GUI/TUI/WASM all go through the two public functions only. pest v2.8 is pure Rust and compiles to wasm32. The body-mode statefulness (blank line → body) cannot be expressed in PEG and requires a Rust post-pass. condition_parser.rs should survive as a thin helper; timeout_parser can be absorbed into grammar. 55 parser tests + 18 condition + 10 timeout + serializer round-trip tests form the acceptance gate. Decision written to `.squad/decisions/inbox/ripley-pest-parser-migration.md`.
- 📌 **Team update (2026-04-06T10:40:50Z):** Parser contract baseline work completed by Bishop; pest migration approved by squad with consolidated safety gates. Decisions merged and inbox cleared. Ready for Phase 1 grammar authoring. All 48 parser tests locked, serializer round-trip stable. — decided by Ripley, Bishop, Lambert, Hicks
