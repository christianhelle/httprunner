# Squad Decisions

## Active Decisions

### 2026-03-20: Review-oriented squad composition
**By:** Squad  
**What:** Use a six-member squad for this repository: Ripley (lead), Bishop (core and CLI), Vasquez (GUI/TUI/WASM), Ash (security), Lambert (testing and performance), and Hicks (platform and release).  
**Why:** The repository combines a shared execution engine with multiple user interfaces and release surfaces, so review and issue triage need clear ownership across architecture, engine behavior, UI, security, testing/performance, and delivery.

### 2026-03-20: Prioritized architecture review produced 4 GitHub issues (Ripley)
**By:** Ripley (Lead/Architect)  
**Date:** 2025-07-21  
**What:** Complete architecture review identified per-request HTTP client creation, regex compilation overhead, auth header leakage in verbose mode, and unmanaged GUI/TUI threading.  
**Issues:**
- #213 (P1): HTTP client rebuilt per-request — no connection pooling (Bishop)
- #215 (P2): Regex compiled on every call — 17+ compilations per request (Bishop)
- #217 (P2): Verbose/log mode leaks auth headers in cleartext (Ash)
- #219 (P2): GUI/TUI spawn unmanaged threads — no panic/cancel handling (Vasquez)

**Non-Issues Investigated:**
- `executor_async.rs` naming bug: gated behind WASM-only cfg, not affecting native builds
- Telemetry feature: compiled in but inert in GUI/TUI
- GUI/TUI state duplication: small divergence, not urgent

**Routing:** Ripley reviews all PRs. Bishop (#213, #215), Ash (#217), Vasquez (#219).

### 2026-03-20: Sensitive header redaction must be default in exports and logs (Ash)
**By:** Ash (Security)  
**Severity:** P1 — exploitable credential disclosure  
**What:** Exports, logging, and report formats write unredacted Authorization, Cookie, and API-key header values to disk. Variable substitution resolves secrets before output, so all surfaces inherit exposure.  
**Decision:** Export and logging layer must redact a well-known list of sensitive headers by default. Opt-in flag (`--include-secrets` or similar) available for debugging. Applies to Bishop's domain (core engine).  
**Issues Filed:**
- #214 (P1): Exports/logs write sensitive headers without redaction (Bishop)
- #216 (P1): Resolved variable values appear in all output surfaces (Bishop)
- #218 (P2): No warning when `--insecure` disables TLS validation (Bishop)
- #220 (P2): Install scripts lack download integrity verification (Hicks)
- #221 (P2): TUI response bodies not sanitized for ANSI escape codes (Vasquez)

**Positive Findings (No Action):**
- HTML reports have comprehensive XSS protection with test coverage
- Telemetry sanitizes error messages, collects no PII, respects opt-out
- No production unsafe code; shell/PowerShell install scripts structurally sound
- GUI/TUI correctly hardcode `insecure=false`

### 2026-03-20: Platform automation and installation review identified 4 issues (Hicks)
**By:** Hicks (Platform & Release)  
**Date:** 2026-03-20  
**What:** Audit of build, packaging, install, and release paths found fragile JSON parsing, unaudited direct-to-main commits, weak TUI Docker verification, and missing MSRV documentation.  
**Issues:**
- #222 (P2): `install.sh` uses fragile grep/regex JSON parsing instead of jq
- #223 (P2): `release.yml` commits snapcraft.yaml directly to main without PR approval
- #224 (P2): TUI Docker variant lacks integration verification in release workflow
- #225 (P3): Rust 1.92+ edition 2024 not documented as MSRV

**Priority Reasoning:**
- P2 issues are process risks and automation reliability gaps
- P3 is a documentation/contributor friction item
- Recommend addressing P2 issues before next release to improve automation reliability and auditability

**Recommendation:** Schedule build/release process review with Bishop and Ripley.

### 2026-03-20: Five highest-value testing and performance issues identified (Lambert)
**By:** Lambert (Testing & Performance)  
**Date:** 2026-03-20  
**Recommended Assignee:** Christian Helle  
**What:** Review of src/core, src/cli, src/gui, src/tui, examples, and README.md identified five highest-value issue drafts.  
**Issues Drafted:**

1. **P1 — Serializer/editor round-trip silently strips request semantics**
   - Scope: src/core/src/serializer.rs, src/core/src/parser/file_parser.rs, src/gui/src/request_editor.rs, src/gui/src/request_view.rs
   - Why critical: The save path is not semantically lossless. Serializer emits `# @assert ...` but parser does not parse it. Serializer also skips `@if-not`, `@pre-delay`, `@post-delay`. GUI rebuilds edited requests with empty assertions, conditions, and None delays. Editing and saving can silently change how a test file executes.
   - Squad owner: `squad:bishop`

2. **P1 — Browser GUI single-request execution does not match native/CLI semantics**
   - Scope: src/gui/src/results_view_async.rs, src/gui/src/results_view.rs, src/core/src/parser/mod.rs
   - Why critical: Native GUI goes through `process_http_file_incremental(...)` preserving context; WASM path executes only selected request directly, skipping dependency checks, request-variable context, and incremental model. Browser GUI behavior can diverge from CLI/native for chained requests, `@dependsOn`, request variables, and environment-dependent substitutions. No GUI/WASM regression tests.
   - Squad owner: `squad:vasquez`

3. **P2 — GUI/TUI file discovery does avoidable O(n log n) work and repeated cloning**
   - Scope: src/gui/src/file_tree.rs, src/tui/src/file_tree.rs, src/tui/src/ui.rs
   - Why important: Both UI implementations sort the full shared file list after every discovered file while holding a mutex. Render paths take full snapshots again (guard.clone() in GUI, files() in TUI), so large trees repeatedly allocate and reorder. User-visible performance smell with no benchmark or regression coverage.
   - Squad owner: `squad:vasquez`

4. **P2 — GUI state persistence serializes full execution history on routine UI actions**
   - Scope: src/gui/src/app.rs, src/gui/src/state.rs, src/gui/src/results_view.rs
   - Why important: save_state() called on many routine actions (font changes, environment changes, file selection, view toggles). Stores full result list clone and pretty-serializes entire state to JSON each time. Large responses turn harmless preference save into repeated cloning and disk/local-storage writes, exactly when users are debugging large payloads. Path is untested in GUI crate.
   - Squad owner: `squad:vasquez`

5. **P2 — Core execution pipeline duplicates large request/response payloads**
   - Scope: src/core/src/processor/incremental.rs, src/core/src/runner/executor.rs, src/core/src/runner/executor_async.rs, src/core/src/types/request.rs, src/core/src/types/result.rs
   - Why important: Incremental execution clones HttpRequest on skipped/failed branches and clones both request and result on success. Sync and async executors clone response headers and bodies to build temporary assertion result. HttpRequest and HttpResult own strings, vectors, headers, and bodies, so each clone scales with payload size. Affects CLI, GUI, and TUI execution. Current test suite is correctness-focused rather than allocation-aware.
   - Squad owner: `squad:bishop`

**Team Guidance for Future Reviews:**
- Explicitly compare parser output, serializer output, and GUI editor save behavior as one round-trip contract
- Treat native and WASM execution paths as parity-sensitive features; review together, not independently

## Governance

- All meaningful changes require team consensus
- Document architectural decisions here
- Keep history focused on work, decisions focused on direction
