# Integration/Smoke Test Plan: CLI with Local HTTP Server

**Prepared by:** Bishop (Core & CLI Engineer)  
**Date:** 2026-04-06  
**Status:** Plan guidance (no implementation)

---

## Executive Summary

This plan outlines a strategy for adding integration/smoke tests that verify CLI execution flows against real HTTP requests, **without** internet dependencies. The approach uses a lightweight in-process or standalone HTTP server (e.g., `httptest-server` or a simple `tiny_http` instance) that runs for test duration, plus local `.http` files adapted to point at localhost instead of remote services.

**Key principle:** Tests verify behavior-critical CLI paths (file loading → parsing → execution → reporting/export → exit codes) while keeping test isolation tight and execution fast.

---

## Part 1: Recommended Cargo/Test Layout

### Workspace Structure

```
httprunner/
├── Cargo.toml                          (workspace root)
├── src/
│   ├── cli/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── tests/                      ← NEW: integration tests live here
│   │       ├── common/
│   │       │   ├── mod.rs              (shared test utilities)
│   │       │   └── http_server.rs      (lightweight server fixture)
│   │       ├── cli_basic_flow.rs       (basic CLI smoke tests)
│   │       ├── cli_env_and_vars.rs     (environment/variable binding)
│   │       ├── cli_report_export.rs    (reporting & export features)
│   │       ├── cli_conditions.rs       (@if, @dependsOn, exit codes)
│   │       └── http_files/             (local test .http files)
│   │           ├── simple.http
│   │           ├── with_env.http
│   │           ├── conditional.http
│   │           └── functions.http
│   ├── core/
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── gui/
│   └── tui/
└── examples/
    └── *.http                          (existing remote-only files, unchanged)
```

### Test Crate Setup

**New `tests/` directory in `src/cli/`:**
- Tests are **binary crates** that link httprunner-core and the CLI binary, then invoke the CLI as a subprocess.
- Each test file (e.g., `cli_basic_flow.rs`) is a separate test binary.
- `tests/common/mod.rs` exports shared fixtures:
  - `struct TestServer { addr: SocketAddr, handle: JoinHandle<()> }`
  - `fn start_test_server() -> TestServer`
  - `fn stop_test_server(server: TestServer)`
  - Helper functions to rewrite `.http` file URLs at test time.

**Key Cargo.toml additions for `src/cli`:**
```toml
[dev-dependencies]
tempfile = "3.27"
tokio = { version = "1.51", features = ["full"] }  # or use tiny_http (no async)
tiny_http = "0.15"                                  # lightweight, no deps
assert_cmd = "2.0"                                  # for subprocess assertions
predicates = "3.0"                                  # for output matching
```

### Why Separate from Production Code

- **Isolation:** Test binaries don't bloat the release binary.
- **Dependency flexibility:** Tests can pull in `tiny_http`, `tokio::test`, etc. without affecting CLI shipping deps.
- **Modular scope:** Each test file focuses on one behavior area (basic flow, env handling, reporting, etc.).
- **Fixtures in `common/`:** Shared server setup is centralized and reusable across all test binaries.

---

## Part 2: Coverage Matrix for CLI Execution Flows

### Tier 1: Fundamental CLI Paths (MUST TEST)

| Test | File | Behavior Verified | Exit Code | Notes |
|------|------|-------------------|-----------|-------|
| `smoke_single_file_success` | simple.http | Parse → execute GET → report success | 0 | Minimal happy path |
| `smoke_single_file_failure` | failing-request.http | Parse → execute 404 → exit with error | 1 | Failure propagation |
| `smoke_multiple_files` | basic.http + advanced.http | Process multiple files sequentially | 0 on all OK; 1 if any fail | Batch execution |
| `smoke_discover_mode` | Ad-hoc create 2 .http files in temp dir | `--discover` flag | 0 | File discovery walks temp tree |
| `smoke_help_flag` | N/A | `--help` on empty args | 0 | No files → help → exit 0 |

### Tier 2: Feature Integration (SHOULD TEST)

| Test | File | Behavior Verified | Exit Code | Notes |
|------|------|-------------------|-----------|-------|
| `env_substitution` | with_env.http | `{{HostAddress}}` binds from env file | 0 | `--env dev` loads http-client.env.json |
| `conditions_dependsOn` | conditional.http | `@dependsOn` blocks skipped requests if dependency fails | 1 | Dependency chain failure detected |
| `conditions_if_status` | conditional.http | `@if request.response.status 200` skips/executes | 0 | Conditional branches work |
| `conditions_if_body` | conditional.http | `@if request.response.body.$ <match>` extracts JSONPath | 0 | Body conditions parse & evaluate |
| `verbose_logging` | simple.http | `-v` flag writes request/response to stdout | 0 | Verbose output includes headers |
| `report_markdown` | simple.http | `--report` or `--report markdown` generates .md file | 0 | Markdown report written to disk |
| `report_html` | simple.http | `--report html` generates .html file | 0 | HTML report written to disk |
| `export_files` | simple.http | `--export` writes request/response files | 0 | .req and .resp files created |
| `export_json` | simple.http | `--export-json` writes single JSON file | 0 | JSON aggregate written to disk |
| `delay_between_requests` | two-request.http | `--delay 100` pauses between requests | 0 | Timing observable (slow test, optional) |
| `insecure_flag` | N/A | `--insecure` does not validate TLS (localhost is unverified) | 0 | No cert error on self-signed |
| `pretty_json` | post-body.http | `--pretty-json` formats bodies in verbose output | 0 | JSON is indented in output |

### Tier 3: Error Handling & Edge Cases (NICE TO TEST)

| Test | File | Behavior Verified | Exit Code | Notes |
|------|------|-------------------|-----------|-------|
| `malformed_http_file` | Invalid syntax | Parser error message on stderr | 1 | Clear parse failure |
| `missing_file` | ./nonexistent.http | "file not found" error | 1 | File I/O error handling |
| `empty_http_file` | (empty file) | Processes but no requests | 0 | Graceful no-op |
| `request_timeout` | (N/A—localhost is fast) | Timeout directive parsed (no actual timeout) | 0 | Parser accepts `@timeout` |
| `secrets_redaction` | with-auth.http | `Authorization` header hidden in output without `--include-secrets` | 0 | Security behavior verified |
| `secrets_included_flag` | with-auth.http | `--include-secrets` shows `Authorization` | 0 | Opt-in reveal works |
| `no_telemetry_flag` | simple.http | `--no-telemetry` disables telemetry (check env var or config) | 0 | Telemetry opt-out |
| `no_banner_flag` | simple.http | `--no-banner` suppresses donation banner | 0 | Output does not contain banner |

---

## Part 3: Example File Adaptation Strategy

### Current State
- All examples in `examples/*.http` point to **remote URLs** (`httpbin.org`, `jsonplaceholder.typicode.com`, `api.github.com`, etc.).
- These **work in user workflows** but are **not suitable for CI** (network flake, latency, external service outages).

### Recommendation: Dual-File Approach (No Deletion)

**Option A: Template + Localhost Mirror (PREFERRED)**
1. Keep `examples/*.http` as-is for user reference.
2. Create `src/cli/tests/http_files/` directory with **local-only copies**:
   - `src/cli/tests/http_files/simple.http` — GET `http://localhost:8080/status/200`
   - `src/cli/tests/http_files/conditional.http` — Same logic but `localhost:8080` endpoints
   - `src/cli/tests/http_files/with_env.http` — Environment variable injection test
   - `src/cli/tests/http_files/functions.http` — Function calls test (does not depend on response parsing)
3. Test server implements minimal mock endpoints:
   - `GET /status/{code}` — returns HTTP {code}
   - `POST /post` — echoes JSON body
   - `PUT /put` — returns 200 OK
   - `GET /get` — returns 200 with sample JSON
   - `DELETE /delete` — returns 200 OK

**Advantages:**
- Examples remain user-friendly and runnable against real APIs.
- Tests are deterministic and fast.
- No conflicts: examples folder is never read by tests.
- Easy to add new test scenarios without polluting examples.

**Option B: Conditional URL Rewriting (Alternative)**
- If you want to share `.http` files between test and user examples, implement **URL rewriting logic** at test invocation time.
- Read `examples/simple.http`, replace `https://httpbin.org` with `http://localhost:8080`, write to temp file, execute.
- **Pro:** DRY, examples are source of truth.
- **Con:** More complex fixture code; harder to debug if rewrites fail.

**Recommendation:** Use **Option A** (separate test files). It's clearer for contributors and avoids edge cases in URL rewrites.

### Environment File Handling

- Create `src/cli/tests/http_files/http-client.env.json` with local-only environment:
  ```json
  {
    "test": {
      "HostAddress": "localhost:8080",
      "ApiKey": "test-key-123",
      "Environment": "test"
    }
  }
  ```
- Tests use `--env test` to load it.
- Example `http-client.env.json` remains unchanged (user reference).

---

## Part 4: Interactions with Existing Features

### Parser & Execution Flow

The CLI flow (from `src/cli/src/main.rs`):
```
parse CLI args (Cli::parse)
  ↓
load_files() [discover or explicit]
  ↓
process_http_files_with_options()  [src/core/src/processor/executor.rs]
  ├─ parser::parse_http_file() for each file
  ├─ runner::execute_http_request() for each request (respects @if, @dependsOn)
  └─ returns ProcessorResults { success: bool, files: Vec<...> }
  ↓
generate_report() [--report, --report markdown/html]
  ↓
export_results() [--export]
  ↓
export_json_results() [--export-json]
  ↓
ensure_processor_success() [exit code: 0 if success, 1 if failed requests exist]
```

**Test Integration Points:**
- **Parser:** Tests verify that local `.http` files parse without error.
- **Execution:** Tests verify requests reach the local server and return expected status codes.
- **Conditions:** Tests verify `@if`, `@dependsOn`, and conditional skipping work end-to-end.
- **Reporting:** Tests verify `--report` and `--export` flags generate files correctly.
- **Exit codes:** Tests verify exit code reflects execution success/failure (critical for CI/CD).
- **Logging:** Verbose mode (`-v`) and log file output (`--log`) are tested to ensure they don't crash.

### Key Issues to Be Aware Of

From `.squad/history.md` and `.squad/decisions.md`:
1. **Exit codes (#231):** CLI currently ignores `ProcessorResults.success` in some paths. Tests must verify that exit codes match failure detection.
2. **Request-variable extraction (#232):** Tests using response variables (e.g., `@first_request_guid={{request1.response.body.$.json.guid}}`) must account for the JSON string-scanning quirks.
3. **Condition parsing (#233):** `@if` using `==` is treated as literal text, not comparison. Tests avoid `==` in conditions; use status codes or JSONPath body checks instead.
4. **Parser strictness (#234):** Malformed `@timeout` and unresolved `{{request.response...}}` are warnings, not errors. Tests may pass even with malformed directives.

**For test purposes:** Keep `.http` files simple to avoid triggering these edge cases. Use status codes (`@if request.response.status 200`) rather than complex JSON extraction.

### Logging & Telemetry

- Tests should run with `--no-telemetry` to avoid sending test data to AppInsights.
- Logging (`--log` / `--log <filename>`) should not interfere with concurrent test execution; use temp directories or unique log names per test.

---

## Part 5: Risks, Tradeoffs & Mitigations

### Risk 1: Network Port Collisions (LOW)

**Risk:** Multiple tests start servers on the same port; one fails.

**Mitigations:**
- Use `0` (OS-assigned ephemeral port) when starting test server.
- Extract actual port from server address and rewrite `.http` files at runtime.
- Or use separate port ranges per test (8080–8089, 8090–8099, etc.).

**Effort:** Low. Tiny_http automatically assigns port `0`; extract and use.

### Risk 2: Test Flakiness (MEDIUM)

**Risk:** Concurrent test execution or filesystem race conditions cause intermittent failures.

**Mitigations:**
- Use `tempfile` crate for isolated directories per test.
- Each test has its own temp directory with unique report/export filenames.
- Avoid `cd` into shared directories; tests should be hermetic.
- If using `assert_cmd`, set reasonable timeouts (e.g., 5s per CLI invocation).

**Effort:** Medium. Requires careful fixture design; `tempfile` and unique naming solve most cases.

### Risk 3: Server Shutdown Hang (LOW)

**Risk:** Test server thread panics or deadlocks; cleanup doesn't run; subsequent tests hang.

**Mitigations:**
- Wrap server startup/shutdown in `Drop` trait so cleanup runs even if test panics.
- Use `tokio::spawn` with explicit cancellation token if using async server.
- Set a hard timeout on test execution (e.g., 30s) via Cargo test runner config.

**Effort:** Low to medium. Rust's ownership model helps; explicit `Drop` impl is cheap.

### Risk 4: Parsing or Execution Differences (MEDIUM)

**Risk:** Tests pass locally but fail in CI because of environment differences (e.g., Windows vs. Linux line endings, path separators, DNS resolution).

**Mitigations:**
- Use `tempfile` to generate paths in the OS-native format.
- Use `http://127.0.0.1:port` instead of `localhost` to avoid DNS resolution.
- Test on both Windows and Linux in CI (GitHub Actions already does this).
- If testing file discovery (`--discover`), create `.http` files with platform-aware line endings.

**Effort:** Medium. Requires awareness of platform differences; tempfile and explicit paths solve most cases.

### Risk 5: Incomplete Coverage (MEDIUM)

**Risk:** Integration tests don't exercise all CLI paths; regressions slip through.

**Mitigations:**
- Map all CLI flags (from `src/cli/src/cli/args.rs`) to test cases (see Part 2 coverage matrix).
- Include at least one test per major feature (discovery, reporting, export, conditions, env, verbose, logging, secrets).
- Run tests with `cargo test --all` to catch any breakage.
- Consider adding a checklist in the PR description if adding new CLI flags.

**Effort:** Medium. The coverage matrix in Part 2 gives you a roadmap; implement incrementally.

### Risk 6: Long Test Duration (LOW-MEDIUM)

**Risk:** Tests take too long because of network latency or file I/O.

**Mitigations:**
- Localhost requests are fast (< 10ms). No external network.
- Disable telemetry (`--no-telemetry`) to avoid AppInsights calls.
- Use `tiny_http` (no dependencies, single-threaded) for simplicity; upgrade to tokio only if needed for concurrency.
- Run tests in parallel: `cargo test -- --test-threads=4` (default).
- Aim for < 5s total test suite execution.

**Effort:** Low. Localhost is inherently fast; avoid file I/O, and you're good.

---

## Part 6: Clarifying Questions for the User

Before starting implementation, please clarify:

### Q1: Test Server Framework Preference
- **Tiny_http** (pure Rust, no async, minimal deps): Simplest, < 50 lines to mock endpoints.
- **Tokio + axum** (async, more flexible, more deps): Overkill for simple mocks but future-proof.
- **Mockito or httptest** (dedicated test libraries): Designed for this use case but adds dependencies.

**Recommendation:** Start with **tiny_http** for simplicity; it's sufficient for status codes, echo endpoints, and request/response pairs.

### Q2: Should Tests Verify Exact Report Output?
- **Yes (strict):** Tests assert that markdown/HTML reports contain specific sections, request counts, etc. Requires updating tests if report format changes.
- **No (loose):** Tests only verify that reports are generated (file exists, not empty). Faster to maintain but lower coverage.

**Recommendation:** Start loose (file exists, not empty). Add strict assertions for critical paths (failure detection, request counts) later if needed.

### Q3: Should Tests Cover WASM/GUI/TUI CLI Variants?
- The plan above focuses on **native CLI** (`src/cli/`).
- GUI and TUI have their own execution paths (not CLI-based).
- Should tests cover those, or keep scope to CLI only?

**Recommendation:** Keep scope to **CLI only** (native, non-WASM). GUI/TUI are Vasquez's domain and have separate execution paths. If those need tests, they get their own suite.

### Q4: Should Tests Run in CI/CD?
- Add tests to `test.yml` workflow, or keep them local-only for now?
- If CI: Ensure GitHub Actions has ports available (should be fine on runners).

**Recommendation:** Add to `test.yml` as a new job or step. They run fast and provide regression safety.

### Q5: Should Examples Be Updated to Show Localhost Usage?
- Add a new section to README.md explaining how to write local-testable `.http` files?
- Or leave examples as-is (remote-only) and document the test structure separately?

**Recommendation:** Leave examples as-is. Add a new `TESTING.md` in `src/cli/tests/` explaining how to write and run integration tests. Keep examples user-friendly.

---

## Summary of Decisions

| Area | Recommendation | Rationale |
|------|---|---|
| **Layout** | Separate `src/cli/tests/` (binary crate, not unit tests) | Isolation, clarity, no bloat to release binary |
| **Example Handling** | Dual-file (keep `examples/*` remote, add `tests/http_files/*` local) | DRY, clarity, no conflicts |
| **Server Framework** | tiny_http (lightweight, minimal deps) | Simplicity, fast startup, sufficient for mock endpoints |
| **Coverage** | Tier 1 (fundamentals) + Tier 2 (features) = ~15–20 tests | Covers all CLI flags and execution paths |
| **Exit Codes** | Test that success/failure is correctly reflected (0 vs. 1) | Critical for CI/CD integration |
| **Secrets Handling** | Test both redacted (default) and included (`--include-secrets`) | Security behavior verified |
| **Telemetry** | Tests use `--no-telemetry` | Avoid polluting telemetry with test data |
| **CI Integration** | Add to `test.yml` as a new step | Fast, deterministic, catches regressions |

---

## Implementation Sequence (Recommended)

1. **Phase 0 (foundation):** Create `src/cli/tests/` directory; add `tests/common/http_server.rs` with lightweight server fixture.
2. **Phase 1 (Tier 1):** Implement 5 fundamental tests (single file, multiple files, discover, help, failure detection).
3. **Phase 2 (Tier 2):** Add feature tests (env, conditions, reporting, export, verbose).
4. **Phase 3 (Tier 3):** Add edge cases (malformed files, missing files, secrets redaction).
5. **Phase 4 (CI):** Integrate into `test.yml`; verify all tests pass on Linux, macOS, Windows.
6. **Phase 5 (docs):** Write `TESTING.md` explaining structure and how to add new tests.

Each phase is ~1–2 days of work.

---

## Appendix: Example Test Skeleton

```rust
// src/cli/tests/common/mod.rs
use std::net::SocketAddr;
use tiny_http::{Server, Response};
use std::thread;

pub struct TestServer {
    addr: SocketAddr,
    #[allow(dead_code)]
    handle: thread::JoinHandle<()>,
}

impl TestServer {
    pub fn start() -> Self {
        let server = Server::http("127.0.0.1:0").unwrap();
        let addr = server.server_addr();
        
        let handle = thread::spawn(move || {
            for request in server.incoming_requests() {
                let path = request.url();
                let response = match path {
                    "/status/200" => Response::from_string("OK").with_status_code(200),
                    "/status/404" => Response::from_string("Not Found").with_status_code(404),
                    _ => Response::from_string("OK").with_status_code(200),
                };
                let _ = request.respond(response);
            }
        });
        
        TestServer { addr, handle }
    }
    
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}

// src/cli/tests/cli_basic_flow.rs
#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn smoke_single_file_success() {
        let server = crate::common::TestServer::start();
        let temp_dir = TempDir::new().unwrap();
        
        let http_file = temp_dir.path().join("test.http");
        fs::write(&http_file, format!("GET http://{}/status/200", server.addr())).unwrap();
        
        let mut cmd = Command::cargo_bin("httprunner").unwrap();
        cmd.arg(http_file).assert().success();
    }
}
```

---

**End of Plan.** Ready for user review and feedback before implementation.
