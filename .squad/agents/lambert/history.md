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
- Parser migration to pest.rs carries moderate to high risk due to stateful body mode logic, known serializer/parser round-trip failure (issue #213), and unprotected request variable/function edge cases. 24 new tests recommended before migration, plus performance baselines.
- Current parser has 129 tests covering directives, substitution, and error cases but lacks protection for: body-mode state transitions, request variable extraction, built-in function substitution in URLs/headers, malformed input handling, and cross-platform line endings.
- Pest migration risk assessment identified that PEG grammar is documentation-only and does not express stateful parser logic (directive buffering, body mode switching, IntelliJ script block skipping). Semantic actions will be required.
- Parser hot path performance must be measured before pest migration: no benchmark exists for throughput (files/sec, MB/sec) or allocation patterns on large files (100+ requests) or large bodies (10MB+).
- 📌 **Team update (2026-04-06T10:40:50Z):** Parser contract baseline work completed; pest migration approved with pre-migration safety gates. Lambert's 24 new tests, performance baselines, and WASM verification are part of the consolidated safety plan. Decisions merged and inbox cleared. Ready for Phase 1. — decided by Ripley, Bishop, Lambert, Hicks
- 📌 **Backend swap complete (2026-04-06T11:28:31Z):** Phase 3 complete. Parser public API now uses pest backend. All 927 tests pass (parser, serializer, integration). Handwritten parser available for parity validation. Next phase: performance validation, legacy code cleanup, and decision on test architecture for long-term maintainability. — Orchestration log at `.squad/orchestration-log/2026-04-06T11-28-31Z-bishop.md`
- IntelliJ script-block parity depends on matching the legacy rule literally: any trimmed line ending with `%}` must close script mode, even when the line has script content before the marker.
- An ignored release benchmark that compares `parse_http_content_with_legacy_backend(...)` against the pest backend on the examples corpus, a 1000-request synthetic file, and a 10MB body fixture is a practical way to keep parity checks and throughput evidence together. Current measurements show the pest backend is still materially slower on representative inputs (about 94%, 99.9%, and 21% throughput regressions respectively), so performance sign-off is not yet earned.
- 📌 **Functions example regression fixed (2026-04-06T14:14:53Z):** Request-derived helper variables in `examples/functions.http` moved ahead of request chain because parser body mode treats `@...` lines as body text once a request body opens. Parser regression test in `src/core/src/parser/tests.rs` locks the example parse shape. Incremental processor regression test in `src/core/src/processor/incremental_tests.rs` validates end-to-end execution with `MockHttpExecutor`. Product commit `849ecf0`. — Orchestration log at `.squad/orchestration-log/2026-04-06T14-14-53Z-lambert.md`
