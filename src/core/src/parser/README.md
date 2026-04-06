# Parser Module

This module handles parsing of HTTP request files (`.http`) with support for environment variables, directives, and variable substitution.

Production parsing now runs in two explicit stages:

1. `http-file.pest` classifies the grammar-owned shapes in the source file.
2. `pest_semantic_assembler.rs` reapplies the remaining stateful parser rules and builds `HttpRequest` values.

## Structure

- `mod.rs` - Module entry point and public API; exports the pest-backed `parse_http_file` / `parse_http_content` functions
- `http-file.peg` - Canonical human-readable spec for supported `.http` syntax and parser notes
- `http-file.pest` - Executable pest grammar used by the production parser backend
- `pest_parser.rs` - Runs the pest grammar and builds the intermediate parse tree
- `pest_parse_tree.rs` - Line-oriented intermediate representation shared between the grammar and semantic stages
- `pest_semantic_assembler.rs` - Production Rust semantic post-pass and request builder
- `file_parser.rs` - Legacy handwritten backend retained under `cfg(test)` for parity validation
- `substitution.rs` - Template variable substitution (`{{variable}}` syntax)
- `condition_parser.rs` - Parsing of `@if` and `@if-not` directives
- `timeout_parser.rs` - Parsing of timeout values with unit conversion
- `utils.rs` - HTTP method detection and utility functions
- `tests.rs` - Comprehensive test suite

## Grammar vs. semantic ownership

- `http-file.peg` remains the canonical spec for humans reading or reviewing the parser contract.
- `http-file.pest` owns executable syntax classification such as directives, requests, headers, variables, assertions, comments, blank lines, and IntelliJ script blocks.
- `pest_semantic_assembler.rs` still owns directive buffering onto the next request, blank-line header/body transitions, body-mode downgrades of `@...` and header-shaped lines, plus variable substitution, timeout parsing, and condition parsing.

## Usage

```rust
use crate::parser::parse_http_file;

let requests = parse_http_file("requests.http", Some("dev"))?;
```

See `http-file.peg` for the readable parser contract, `http-file.pest` for the executable grammar, and `pest_semantic_assembler.rs` for the Rust semantic post-pass used by the production parser backend.

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
