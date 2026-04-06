---
name: "parser-migration-checklist"
description: "Risk assessment checklist for migrating parsers to PEG/parser-generator tools"
domain: "testing-performance"
confidence: "high"
source: "Lambert pest.rs migration assessment 2026-03-20"
---

## Context

Use this skill when evaluating a parser rewrite or migration to a parser generator (pest, nom, tree-sitter, etc.) in any project. This checklist identifies risks that must be addressed before migrating from a handwritten parser.

## Checklist

### 1. Audit Existing Test Coverage

**Identify protected behaviors:**
- [ ] Count existing parser tests by category (integration, unit, edge case)
- [ ] List all syntax constructs with test coverage
- [ ] List all error cases with test coverage

**Identify coverage gaps:**
- [ ] Stateful parser behavior (mode switches, context buffering)
- [ ] Serializer/parser round-trip (if applicable)
- [ ] Edge cases (whitespace, line endings, malformed input)
- [ ] Substitution and variable expansion
- [ ] Performance on large inputs

### 2. Identify Migration Risks

**State machine logic:**
- [ ] Does the handwritten parser use state flags or modes?
- [ ] Can the target parser generator express stateful behavior?
- [ ] If not, can state be moved to semantic actions?

**Grammar vs implementation drift:**
- [ ] If grammar exists, is it documentation or executable?
- [ ] Line-by-line audit: does grammar match implementation?
- [ ] Document intentional deviations

**Serializer/parser parity:**
- [ ] Does a serializer exist for this parser?
- [ ] Are round-trip tests present and comprehensive?
- [ ] Do they test all fields, not just a subset?

**Cross-platform compatibility:**
- [ ] Does the target parser generator support all required platforms (WASM, embedded, etc.)?
- [ ] Are there conditional compilation differences?

### 3. Add Pre-Migration Tests

**Priority 1 (Blocking):**
- [ ] Stateful behavior: mode switches, context buffering
- [ ] Round-trip tests: parse → serialize → parse again
- [ ] Edge cases: empty input, comments-only, malformed input

**Priority 2 (High Value):**
- [ ] Substitution and expansion edge cases
- [ ] Nested constructs (variables in variables, etc.)
- [ ] Cross-platform edge cases (line endings, whitespace)

**Priority 3 (Performance):**
- [ ] Benchmark current parser on representative inputs
- [ ] Benchmark on large inputs (1000x scale, 10MB payloads)
- [ ] Measure allocation patterns and throughput

### 4. Migration Decision Points

**Before starting migration:**
- [ ] Can target parser generator handle state machine logic?
- [ ] Is WASM/cross-platform compatibility confirmed?
- [ ] Do performance benchmarks exist?

**During migration:**
- [ ] All existing tests pass with new parser?
- [ ] New tests pass (from step 3)?
- [ ] Performance within acceptable bounds (<20% regression)?

**After migration:**
- [ ] Round-trip tests pass?
- [ ] WASM/cross-platform parity verified?
- [ ] Documentation updated with migration notes?

### 5. Acceptance Criteria

**Migration is complete when:**
- [ ] All existing tests pass (100% pass rate)
- [ ] All new tests pass
- [ ] Performance benchmarks show <20% throughput regression
- [ ] Memory usage shows <50% increase on large inputs
- [ ] Round-trip tests pass (parse → serialize → parse)
- [ ] Cross-platform parity verified (native, WASM, etc.)
- [ ] Documentation updated

## Anti-Patterns

**Do not migrate if:**
- ❌ No baseline tests exist for current parser
- ❌ No performance benchmarks exist
- ❌ Round-trip tests fail or are missing
- ❌ Target parser generator does not support required platforms
- ❌ State machine logic cannot be expressed in target generator

**Do not assume:**
- ❌ Parser generator is automatically faster (measure!)
- ❌ Grammar documentation matches implementation (audit!)
- ❌ Round-trip tests are comprehensive (validate all fields!)

## Examples

**Good approach:**
1. Add 20+ pre-migration tests for gaps
2. Establish performance baseline (3+ benchmarks)
3. Audit grammar vs implementation line-by-line
4. Implement new parser alongside old (not replacing)
5. Run parity tests (old vs new parser on all examples)
6. Verify <20% performance regression
7. Replace old parser only after all tests pass

**Bad approach:**
1. Start rewriting parser without baseline tests
2. Assume parser generator handles edge cases
3. Replace old parser before verifying parity
4. Discover round-trip bugs in production

## When to Skip Migration

**Do not migrate if:**
- Performance benchmarks show >20% regression
- State machine logic requires complex semantic actions
- Target parser generator does not support required platforms
- Round-trip tests cannot be made to pass
- Risk outweighs benefit (handwritten parser is already well-tested and fast)

## Related Patterns

- **Round-trip testing:** Parse → serialize → parse must produce identical output
- **Parity testing:** Old parser vs new parser on same inputs must agree
- **Performance regression testing:** Benchmark before and after migration
