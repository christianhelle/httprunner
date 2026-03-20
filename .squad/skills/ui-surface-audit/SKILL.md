---
name: "ui-surface-audit"
description: "Audit GUI, TUI, and WASM flows for hidden state, shortcut drift, and parity regressions"
domain: "ui-review"
confidence: "high"
source: "vasquez"
---

## Context

Use this skill when reviewing `src/gui` or `src/tui`, especially when the task asks for issue finding rather than implementation.

## Patterns

### Compare promises against gates

Start with the README and the surface-specific README files, then trace each promised shortcut or interaction back to `app.rs`. In this repo, the highest-value bugs come from documented behaviors that are still gated behind desktop-only state like `selected_file`.

### Verify persistence as a full round trip

A save path is not enough. For WASM, confirm that startup actually calls the code that hydrates LocalStorage-backed editor and environment state; for native/TUI, confirm that saved selections are explicitly restored during app initialization.

### Treat run actions as stateful workflows

Check whether buttons and shortcuts share the same enable/disable rules, and whether repeated runs are blocked while a result view is already active. Shared `Arc<Mutex<...>>` state with no run token or disable gate is a strong smell for interleaved results.

### Round-trip the full model

When a UI editor converts between editable fields and `HttpRequest`, verify that every important field survives the round trip. If the editor serializes only a subset of the request shape, log it as a data-loss bug even if the rest of the app still parses correctly.

## Anti-Patterns

- **Desktop-only assumptions in WASM code paths** — hiding load or run behavior behind file-system state in the web build.
- **Shortcut-specific behavior** — keyboard actions that run different logic than the visible buttons.
- **Partial serializers in editors** — request editors that drop assertions, variables, conditions, or delay metadata on save.
