# Project Context

- **Owner:** Christian Helle
- **Project:** httprunner
- **Stack:** Rust 2024 workspace, reqwest, clap, tokio, egui/eframe, ratatui/crossterm, wasm32, GitHub Actions
- **Created:** 2026-03-20T17:28:26.715Z

## Learnings

- Bishop owns `src/core` and `src/cli`, including parser, runner, processor, and CLI behavior.
- The repo's core responsibilities include `.http` parsing, request chaining, assertions, variables, and reporting.
- Initial review scope includes possible defects plus security and performance findings that touch execution flow.
- **2026-03-20 Core/CLI audit completed.** Filed GitHub issues #231-#235 for exit codes, JSON extraction correctness, condition parsing, parser strictness, and fail-open request-variable substitution.
- CLI execution failures do not currently propagate to the process exit code because `run()` ignores `ProcessorResults.success` and only exits non-zero on hard `Err` returns.
- Request-variable extraction and `@if` JSON body conditions both depend on `src/core/src/variables/json.rs`, whose string-scanning approach can select nested duplicate keys and truncate object values when braces appear inside JSON strings.
- Condition parsing treats `==` as literal expected text, so `@if login.response.status == 200` silently becomes a never-matching comparison.
- Parser directives and request-variable substitution currently favor warnings/pass-through over hard failures, which makes malformed `@timeout` directives and unresolved `{{request.response...}}` placeholders execute as if they were valid inputs.
