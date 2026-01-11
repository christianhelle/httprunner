# Processor Module

This module handles the execution of HTTP requests from parsed HTTP files, including dependency resolution, conditional execution, variable substitution, and result aggregation.

## Structure

- `mod.rs` - Module entry point and public API
- `executor.rs` - Main request processing and execution logic
- `substitution.rs` - Request variable substitution in request components
- `formatter.rs` - JSON and output formatting utilities
- `tests.rs` - Test suite

## Usage

```rust
use crate::processor::process_http_files;

let results = process_http_files(
    &["requests.http"],
    true,  // verbose
    Some("output.log"),
    Some("dev"),  // environment
    false,  // insecure
    true    // pretty_json
)?;
```

## Features

### Request Execution Flow
1. Parse HTTP file(s)
2. For each request:
   - Check dependencies (`@dependsOn`)
   - Evaluate conditions (`@if`, `@if-not`)
   - Substitute request variables
   - Execute HTTP request
   - Evaluate assertions
   - Store context for subsequent requests

### Variable Substitution
Request variables from previous requests can be referenced using:
```
{{request_name.request.body}}
{{request_name.response.body.$.property}}
{{request_name.response.headers.Content-Type}}
```

### Result Aggregation
Results are aggregated per file and overall:
- Success count
- Failed count
- Skipped count
- Success rate calculation
- Per-request detailed context
