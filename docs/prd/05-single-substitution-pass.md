# PRD 5 — One substitution pass; delete the JSON wrapper

- **Strength:** Worth exploring
- **Dependency category:** in-process
- **Status:** Proposed
- **Modules:** `parser/substitution.rs`, `variables/substitution.rs`, `functions/substitution.rs`, `request_substitution.rs`, `conditions/json_extractor.rs`, `variables/json.rs`

## Summary

Three substitution scanners re-walk each request field independently, and one
JSON module just forwards to another. Consolidate to one substitution pass and
delete the forwarding wrapper.

## Current state

- Variable subst: `parser/substitution.rs:3` (`{{var}}`).
- Request-variable subst: `variables/substitution.rs:6` (`{{name.response.body}}`).
- Function subst: `functions/substitution.rs:127` (`guid()`, `name()`…).
- Per-request orchestration: `request_substitution.rs:6-37`.
- `conditions/json_extractor.rs:4-10` is a 6-line wrapper over
  `variables::extract_json_property` (`variables/json.rs:9`).

Each scanner walks the string for `{{…}}` separately (`parser/substitution.rs:3-50`,
`variables/substitution.rs:6-49`, functions loop `functions/substitution.rs:148-153`).

## Problem

Three independent scans; precedence is implicit and spread across modules. The
JSON extractor adds a module that only forwards — interface ≈ zero implementation.

## Goals

- One substitution pass with explicit precedence (vars → request-vars → funcs).
- Delete `conditions/json_extractor.rs`; callers use `extract_json_property`.

## Deletion test

`json_extractor` passes outright — delete it, call through. The three scanners
fold into one pass; logic concentrates rather than scatters.

## Proposed design

1. Single `substitute(text, ctx)` walking `{{…}}` once, dispatching by token kind.
2. Function trait registry stays (`functions/substitution.rs:127-146`).
3. Remove the JSON wrapper; repoint condition callers.

## Testing

`cargo test -p httprunner-core variables functions conditions parser`.

## Risks

- Precedence regressions — lock with substitution tests first.

## Success metrics

- One scan per string; one shallow module deleted.

## References

- `parser/substitution.rs:3`, `variables/substitution.rs:6`, `functions/substitution.rs:127`
- `conditions/json_extractor.rs:4-10`, `variables/json.rs:9`
