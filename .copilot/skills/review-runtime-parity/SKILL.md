---
name: "review-runtime-parity"
description: "Checklist for reviewing parser/serializer parity, native-vs-WASM behavior, and large-payload hot paths"
domain: "testing-performance"
confidence: "high"
source: "Lambert review 2026-03-20"
---

## Context

Use this skill when reviewing changes in `src/core`, `src/gui`, or `src/tui` that touch parsing, serialization, request editing, execution flow, or UI state.

## Patterns

### Parse/serialize parity

- Treat `src/core/src/parser/file_parser.rs` and `src/core/src/serializer.rs` as a round-trip contract.
- Every directive emitted by the serializer must be accepted by the parser.
- Round-trip tests should cover assertions, `@dependsOn`, `@if`, `@if-not`, `@pre-delay`, and `@post-delay`.

### GUI editor parity

- Review `src/gui/src/request_editor.rs` for fields that are reconstructed manually.
- Confirm edited requests keep all semantics from `HttpRequest`, not just the visible subset shown in the UI.

### Native vs WASM execution parity

- Compare `src/gui/src/results_view.rs` and `src/gui/src/results_view_async.rs` whenever execution semantics change.
- Single-request execution must preserve prior-request context, dependency handling, request variables, and environment-sensitive substitutions.

### Large-payload hot paths

- Watch for `.clone()` of `HttpRequest`, `HttpResult`, `ExecutionResult`, and full `Vec<PathBuf>` collections inside per-request or per-frame code.
- Watch for full-state serialization in response to routine UI interactions like toggles, selections, and font changes.

## Examples

- `RequestEditor::to_http_request()` should not reset assertions, conditions, or delay fields.
- `FileTree` discovery should batch discovery work and sort once, not on every insert.
- Result persistence should avoid cloning full response bodies just to remember window state.

## Anti-Patterns

- Serializer emits syntax the parser does not understand.
- WASM request execution uses a different semantic path than native/CLI without parity tests.
- UI code clones or serializes entire result sets on routine interactions.
