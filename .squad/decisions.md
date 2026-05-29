# Squad Decisions

## Active Decisions

### 2026-05-29: Fail-fast mode contract (core + CLI)
**By:** Bishop (Core & CLI Engineer)  
**Date:** 2026-05-29  
**Scope:** `src/core/src/processor/executor.rs`, `src/cli/src/cli/args.rs`, `src/cli/src/main.rs`, `src/core/src/telemetry/tracking.rs`

**What:** A new opt-in fail-fast mode, exposed as the long-only CLI flag `--fail-fast` and as `ProcessorConfig::with_fail_fast(bool)` in the core engine. When enabled, execution halts at the first failed request and that request's full verbose detail is shown.

**Contract (so Vasquez and Lambert can align):**

1. **Failure trigger** = any `!result.success` outcome: a network/execution error, a non-2xx HTTP status, OR a failed assertion; PLUS parse errors / internal request-processing errors. **Skips never trigger fail-fast** (unmet `@if` condition or unmet `@dependsOn` dependency).

2. **Halt scope (CLI)** = stop the ENTIRE run: abandon the remaining requests in the current file AND skip all remaining files. Partial results gathered so far are retained, and `ProcessorResults.success` is `false`.

3. **Force capture** = when fail-fast is on, full response capture is forced for ALL requests so the failing request always has body/headers. Implemented by passing `config.verbose || config.fail_fast` as the executor closure's `verbose` argument (that arg drives `should_capture_response`). This is capture-only; verbose detail is NOT printed for successful requests.

4. **Verbose-on-failure** = on the failing request ONLY, emit the full verbose block (Request Details, Response Details, Assertion Results), honoring `--pretty-json` and secret redaction (unless `--include-secrets`). Earlier successful requests stay compact. If `--verbose` is already set, no extra dump is emitted (the normal flow already prints it). The in-processor "File Summary" / "Overall Summary" blocks are suppressed on a fail-fast halt.

5. **Post-halt (CLI)** = treated as a normal failed run by `main.rs`: the ❌ "Some discovered files failed to process" line still prints, `--report`/`--export`/`--export-json` are still generated from the partial results, the support key + donation banner still show, and the process exits with code 1. Only the per-request loop behavior changes.

**Implementation notes for parallel work:**
- Public core signatures stayed stable. `process_http_files_with_executor`, `process_http_files_with_config`, and `process_http_files` default `fail_fast = false`, so existing behavior (and existing tests) is unchanged.
- `process_http_files_with_options` gained a trailing `fail_fast: bool` parameter (its only caller is the CLI).
- The internal `process_single_file` now returns `Result<(HttpFileResults, bool)>` where the bool signals a fail-fast halt up to `process_http_files_with_config`.
- **GUI/TUI are untouched.** `process_http_file_incremental*` signatures are intentionally NOT changed. If Vasquez wants fail-fast in the incremental/UI path, that is a separate follow-up and should reuse this same trigger/halt/force-capture contract.

**Why:** Fail-fast gives CI/CD and debugging users an immediate, comprehensive view of the first failure without running the whole suite, while keeping the engine's normal execution path observable and unchanged when the flag is off.

### 2026-05-29: Fail-fast toggle for GUI (native + WASM) and TUI (Vasquez)
**By:** Vasquez (GUI / TUI / WASM Engineer)  
**Date:** 2026-05-29  
**Status:** Implemented (native build + gui/tui tests green)

**What:** Added a user-toggleable "fail-fast" mode to all three UI execution paths. When enabled, a "Run all" stops immediately at the first FAILED request and the results view auto-switches to verbose so the failing request's comprehensive details are shown.

**Behavior contract:**
- **Failure trigger:** `RequestProcessingResult::Executed { result.success == false }` (non-2xx or failed assertion) OR `RequestProcessingResult::Failed { .. }`. `Skipped { .. }` NEVER triggers fail-fast.
- **Toggle:** in-memory only, default OFF, NOT persisted to disk/localStorage.
  - GUI: a "Fail-fast" checkbox in the results controls, next to the Compact/Verbose (Ctrl+D) toggle. Stored on `ResultsView` (keeps it in sync with what the user sees; the run methods live on `ResultsView`).
  - TUI: the `F` key (gated so it does not fire while editing an environment), with a `Fail-fast: ON/OFF` indicator in the status bar and an `F` hint in the help line. Stored on the `App` struct (alongside `delay_ms`).
- **On first failure (fail-fast ON):**
  - The incremental callback returns `false` (should_continue=false) on the first failing result.
  - Force-capture: every run path passes a custom executor that forwards `verbose = fail_fast` so the failing request captures the full response body even when it is unnamed / has no assertions. Native uses `process_http_file_incremental_with_executor` with `runner::execute_http_request(req, fail_fast, insecure)`; WASM uses `process_http_requests_incremental_async` with `execute_http_request_async(req, fail_fast, insecure)`.
  - Auto-verbose: run threads cannot mutate `compact_mode` directly, so they set a shared `Arc<AtomicBool>`. The GUI consumes it at the top of `ResultsView::show()`; the TUI consumes it via `App::results_view.apply_pending_verbose_switch()` in the main loop before each draw.

**Design notes / decisions:**
- A tiny pure helper `should_continue_after(result, fail_fast) -> bool` (plus an async variant) encodes the stop decision and is unit-tested in both the gui and tui crates. Failure detection lives in one place per crate.
- The `fail_fast` flag is captured **by value** before each run thread / `spawn_local` closure, mirroring how `delay_ms` is threaded.
- **No core/cli changes.** The existing core incremental APIs already support early-stop (bool callback) and force-capture (executor `verbose` arg); their signatures were left untouched (Bishop's domain).
- Stored the toggle on the runtime UI structs rather than the serializable `AppState`, because the agreed contract is "in-memory only / do not persist" — adding a `#[serde(skip)]` field to the persisted state struct would have been dead state.

**Build/verify:**
- `cargo build -p httprunner-gui -p httprunner-tui` — clean.
- `cargo test -p httprunner-gui -p httprunner-tui` — all green (incl. new fail-fast helper and auto-verbose tests).
- `cargo check -p httprunner-gui --target wasm32-unknown-unknown` — currently blocked by a PRE-EXISTING compile error in `src/core/src/runner/executor_async.rs` (`Result` missing its error type parameter), confirmed present without my changes. WASM-side GUI code follows the same parity-checked pattern; needs a re-check once core compiles for wasm again (Bishop).

**Parity:** desktop, terminal, and browser behavior are intentionally identical. No "desktop-only" assumptions were introduced.

### 2026-05-29: User directive on commit hygiene
**By:** Christian Helle (via Copilot)  
**What:** Commit changes in small logical groups without a co-author, and make that automatic in future by adding it to the Copilot/agent instructions or configuration.  
**Why:** User request — captured for team memory.

### 2026-05-29: Commit hygiene automation location (Hicks)
**By:** Hicks (Platform & Release)  
**What:** Encode commit hygiene rules in three locations for consistency:
- `.github/copilot-instructions.md` — governs runtime Copilot behavior for this repo
- `.squad/copilot-instructions.md` — mirrors the same rule set; Squad sessions load this as an input artifact
- `.squad/templates/copilot-instructions.md` — carries the same commit hygiene rules so future refreshes do not drift back to auto co-authoring or monolithic commits

**Why:** Ensures that the user's directive (small logical commits, no co-author trailers) is automatically enforced across Copilot runtime sessions and survives template refreshes.


## Governance

- All meaningful changes require team consensus
- Document architectural decisions here
- Keep history focused on work, decisions focused on direction

