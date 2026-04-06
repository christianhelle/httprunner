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

### 2026-03-20: Parser contract baseline frozen for pest migration (Bishop)
**By:** Bishop (Core & CLI Engineer)  
**Date:** 2026-03-20  
**What:** Before pest.rs migration, the parser's stateful behavior is locked in tests: blank-line body-mode transitions, raw `@...` lines inside bodies, directive buffering to next request, precedence between comments/IntelliJ script blocks/assertions/request lines vs body text, README/reference examples, and multi-request serializer round-trips.  
**Why:** These behaviors are most likely to drift during migration from line-by-line to grammar-driven parsing. Freezing them in tests provides a concrete, executable parity target. The serializer round-trip test ensures parse→serialize→parse cycles remain stable across the rewrite.

### 2026-03-20: Pest.rs parser migration recommended (consolidated)
**By:** Ripley (Lead), Bishop (Core), Hicks (Platform), confirmed by team  
**Date:** 2026-03-20  
**Status:** Approved for implementation  
**What:** Migrate `.http` file parser from handwritten state machine (`src/core/src/parser/file_parser.rs`, 500+ lines) to pest.rs (PEG parser generator). Public API (`parse_http_file`, `parse_http_content`) remains unchanged. All existing test suites remain valid if semantics match.  
**Why:** 
- Grammar file (`http-file.peg`) already exists as documentation (164 lines, well-specified)
- Handwritten parser is hard to understand; grammar spec divergence creates contributor friction  
- PEG makes parser behavior explicit and testable with ordered choice (directive-vs-comment precedence unambiguous)
- Separation of concerns: lexical parsing (pest) vs semantic post-processing (Rust)
- No platform, build, CI, distribution, or artifact impact (pest is compile-only, no runtime deps)

**Platform & Build Validation (Hicks):**
- Build time: +2-5s per full rebuild (proc-macro), incremental unaffected
- CI/CD: test.yml, release.yml: +2-5s per job (acceptable)
- All native targets (Linux, macOS, Windows) and WASM verified compatible
- Distribution (install scripts, Docker, binary size, version mgmt): unchanged
- Release artifacts: unchanged, grammar validation is just `cargo build`

**Critical Constraints (Bishop):**
- API stability: `parse_http_file()` and `parse_http_content()` signatures cannot change
- All 48 existing parser tests must pass unchanged (zero test modifications)
- Serializer round-trip test must pass (parser/serializer contract)
- Directive precedence, body mode state, whitespace handling, trailing token ignoring: all preserved
- Consumers (CLI, GUI native/WASM, TUI, serializer) use same API — no changes required
- WASM target must work (pest is pure Rust, no concern)

**Implementation Phases:**
1. Add pest dependencies, finalize grammar, new module structure (1-2 days)
2. Implement pest grammar parser, test parse tree generation (2-3 days)
3. Implement post-processing (parse tree → `Vec<HttpRequest>`), preserve state machine (3-4 days)
4. Run all tests (parser, serializer, integration, manual), validate performance <20% regression (2-3 days)
5. Cleanup old code, update documentation (1-2 days)

**Risks & Mitigations:**
- State machine complexity → Keep state in post-processing only, not grammar
- Precedence mismatches → Test directive vs. comment ordering with frozen tests
- Round-trip breakage → Run serializer tests early and often
- Performance regression → Benchmark before/after, gate on <20% increase
- WASM breaks silently → cargo check against wasm32-unknown-unknown
- Error message quality → Wrap pest errors with context

**Success Criteria (Lambert & Bishop):**
1. All 48 parser tests pass
2. Serializer round-trip test passes
3. CLI/GUI/TUI execution produces identical output
4. Parse time < 2x handwritten parser
5. Error messages remain actionable

**Pre-Migration Safety (Lambert):**
- Parser contract baseline tests added (24 new tests planned across P1-P5)
- Performance baseline established: benchmark current parser on examples/ dir (1000x iterations), 1000-request file, 10MB body file
- WASM compatibility verified (pest supports wasm32-unknown-unknown)
- After migration: ensure <20% throughput regression, <50% memory increase

**Routing:** Bishop (grammar authoring, AST converter), Lambert (pre-migration tests, performance benchmarks), Hicks (CI/platform validation), Vasquez (WASM parity verification), Ripley (PR review).

**Related Decisions:** Parser contract baseline (Bishop), parser safety requirements (Lambert).

### 2026-03-21: Pest.rs migration safety requirements (Lambert)
**By:** Lambert (Testing & Performance)  
**Date:** 2026-03-21  
**Severity:** P1 (parser is semantic core)  
**What:** Before pest migration, establish the following safety measures:
- Add 24 new tests (7 P1 blocking tests for stateful behavior/round-trip parity, 7 P2 high-value for directive buffering/request variables, 4 P3 error handling, 3 P4 edge cases, 3 P5 performance benchmarks)
- Fix serializer/parser round-trip (assertions, `@if-not`, `@pre-delay`/`@post-delay` must round-trip correctly)
- Establish performance baseline: benchmark current parser on examples/ dir (1000x iterations), 1000-request synthetic file, 10MB body file, ensure post-migration <20% throughput regression, <50% memory increase
- Verify WASM compatibility: confirm pest supports wasm32-unknown-unknown, test parser behavior identical in native and WASM builds

**Why:** Parser changes affect CLI, GUI, TUI, WASM execution paths. Known issues (#213 round-trip failure, #214 WASM execution divergence) mean high-risk change. PEG grammar is documentation, not executable; does not express stateful logic (body mode switching, directive buffering, IntelliJ skipping). Without pre-migration tests and baselines, no validation possible.

**Routing:** Bishop (core engine, parser implementation), Ripley (architectural review), Vasquez (WASM parity), Lambert (tests and benchmarks).

**Related:** Parser contract baseline (Bishop), pest migration plan (Ripley).

### 2026-04-06: Bishop pest scaffolding boundary
**By:** Bishop (Core & CLI Engineer)  
**Date:** 2026-04-06  
**What:** Keep `src/core/src/parser/http-file.peg` as the canonical human-readable parser spec, and mirror only grammar-owned syntax in `src/core/src/parser/http-file.pest` during Phase 1. Leave stateful behavior—body-mode transitions, directive buffering, body-line precedence, and semantic conversions—in Rust post-processing until later migration phases wire pest pairs into the production parser.

**Why:** This keeps the readable spec stable while giving the crate an executable grammar that can compile and be tested incrementally. It also isolates the highest-risk state machine behavior from the dependency-and-grammar slice so later phases can change one moving part at a time.

### 2026-04-06: Bishop parse-tree layer (Phase 2 completion)
**By:** Bishop (Core & CLI Engineer)  
**Date:** 2026-04-06  
**What:** Phase 2 of the pest migration now builds a line-oriented intermediate representation from the executable grammar. Each parsed node keeps its starting line number, raw matched source, and the grammar-selected syntactic classification (`directive`, `comment`, `request`, `header`, `variable`, `assertion`, `body`, or IntelliJ script block).

**Why:** This gives later migration phases a stable handoff point between the grammar pass and the handwritten parser's remaining state machine behavior. Body-mode downgrades, directive buffering, and request finalization can move onto the IR without reparsing the source text or rewalking raw pest pairs.

**Implications:** Future converter work should consume `PestHttpFile` rather than binding directly to pest `Pair` trees. Tests can now validate parse-tree shape separately from the production semantic contract.

### 2026-04-06: Bishop semantic assembler contract (Phase 3 strategy)
**By:** Bishop (Core & CLI Engineer)  
**Date:** 2026-04-06  
**What:** Phase 3 semantic assembler will consume `PestHttpFile` IR and reapply the handwritten parser's trim-based line classification rules and state machine behavior from each IR node's raw source text instead of trusting grammar-selected line kinds as the final contract.

**Why:** The legacy parser classifies directives, comments, headers, assertions, request lines, and IntelliJ script starts after trimming leading whitespace, and raises explicit errors for malformed `@timeout`, `@if`, and variable lines that the grammar can legally classify as plain comments or body text. Reusing raw source plus existing timeout/condition/substitution helpers keeps parity-sensitive behavior boring and reproducible.

**Implications:** 
- `src/core/src/parser/pest_semantic_assembler.rs` can be swapped under `parse_http_file()` / `parse_http_content()` later with a small call-site change instead of another semantic rewrite.
- Pest IR builders still need to preserve comment/separator nodes (`###`) and accurate line numbers, because the semantic layer depends on them for buffering and error context.
- Future grammar tightening must keep the parity tests green before the production backend swap.

## Governance

- All meaningful changes require team consensus
- Document architectural decisions here
- Keep history focused on work, decisions focused on direction
