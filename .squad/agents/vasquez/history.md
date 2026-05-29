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

### 2026-05-29: Fail-fast toggle (GUI native + WASM, TUI)

- **Feature:** user-toggleable fail-fast. When ON, a run halts at the first FAILED request and the results view auto-switches to verbose so the failing request's full detail is shown. Default OFF, in-memory only (never persisted).
- **Files touched:**
  - `src/gui/src/results_view.rs` — added `fail_fast: bool` + `switch_to_verbose: Arc<AtomicBool>` to `ResultsView`; `is_fail_fast`/`set_fail_fast`/`switch_to_verbose_flag` accessors; "Fail-fast" checkbox next to the Compact/Verbose toggle in `show()`; `show()` consumes the flag and forces `compact_mode=false`. Native `run_file`/`run_single_request` now use `process_http_file_incremental_with_executor`. Added pure helpers `should_continue_after` (sync, cfg!=wasm) and `should_continue_after_async` + unit tests.
  - `src/gui/src/results_view_async.rs` — WASM `run_content_async`/`run_single_request_async` capture `fail_fast`+flag, use `make_async_executor(fail_fast)` (forwards `verbose=fail_fast`), call `should_continue_after_async`, set the shared flag on halt.
  - `src/tui/src/results_view.rs` — added `switch_to_verbose` flag + `switch_to_verbose_flag()`/`apply_pending_verbose_switch()`; pure `should_continue_after` helper + tests.
  - `src/tui/src/app.rs` — added `fail_fast: bool` to `App`; `f` key (gated to non-EnvironmentEditor pane) → `toggle_fail_fast`; `run_all_requests` uses the with-executor processor + helper + halt flag.
  - `src/tui/src/main.rs` — `apply_pending_verbose_switch()` each frame before draw.
  - `src/tui/src/ui.rs` — `Fail-fast: ON/OFF` status indicator + `F` help hint.
  - READMEs (`src/gui`, `src/tui`) documented the toggle.
- **How the toggle threads into each run path:** the bool is captured by value before the run thread / `spawn_local` closure (mirrors `delay_ms`). The closures own a `Copy` of `fail_fast`.
- **Force-capture-via-executor:** the native incremental path defaults its executor to `verbose=false`, so unnamed/no-assertion requests don't capture a body. To guarantee the failed request shows full detail, all run paths pass a custom executor that forwards `verbose = fail_fast` (`runner::execute_http_request(req, fail_fast, insecure)` native; `execute_http_request_async(req, fail_fast, insecure)` WASM). Core signatures were NOT changed (Bishop's domain).
- **Auto-verbose-on-halt:** run threads can't touch `compact_mode` directly, so they set a shared `Arc<AtomicBool>`; GUI consumes it at the top of `show()`, TUI consumes it via `apply_pending_verbose_switch()` in the main loop.
- **Decision helper for testability:** extracted `should_continue_after(result, fail_fast) -> bool` (and async variant) as pure functions. Failure = `Executed{success:false}` OR `Failed{..}`; `Skipped{..}` never stops. Unit-tested in both crates.
- **Gotcha:** `httprunner_core::processor` (and `RequestProcessingResult`) is `#[cfg(not(target_arch="wasm32"))]`, so the sync helper must be cfg-gated; `AsyncRequestProcessingResult` (in `runner`) is available on all targets.
- **WASM build caveat:** `cargo check -p httprunner-gui --target wasm32-unknown-unknown` currently fails inside `src/core/src/runner/executor_async.rs` (`Result` missing its error type param) — a pre-existing core error in Bishop's domain, confirmed present without my changes. Native build + `cargo test` for gui/tui are clean.

