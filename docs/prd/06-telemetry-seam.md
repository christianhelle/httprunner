# PRD 6 — A seam in front of telemetry

- **Strength:** Speculative
- **Dependency category:** ports & adapters
- **Status:** Proposed
- **Modules:** `src/core/src/telemetry/tracking.rs`, `telemetry/config.rs`, `telemetry/sanitize.rs`

## Summary

Telemetry is a global singleton with 28 free functions and no runtime
substitution. Introduce a `Telemetry` interface with two adapters — AppInsights
(prod) and InMemory (tests) — so emitted events can be observed.

## Current state

- `tracking.rs` owns `OnceLock<Mutex<TelemetryState>>` plus 28 `pub fn`
  (init, set_enabled, track_event, track_error, track_metric, track_cli_args,
  track_request_result, track_feature_usage, flush…). 1019 lines.
- Substitution is `cfg`-gated only (`feature = "telemetry"` / wasm); no trait, no
  injected backend.
- `sanitize.rs` strips PII; `redaction.rs` handles output (distinct concern).

## Problem

No seam: tests can't assert what telemetry emits. The module is overgrown and
coupled to a global, so behaviour is verified only by running the real client.

## Goals

- `trait Telemetry` capturing track/flush.
- AppInsights adapter (prod) + InMemory adapter (tests).
- Default global wiring unchanged for production.

## Deletion test

Two adapters make the seam real (prod + in-memory). Without a second adapter the
trait would be hypothetical — the in-memory test sink justifies it.

## Proposed design

1. Define `trait Telemetry { track(event); flush(); }`.
2. Wrap current state in the AppInsights adapter; keep global default.
3. InMemory adapter records events; tests assert on them.

## Testing

`cargo test -p httprunner-core telemetry` with the in-memory adapter.

## Risks

- Largest, lowest-value change; do last. Keep prod path byte-for-byte.

## Success metrics

- Tests assert emitted events; tracking surface shrinks behind one interface.

## References

- `src/core/src/telemetry/tracking.rs` (28 pub fns, OnceLock state)
- `telemetry/sanitize.rs`, `redaction.rs`
