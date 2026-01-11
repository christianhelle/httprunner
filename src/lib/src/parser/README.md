# Parser Module

This module handles parsing of HTTP request files (.http) with support for environment variables, directives, and variable substitution.

## Structure

- `mod.rs` - Module entry point and public API
- `file_parser.rs` - Main HTTP file parsing logic
- `variable_substitution.rs` - Template variable substitution (`{{variable}}` syntax)
- `condition_parser.rs` - Parsing of `@if` and `@if-not` directives
- `timeout_parser.rs` - Parsing of timeout values with unit conversion
- `utils.rs` - HTTP method detection and utility functions
- `tests.rs` - Comprehensive test suite

## Usage

```rust
use crate::parser::parse_http_file;

let requests = parse_http_file("requests.http", Some("dev"))?;
```

## Supported Directives

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
# @TOKEN=abc123
GET https://api.example.com/data
Authorization: Bearer {{TOKEN}}
```

### Assertions
```
GET https://api.example.com/users/1
EXPECTED_RESPONSE_STATUS: 200
EXPECTED_RESPONSE_BODY: John
EXPECTED_RESPONSE_HEADERS: Content-Type: application/json
```

## Variable Substitution

Variables defined in environment files or with `@NAME=VALUE` can be referenced using `{{NAME}}` syntax in URLs, headers, and bodies.
