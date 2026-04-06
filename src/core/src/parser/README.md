# Parser Module

This module handles parsing of HTTP request files (`.http`) with support for environment variables, directives, and variable substitution.

Production parsing uses the pest-backed semantic assembler in `pest_semantic_assembler.rs`. The handwritten state-machine parser in `file_parser.rs` remains available under `#[cfg(test)]` for parity tests and benchmark comparisons.

## Structure

- `mod.rs` - Module entry point and public API; exports the pest-backed `parse_http_file` / `parse_http_content`
- `file_parser.rs` - Legacy handwritten state-machine parser retained under `cfg(test)` for parity and benchmark comparisons
- `http-file.peg` - Canonical human-readable spec for supported `.http` syntax and parser notes
- `http-file.pest` - Executable pest grammar used by the structured parse-tree path and parser validation
- `pest_parser.rs` - Runs the pest grammar and builds the intermediate parse tree
- `pest_parse_tree.rs` - Line-oriented intermediate representation shared between the grammar and semantic stages
- `pest_semantic_assembler.rs` - Production pest-backed semantic post-pass and request builder
- `substitution.rs` - Template variable substitution (`{{variable}}` syntax)
- `condition_parser.rs` - Parsing of `@if` and `@if-not` directives
- `timeout_parser.rs` - Parsing of timeout values with unit conversion
- `utils.rs` - HTTP method detection and utility functions
- `tests.rs` - Comprehensive test suite

## Grammar vs. semantic ownership

- `http-file.peg` remains the canonical spec for humans reading or reviewing the parser contract.
- Production parsing now routes through `pest_semantic_assembler.rs`.
- `file_parser.rs` remains available under `cfg(test)` as the legacy handwritten comparison backend.
- `http-file.pest` owns executable syntax validation for the structured parse-tree path, while the production hot path uses zero-copy line splitting before applying the same Rust semantic state machine.
- `pest_semantic_assembler.rs` reapplies directive buffering, body-mode transitions, and remaining semantic rules after line splitting or parse-tree construction.

## Usage

```rust
use crate::parser::parse_http_file;

let requests = parse_http_file("requests.http", Some("dev"))?;
```

See `http-file.peg` for the readable parser contract and `pest_semantic_assembler.rs` for the production implementation. The handwritten parser remains available under `cfg(test)` for parity and benchmark comparisons.

## Supported Directives

Directive examples below can use either `# @...` or `// @...` prefixes.

### Request Naming
```
# @name login
POST https://api.example.com/login
```

### Timeouts
```
# @timeout 5000ms
# @timeout 5s
# @connection-timeout 3s
GET https://api.example.com/slow
```

### Dependencies
```
# @dependsOn login
GET https://api.example.com/profile
```

### Conditional Execution
```
# @if login.response.status 200
# @if-not auth.response.body.$.expired true
GET https://api.example.com/data
```

### Variables
```
@TOKEN=abc123
GET https://api.example.com/data
Authorization: Bearer {{TOKEN}}
```

### Assertions
```
GET https://api.example.com/users/1
EXPECTED_RESPONSE_STATUS 200
EXPECTED_RESPONSE_BODY John
> EXPECTED_RESPONSE_HEADERS Content-Type: application/json
```

## Variable Substitution

Variables defined in environment files or with `@NAME=VALUE` can be referenced using `{{NAME}}` syntax in URLs, headers, and bodies.
