# PRD 4 — A core session module behind the GUI/TUI seam

- **Strength:** Worth exploring
- **Dependency category:** local-substitutable
- **Status:** Proposed
- **Modules:** `gui/src/state.rs`, `tui/src/state.rs`, `gui|tui/file_tree.rs`, `gui|tui/environment_editor.rs`, `gui|tui/results_view.rs`, `src/core`

## Summary

GUI and TUI duplicate non-rendering work: discovery, environment resolution and
result mapping. Deepen a core session module that owns these; front-ends keep
only rendering and share one result DTO.

## Current state

- App state duplicated: `gui/src/state.rs:7-25`, `tui/src/state.rs:4-26` (same
  core fields; diverging persistence).
- Env resolution copied: `gui/src/environment_editor.rs:41-63`,
  `tui/src/environment_editor.rs:77-96`.
- File discovery/sort copied: `gui/src/file_tree.rs:16-77`, `tui/src/file_tree.rs:15-63`.
- `RequestProcessingResult` → local `ExecutionResult` mapping copied:
  `gui/src/results_view.rs:36-88`, `tui/src/results_view.rs:7-44`.

## Problem

Two front-ends maintain parallel logic; only rendering legitimately differs. A
mapping bug or env-resolution change must be made twice and tested through two UIs.

## Goals

- Core `session` module: discovery, env resolution, run, shared result DTO.
- GUI/TUI render the DTO; CLI may adopt later.
- Logic testable without a UI.

## Non-goals

- Unifying rendering or widget toolkits.

## Deletion test

Delete GUI/TUI env+discovery+mapping; have both call core session — complexity
concentrates in one tested module rather than scattering across UIs.

## Proposed design

1. Core `session`: `discover()`, `resolve_env(file)`, run via PRD 2 pipeline,
   shared `RequestResult` DTO.
2. Front-ends hold render state only; map DTO to widgets.

## Testing

`cargo test -p httprunner-core` for session; UI crates compile-check.

## Risks

- wasm persistence divergence (GUI localStorage) — keep persistence in UI.

## Success metrics

- Discovery/env/mapping defined once; UIs shrink to rendering.

## References

- `gui/src/state.rs:7-25`, `tui/src/state.rs:4-26`
- `gui/src/results_view.rs:36-88`, `tui/src/results_view.rs:7-44`
