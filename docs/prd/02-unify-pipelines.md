# PRD 2 — Unify the batch and incremental pipelines

- **Strength:** Strong (keystone)
- **Dependency category:** in-process, local-substitutable
- **Status:** Proposed
- **Modules:** `src/core/src/processor/executor.rs`, `processor/incremental_loop.rs`, `processor/incremental.rs`, `cli/src/main.rs`, `gui/src/results_view.rs`, `tui/src/app.rs`

## Summary

Two engines run `.http` requests and duplicate orchestration. The batch engine
(`process_http_files_with_config`, `executor.rs:498-554`) is used by the CLI; the
incremental engine (`process_requests_incremental`, `incremental_loop.rs:170`) is
used by GUI/TUI. Both implement dependency checks, condition evaluation,
substitution, assertions and delays. Deepen into one pipeline that emits
per-request events; CLI collects, UIs stream.

## Current state

- Batch: `executor.rs:498-554` loops files, applies config, totals, fail-fast.
- Incremental: `incremental_loop.rs:170-343` — deps, conditions, substitution,
  delays, callback + context tracking; unifies sync/async via `Sleep` trait
  (`incremental_loop.rs:38-61`).
- Front-ends bypass batch and call `process_http_file_incremental_with_executor`:
  GUI `gui/src/results_view.rs:180,357`; TUI `tui/src/app.rs:339`.
- CLI alone uses batch (`cli/src/main.rs:110`).

## Problem

Two modules implement the same run loop. Bugs (dependency skips, condition
evaluation, substitution precedence) must be fixed twice; behaviour drifts. ~800
lines duplicate intent across `executor.rs` and `incremental_loop.rs`.

## Goals

- One pipeline yielding per-request results (`RequestProcessingResult`).
- CLI buffers events into `ProcessorResults`; GUI/TUI stream them live.
- Preserve fail-fast, delays, dependencies, conditions, assertions.

## Non-goals

- Sync/async executor merge (PRD 3) — orthogonal, complementary.
- UI redesign.

## Deletion test

Deleting the batch loop and reimplementing CLI as "collect the event stream"
concentrates orchestration in one module; nothing lost. Confirms depth.

## Proposed design

1. Promote the incremental engine to the single pipeline; the **interface is the
   event stream** (`fn(index,total,RequestProcessingResult) -> continue`).
2. CLI sink: collect events → existing report/export. Drop the batch loop.
3. UIs keep streaming sink. Default executor = real HTTP; tests pass mock.
4. `ProcessorConfig` flows through; fail-fast = sink returns `false`.

## Migration

- Add a collecting sink in core; CLI uses it, removing `executor.rs` loop.
- Repoint CLI off `process_http_files_with_options` (see PRD 1).
- Verify GUI/TUI unchanged at the call site.

## Testing

`cargo test -p httprunner-core processor` plus a CLI-equivalent collector test.
Mock executor already exists (`processor/mock_executor.rs`).

## Risks

- CLI output parity (totals, per-file summary) — snapshot before/after.
- fail-fast semantics; cover with a halting-sink test.

## Success metrics

- One run loop; ~800 duplicate lines collapse.
- One place to test orchestration; three front-ends, one engine.

## References

- `src/core/src/processor/executor.rs:498-554`
- `src/core/src/processor/incremental_loop.rs:38-61,170-343`
- `gui/src/results_view.rs:180,357`, `tui/src/app.rs:339`, `cli/src/main.rs:110`
