# Architecture PRDs

Deepening opportunities for httprunner, framed in the shared design vocabulary
(**module**, **interface**, **implementation**, **depth**, **shallow/deep**,
**seam**, **adapter**, **leverage**, **locality**) and the deletion test. The aim
is testability and AI-navigability: fewer, deeper modules with one interface to
test against.

| # | PRD | Strength | Scope |
|---|-----|----------|-------|
| 1 | [Collapse the file-processing entry into one interface](01-collapse-processor-entry.md) | Strong | `core/processor` |
| 2 | [Unify the batch and incremental pipelines](02-unify-pipelines.md) | Strong (keystone) | `core/processor`, `cli`, `gui`, `tui` |
| 3 | [One HTTP execution interface, sync & async adapters](03-unify-executors.md) | Worth exploring | `core/runner` |
| 4 | [A core session module behind the GUI/TUI seam](04-core-session-module.md) | Worth exploring | `gui`, `tui`, `core` |
| 5 | [One substitution pass; delete the JSON wrapper](05-single-substitution-pass.md) | Worth exploring | `core/parser`, `core/variables`, `core/conditions` |
| 6 | [A seam in front of telemetry](06-telemetry-seam.md) | Speculative | `core/telemetry` |

**Recommended order:** #1 (warm-up) → #2 (keystone) → #3/#5 (reinforce locality) → #4 → #6.
