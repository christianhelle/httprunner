# Parser Module

This module handles parsing of HTTP request files (.http) with support for environment variables, directives, and variable substitution.

## Structure

- `mod.rs` - Module entry point and public API
- `file_parser.rs` - Main HTTP file parsing logic
- `http-file.peg` - PEG-style documentation grammar for the current supported `.http` syntax
- `http-file.pest` - Executable pest grammar scaffold kept in sync with the PEG spec
- `substitution.rs` - Template variable substitution (`{{variable}}` syntax)
- `condition_parser.rs` - Parsing of `@if` and `@if-not` directives
- `timeout_parser.rs` - Parsing of timeout values with unit conversion
- `utils.rs` - HTTP method detection and utility functions
- `tests.rs` - Comprehensive test suite

## Usage

```rust
use crate::parser::parse_http_file;

let requests = parse_http_file("requests.http", Some("dev"))?;
```

See `http-file.peg` for the canonical parser-adjacent grammar/spec and `http-file.pest` for the executable pest scaffold that mirrors the syntactic pieces owned by the grammar.

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
