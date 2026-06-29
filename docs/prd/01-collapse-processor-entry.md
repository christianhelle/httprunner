# PRD 1 — Collapse the file-processing entry into one interface

- **Strength:** Strong
- **Dependency category:** in-process
- **Status:** Proposed
- **Modules:** `src/core/src/processor/executor.rs`, `src/core/src/processor/mod.rs`, `src/cli/src/main.rs`

## Summary

`processor` exposes five entry points to run `.http` files. Four are shallow
wrappers that rebuild the same `ProcessorConfig` and call the fifth. Collapse to
one config-driven interface: `process_http_files(&config, &executor)`.

## Current state

`process_http_files_with_config<F>(config, executor)` holds the real loop
(`executor.rs:498-554`). The other four only assemble config and delegate:

- `process_http_files(7 args)` — `executor.rs:421-441`
- `process_http_files_with_options(9 args)` — `executor.rs:444-468`
- `process_http_files_with_silent(config)` — `executor.rs:470-474`
- `process_http_files_with_executor(6 args + F)` — `executor.rs:476-496`

`ProcessorConfig` already carries every flag (10 fields) with a builder
(`executor.rs:18-91`). External usage: CLI calls `_with_options` once
(`cli/src/main.rs:110-120`); the bare `process_http_files` wrapper is CLI-local
(`cli/src/main.rs:109`). `_with_silent` has zero call sites; `_with_executor`
is tests-only. GUI/TUI do not use these at all (they use the incremental path).

## Problem

The interface is nearly as wide as the implementation — a textbook shallow
surface. Every flag added means touching multiple wrappers; tests pick from five
look-alike functions.

## Goals

- One public entry: `process_http_files(&ProcessorConfig, &executor)`.
- Callers build a `ProcessorConfig`; default executor is real HTTP.
- No behavioural change.

## Non-goals

- Merging batch and incremental engines (PRD 2).
- Touching `ProcessorConfig` fields.

## Deletion test

Deleting `process_http_files`, `_with_options`, `_with_silent`, `_with_executor`
concentrates flag handling into config construction — capability preserved.
`_with_config` (rename to `process_http_files`) is the keeper.

## Proposed design

1. Keep `process_http_files_with_config`, rename to `process_http_files`.
2. Provide a default executor constructor (`runner::execute_http_request`) so
   most callers pass only config.
3. CLI builds `ProcessorConfig` from `Cli` directly.
4. Tests build config + mock executor — one interface.

## Migration

- Update `cli/src/main.rs` to build config + call the single entry.
- Replace test call sites of the four variants.
- `mod.rs` re-exports shrink to `ProcessorConfig` + `process_http_files`.

## Testing

`cargo test -p httprunner-core processor` is the gate. No new assertions;
existing `executor_tests.rs` retargeted to the single entry.

## Risks

- Wide test churn (`executor_tests.rs` ~2133 lines). Mechanical, low risk.

## Success metrics

- 5 entry points → 1; 4 shallow wrappers deleted.
- Adding a flag touches config only.

## References

- `src/core/src/processor/executor.rs:18-91,421-554`
- `src/cli/src/main.rs:109-134`
