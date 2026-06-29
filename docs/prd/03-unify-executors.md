# PRD 3 — One HTTP execution interface, sync & async behind it

- **Strength:** Worth exploring
- **Dependency category:** ports & adapters
- **Status:** Proposed
- **Modules:** `src/core/src/runner/executor.rs`, `runner/executor_async.rs`, `runner/response_processor.rs`, `runner/url_encoding.rs`

## Summary

Sync and async executors duplicate client construction, request building and
response capture, differing only by `.await`. Share the builders behind one
`execute` interface with two adapters: blocking (CLI) and async (wasm).

## Current state

- Sync: `runner/executor.rs:102-263` (blocking reqwest). build_client `144-190`,
  build_request `193-216`, capture `250-263`.
- Async: `runner/executor_async.rs:17-127`. build_client_async `55-91`,
  build_request_async `93-113`, capture_async `115-127`.
- Shared helpers exist: `response_processor.rs`, `url_encoding.rs`.

## Problem

Two near-verbatim implementations. Timeout, header and form-encoding fixes land
in one and miss the other. Two adapters already exist (blocking + wasm), so the
seam is real — but the implementation is copied, not shared.

## Goals

- One `execute(request, insecure)` interface; two adapters behind it.
- Client config, request building, response capture written once.
- No behaviour change for CLI or wasm.

## Deletion test

Deleting the async copies and routing both through shared builders concentrates
encoding/timeout logic in one place — capability kept. Two adapters justify the
seam (one adapter would be hypothetical; two is real).

## Proposed design

1. Extract pure builders (client config, header map, form-encode) used by both.
2. Keep blocking and async as thin adapters that await/block over shared steps.
3. Capture path shares `response_processor` helpers.

## Migration

- Factor builders; point sync + async at them; delete duplicate bodies.

## Testing

`cargo test -p httprunner-core runner` plus existing `executor_async` tests.

## Risks

- wasm `cfg` gates — keep adapters cfg-split, builders shared and platform-free.

## Success metrics

- Runner duplication ~halved; one place for client/headers/encoding.

## References

- `src/core/src/runner/executor.rs:144-263`
- `src/core/src/runner/executor_async.rs:55-127`
