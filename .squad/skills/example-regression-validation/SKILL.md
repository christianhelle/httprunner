---
name: "example-regression-validation"
description: "Validate checked-in .http examples with parser assertions plus MockHttpExecutor request-chain tests"
domain: "core-cli"
confidence: "high"
source: "Bishop regression fix 2026-04-06"
---

## Context

Use this skill when a checked-in example under `examples/` starts failing in CLI or core execution after parser, substitution, or runner changes.

## Patterns

### Lock the parse shape first

- Add a parser test that loads the real example file.
- Assert the preceding request body does **not** absorb helper directives or variables.
- Assert the downstream request still contains the request-variable/function expressions you expect after parser-level variable substitution.

### Validate execution without the network

- Add a `process_http_file_incremental_with_executor(...)` test in `src/core/src/processor/incremental_tests.rs`.
- Feed `MockHttpExecutor` a response body that matches the example's expected request-variable paths.
- Assert the executed downstream request body contains the resolved request-variable values and any derived function output.

## Examples

- `examples/functions.http` uses helper variables derived from `request1.response.body.$.json.guid`; the regression test should confirm those helpers stay outside `request1`'s body and resolve to concrete values in `request2`.

## Anti-Patterns

- Relying on live `httpbin.org` calls in unit tests.
- Updating the example file without a test that loads the checked-in example.
- Treating `@...` lines after a JSON body as helper variables without proving the parser leaves body mode first.
