# Parser Module

This module handles parsing of HTTP request files (`.http`) with support for environment variables, directives, and variable substitution.

Production parsing uses the handwritten state-machine parser in `file_parser.rs`. A pest-based backend (`pest_semantic_assembler.rs`) is present and passes all parity tests but is not yet the production default due to performance regression. It remains available under `#[cfg(test)]` for continued iteration and benchmarking.

## Structure

- `mod.rs` - Module entry point and public API; exports `parse_http_file` / `parse_http_content` from the handwritten parser
- `file_parser.rs` - Production handwritten state-machine parser
- `http-file.peg` - Canonical human-readable spec for supported `.http` syntax and parser notes
- `http-file.pest` - Executable pest grammar (not yet the production default)
- `pest_parser.rs` - Runs the pest grammar and builds the intermediate parse tree
- `pest_parse_tree.rs` - Line-oriented intermediate representation shared between the grammar and semantic stages
- `pest_semantic_assembler.rs` - Pest-backed semantic post-pass and request builder (available under `cfg(test)` for parity validation)
- `substitution.rs` - Template variable substitution (`{{variable}}` syntax)
- `condition_parser.rs` - Parsing of `@if` and `@if-not` directives
- `timeout_parser.rs` - Parsing of timeout values with unit conversion
- `utils.rs` - HTTP method detection and utility functions
- `tests.rs` - Comprehensive test suite

## Grammar vs. semantic ownership

- `http-file.peg` remains the canonical spec for humans reading or reviewing the parser contract.
- `file_parser.rs` is the production parser and owns all runtime behavior.
- `http-file.pest` owns executable syntax classification (directives, requests, headers, variables, assertions, comments, blank lines, IntelliJ script blocks) for the pest backend, which is not yet the production default.
- `pest_semantic_assembler.rs` reapplies directive buffering, body-mode transitions, and remaining semantic rules on top of the pest parse tree. It is available for parity testing and benchmarking.

## Usage

```rust
use crate::parser::parse_http_file;

let requests = parse_http_file("requests.http", Some("dev"))?;
```

See `http-file.peg` for the readable parser contract and `file_parser.rs` for the production implementation. The pest backend (`http-file.pest` + `pest_semantic_assembler.rs`) is available for testing but not yet the default.

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
