# Conditions Module

This module handles conditional request execution based on previous request results.

## Structure

- `mod.rs` - Module entry point and public API
- `evaluator.rs` - Condition evaluation logic for `@if` and `@if-not` directives
- `dependency.rs` - Dependency checking for `@dependsOn` directive
- `formatter.rs` - Condition type formatting utilities
- `json_extractor.rs` - JSON value extraction for body conditions
- `tests.rs` - Comprehensive test suite

## Usage

### Evaluating Conditions

```rust
use crate::conditions::evaluate_conditions;

let all_met = evaluate_conditions(&request.conditions, &context)?;
if all_met {
    // Execute request
}
```

### Checking Dependencies

```rust
use crate::conditions::check_dependency;

if !check_dependency(&request.depends_on, &context) {
    // Skip request
}
```

## Supported Condition Types

### Status Condition
Checks HTTP status code from a previous request:
```
# @if login.response.status 200
# @if-not auth.response.status 404
```

### Body JSON Path Condition
Extracts and compares values from JSON response body:
```
# @if getUser.response.body.$.username johndoe
# @if-not checkToken.response.body.$.expired true
```

## Negation

The `@if-not` directive inverts condition logic. A condition is considered met when the comparison fails.
