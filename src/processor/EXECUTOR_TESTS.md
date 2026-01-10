# Processor Executor Tests

## Overview

Comprehensive tests for the processor executor using **dependency injection with closures** - the idiomatic Rust approach.

## The Rust Way: Functions Over Traits

Instead of creating an interface/trait (the C# way), we use **function parameters** directly:

```rust
// Simple and idiomatic Rust
pub fn process_http_files_with_executor<F>(
    files: &[String],
    executor: &F,  // Just a function!
) -> Result<ProcessorResults>
where
    F: Fn(&HttpRequest, bool, bool) -> Result<HttpResult>,
```

Compare to the C# approach:
```csharp
// C# way - requires interface
interface IHttpExecutor {
    HttpResult Execute(HttpRequest req, bool verbose, bool insecure);
}

void ProcessHttpFiles(string[] files, IHttpExecutor executor) { ... }
```

## Why This is Better

1. **Less Code** - No trait definition needed
2. **More Flexible** - Can pass any compatible function or closure
3. **Idiomatic Rust** - This is how the standard library works (`map`, `filter`, etc.)
4. **Still Type-Safe** - Compiler enforces the function signature

## MockHttpExecutor: A Useful Test Helper

We keep `MockHttpExecutor` as a struct (not implementing a trait) because it provides useful functionality:

```rust
struct MockHttpExecutor {
    responses: Arc<Mutex<Vec<HttpResult>>>,    // Queue of responses
    call_count: Arc<Mutex<usize>>,             // Track calls
    executed_requests: Arc<Mutex<Vec<HttpRequest>>>,  // Verify requests
}

impl MockHttpExecutor {
    fn execute(&self, request: &HttpRequest, ...) -> Result<HttpResult> {
        // Returns predefined responses in sequence
        // Tracks calls for assertions
    }
}
```

## Usage Example

```rust
#[test]
fn test_with_mock() {
    // Create mock with predefined responses
    let mock = MockHttpExecutor::new(vec![
        create_success_response(),
        create_success_response(),
    ]);

    // Pass mock's execute method as a closure
    let result = process_http_files_with_executor(
        &files,
        &|req, v, i| mock.execute(req, v, i),  // Simple!
    );

    assert_eq!(mock.get_call_count(), 2);  // Verify behavior
}
```

For simple cases, you don't even need a struct:
```rust
#[test]
fn test_with_inline_closure() {
    let mut calls = 0;
    
    process_http_files_with_executor(
        &files,
        &|_req, _v, _i| {
            calls += 1;
            Ok(HttpResult { status_code: 200, ... })
        },
    );
    
    assert_eq!(calls, 1);
}
```

## Test Coverage: 13 Tests

### Core (5)
- ✅ Single request
- ✅ Multiple requests
- ✅ Failed requests
- ✅ Named requests
- ✅ Empty files

### Config (3)
- ✅ Verbose mode
- ✅ Insecure flag
- ✅ Multiple files

### HTTP (3)
- ✅ Headers
- ✅ Body
- ✅ All methods

### Advanced (2)
- ✅ Dependencies
- ✅ Skipped dependencies

## Running Tests

```bash
cargo test processor::executor_tests
```

## Key Takeaway

**In Rust, prefer functions/closures over traits** when you just need to pass behavior. Traits are powerful, but often overkill. This is a fundamental difference from C#/Java where interfaces are the primary abstraction mechanism.
