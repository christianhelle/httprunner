# Project Context

- **Owner:** Christian Helle
- **Project:** httprunner
- **Stack:** Rust 2024 workspace, reqwest, clap, tokio, egui/eframe, ratatui/crossterm, wasm32, GitHub Actions
- **Created:** 2026-03-20T17:28:26.715Z

## Learnings

- Ash owns security and privacy review across CLI, GUI, TUI, reports, exports, telemetry, and TLS handling.
- The project supports `--insecure`, log/export/report generation, environment files, and request chaining, all of which are likely review surfaces.
- Findings should be framed in terms of exposure path, affected surfaces, and practical mitigation.
- **2026-07-24 Security Review completed.** Reviewed all src/core, src/cli, src/gui, src/tui, install scripts, and Cargo.toml. Filed 5 GitHub issues (#214, #216, #218, #220, #221).
- HTML report generation is well-protected against XSS (custom `escape_html` in `html.rs` with test coverage). No action needed there.
- Telemetry is clean: sanitizes errors, collects no PII, respects `DO_NOT_TRACK` and `--no-telemetry`. No action needed.
- The primary leak surface is the export/log pipeline: `exporter.rs`, `json_exporter.rs`, and `executor.rs` verbose logging all write unredacted sensitive headers to disk.
- Variable substitution resolves secrets before any output path sees the data, meaning every report/export format inherits the exposure.
- Install scripts are structurally sound (quoted variables, HTTPS URLs) but lack SHA-256 checksum verification of downloaded binaries.
- GUI and TUI hardcode `insecure=false`, which is correct. Only CLI exposes the `--insecure` flag.
- `strip_ansi_codes` exists in the logging module but is not applied to TUI response body rendering.
- `unsafe` blocks are test-only (env var setup). No production unsafe code.
