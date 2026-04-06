# Skill: Pest.rs Parser Migration in Rust Workspaces

**Author:** Hicks (Platform & Release Engineer)  
**Level:** Intermediate  
**Context:** Rust workspaces shipping CLI/GUI/TUI + WASM binaries  
**Date:** 2026-03-21  

---

## What This Skill Covers

How to migrate a handwritten parser to **pest.rs** (PEG parser generator) in a multi-target Rust workspace without introducing build, CI, or distribution friction.

---

## When to Use This Skill

- Your project has a handwritten or regex-based parser that's hard to maintain
- You have a PEG-style grammar spec (or can write one)
- You ship native + WASM binaries and need to validate cross-target compatibility
- You want to lower contributor onboarding friction by making grammar executable

---

## Key Principles

### 1. Pest is Compile-Only
- pest + pest_derive are proc-macro dependencies
- They expand at compile-time on the build host, not the target
- No binary bloat, no runtime overhead, zero artifact size increase

### 2. WASM & Cross-Platform Work by Default
- Proc-macro runs on build host (Linux, macOS, or Windows)
- Generated parser code is pure Rust (no platform-specific logic)
- WASM target works identically to native targets
- No new cfg gating needed

### 3. Compile Time Trade-Off Is Acceptable
- Adds ~2-5s per full rebuild (proc-macro expansion)
- Incremental rebuilds unaffected (grammar file rarely changes)
- In CI context, acceptable even for multi-target matrices

### 4. Grammar Becomes Source of Truth
- The `.pest` file is now the spec—keep it well-documented
- Existing test suite validates semantics (no test rewrites if behavior matches)
- Easier for contributors to understand and modify syntax
- If you already have a documentation grammar, keep that human-readable file canonical during a staged migration and make the executable `.pest` file explicitly document which stateful behaviors still live in Rust post-processing

### 5. Add a Line-Oriented IR Before Rewriting Semantics
- Keep the production parse entry points unchanged while you validate the executable grammar
- Convert `Pair`s into an internal file/line IR that preserves source line numbers, raw matched text, and grammar-selected node types
- Let later phases consume that IR to reapply stateful behaviors (body mode, directive buffering, request finalization) without reparsing source text or matching over raw `Pair`s everywhere

### 6. Reapply Legacy Semantics From IR Raw Text
- If the handwritten parser classifies lines after trimming whitespace, keep that trim-based contract in the semantic assembler even when the grammar classifies fewer shapes
- Use IR node kinds for structure the grammar owns (for example grouped script blocks or exact line numbers), but fall back to each node's `raw` text for leading-space directives, headers, assertions, malformed variable lines, and invalid directive errors
- Keep existing timeout parsing, condition parsing, and substitution helpers in the semantic layer so delayed substitutions and malformed directives fail exactly where the old parser failed

---

## Migration Checklist

### Phase 1: Dependency Setup
```toml
[workspace.dependencies]
pest = "2.7"
pest_derive = "2.7"

# In your parser crate (e.g., src/core/Cargo.toml)
[dependencies]
pest = { workspace = true }
pest_derive = { workspace = true }
```

**Verify:**
- `cargo check` succeeds on all platforms
- `cargo check --target wasm32-unknown-unknown` succeeds

### Phase 2: Grammar File Conversion
- Start with your existing PEG grammar (comment-based spec)
- Convert to pest syntax (mostly 1:1; mostly just rename `.peg` to `.pest`)
- Add comments explaining complex rules
- Place in `src/crate_name/src/parser/http-file.pest` (or similar)

**Key pest syntax notes:**
- Rules: `rule_name = { expression }` (not `rule_name <- expression`)
- Modifiers: `_name = {}` (silent), `@name = {}` (atomic), `$name = {}` (compound)
- Quantifiers: `*`, `+`, `?` work as in PEG
- Alternation: `|` works as in PEG
- Negation: `!` and `&` work as in PEG

### Phase 3: Parser Implementation
```rust
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/http-file.pest"]  // relative to src/
pub struct HttpFileParser;

// Usage in your parser module:
pub fn parse_http_content(content: &str) -> Result<Vec<HttpRequest>> {
    let pairs = HttpFileParser::parse(Rule::HttpFile, content)?;
    // Build your output from pairs
    Ok(...)
}
```

**Adapt existing code:**
- Replace line-by-line parsing with pest Pair iteration
- ParserState / context variables remain similar
- Substitution and condition parsing can stay as separate modules

### Phase 4: Testing
- Run existing test suite (should validate identical behavior)
- Fix any semantic mismatches
- Test WASM target: `cargo check --target wasm32-unknown-unknown`

### Phase 5: Performance & CI Validation
```bash
# Before merge, measure:
cargo clean && time cargo build --release

# Verify cross-platform builds:
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

Gate on: compile-time increase <20%, all tests pass, end-to-end flows work.

---

## Common Pitfalls

### Pitfall 1: Proc-Macro Errors Break at Compile-Time
**Problem:** Syntax error in grammar file breaks the entire build.  
**Solution:** Test grammar syntax with `cargo check` frequently during migration.

### Pitfall 2: Pest Error Messages Are Different
**Problem:** Pest's parser errors differ from your handwritten parser.  
**Solution:** Wrap pest errors in your own error type; document differences in CHANGELOG.

### Pitfall 3: Existing Tests Break Due to Refactoring
**Problem:** Parser API changes break test setup.  
**Solution:** Keep public API (`parse_http_file`, `parse_http_content`) identical. Refactor internals only.

### Pitfall 4: WASM Silently Succeeds Locally but Fails in CI
**Problem:** You didn't test WASM target.  
**Solution:** Always run `cargo check --target wasm32-unknown-unknown` as gate.

### Pitfall 5: Grammar File Comment Syntax Wrong
**Problem:** Pest uses `//` comments; old grammar used `#` for documentation.  
**Solution:** Convert comments; link to pest.rs book in your grammar.

### Pitfall 6: Treating Grammar Classification as the Whole Legacy Contract
**Problem:** A staged migration starts silently accepting or ignoring lines differently because the grammar does not encode leading-whitespace handling or malformed-directive errors from the handwritten parser.  
**Solution:** Build the semantic assembler on the IR, but re-run the old trim-based classification and helper parsers against each node's `raw` text anywhere parity depends on legacy behavior.

---

## Example: Minimal Pest Grammar & Parser

**File: `src/parser/example.pest`**
```pest
HttpFile = { SOI ~ (Line)* ~ EOI }
Line = _{ BlankLine | CommentLine | DirectiveLine | RequestLine | HeaderLine | BodyLine }

BlankLine = { WHITESPACE* ~ NEWLINE }
CommentLine = { ("#" | "//") ~ (!" " | !NEWLINE)* ~ NEWLINE? }
DirectiveLine = { ("#" | "//") ~ "@" ~ Identifier ~ " " ~ (!"NEWLINE" ~ ANY)* ~ NEWLINE? }
RequestLine = { Method ~ WHITESPACE+ ~ Url ~ NEWLINE? }
HeaderLine = { HeaderName ~ ":" ~ WHITESPACE* ~ HeaderValue ~ NEWLINE? }
BodyLine = { (!NEWLINE ~ ANY)+ ~ NEWLINE? }

Method = @{ "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS" }
Url = @{ (!"WHITESPACE" ~ !"NEWLINE" ~ ANY)+ }
HeaderName = @{ (!":" ~ !NEWLINE ~ ANY)+ }
HeaderValue = @{ (!NEWLINE ~ ANY)* }
Identifier = @{ ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_")* }

WHITESPACE = _{ " " | "\t" }
NEWLINE = _{ "\r\n" | "\n" | "\r" }
```

**File: `src/parser/mod.rs`**
```rust
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/example.pest"]
pub struct ExampleParser;

pub fn parse_example(input: &str) -> Result<Vec<Request>> {
    let pairs = ExampleParser::parse(Rule::HttpFile, input)?;
    
    let mut requests = Vec::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::RequestLine => {
                // Process request
            }
            _ => {}
        }
    }
    
    Ok(requests)
}
```

---

## Key References

- **Pest Book:** https://pest.rs/book
- **Pest Docs:** https://docs.rs/pest/latest/pest/
- **MSRV Note:** Pest requires Rust 1.83.0+; if your MSRV is newer, you're fine
- **Workspace Patterns:** Use `workspace.dependencies` to share pest + pest_derive across crates
- **httprunner reference:** `src/core/src/parser/pest_semantic_assembler.rs` replays `file_parser.rs` semantics over `PestHttpFile`, while `src/core/src/parser/pest_parser.rs` keeps top-level comment separators like `###` in the IR

---

## Platform & Release Notes

- **Binary size:** Zero impact (compile-only)
- **CI/CD:** +2-5s per target build (proc-macro overhead)
- **Distribution:** No changes to install scripts, Docker, or release artifacts
- **WASM:** Works identically to native targets (no target-specific code generation)
- **Cross-compilation:** Pest handles all native architectures (x86_64, aarch64, etc.)

---

## When to Escalate

- If compile-time increases >20%, escalate to platform/release team (may need sccache or crate partitioning)
- If grammar file becomes too large (>500 lines), consider splitting into submodules (advanced)
- If WASM target fails to compile, verify no platform-specific code in grammar or generated code

---

**Skill Owner:** Hicks  
**Last Updated:** 2026-04-06  
**Status:** Proven in httprunner migration investigation  
