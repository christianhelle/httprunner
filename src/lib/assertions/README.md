# Assertions Module

This module handles the evaluation of assertions for HTTP responses, including status code, header, and body content validation.

## Structure

- `mod.rs` - Module entry point and public API
- `evaluator.rs` - Assertion evaluation logic for all assertion types
- `tests.rs` - Test suite

## Usage

```rust
use crate::assertions::evaluate_assertions;

let assertion_results = evaluate_assertions(&request.assertions, &http_result)?;
for result in assertion_results {
    if !result.passed {
        println!("Assertion failed: {}", result.message);
    }
}
```

## Supported Assertion Types

### Status Code Assertion
Validates the HTTP response status code:
```
EXPECTED_RESPONSE_STATUS: 200
EXPECTED_RESPONSE_STATUS: 404
```

### Header Assertion
Validates response header values:
```
EXPECTED_RESPONSE_HEADERS: Content-Type: application/json
EXPECTED_RESPONSE_HEADERS: Authorization: Bearer token
```

### Body Content Assertion
Validates response body contains expected text:
```
EXPECTED_RESPONSE_BODY: success
EXPECTED_RESPONSE_BODY: {"status":"ok"}
```

## Assertion Results

Each assertion produces an `AssertionResult` with:
- `assertion_type`: Type of assertion (Status, Headers, Body)
- `expected`: Expected value
- `actual`: Actual value from response
- `passed`: Boolean indicating pass/fail
- `message`: Descriptive message about the result
